//! Scrape Job Processor
//!
//! Processes hotel price scraping jobs.

use anyhow::Result;
use futures::future::join_all;
use std::sync::Arc;

use crate::api::AppState;
use crate::cache::{CacheKeys, CacheOps};
use crate::db::{CurrencyRepo, HotelRepo, MaterializedViewRepo, PriceHistoryRepo, ScrapeJobRepo, ScrapeResultRepo};
use crate::excel::merge_job_params;
use crate::models::scrape_job::{Device, LoginState, ScrapeMethod};
use crate::models::{Hotel, HotelScrapeStatus, JobDefaults, ScrapeJobMessage, ScrapeJobStatus};
use crate::scraper::{ScrapeParams, ScrapeResult as ScraperResult, Scraper};

/// Process a scrape job
pub async fn process_scrape_job(state: &Arc<AppState>, message: ScrapeJobMessage) -> Result<()> {
    let job_id = message.job_id;

    // Update job status to processing
    ScrapeJobRepo::update_status(&state.db, job_id, ScrapeJobStatus::Processing).await?;

    // Get hotels for this group
    let hotels = HotelRepo::get_for_scrape_job(&state.db, message.hotel_group_id).await?;

    if hotels.is_empty() {
        tracing::warn!("No hotels found for group {}", message.hotel_group_id);
        ScrapeJobRepo::update_status(&state.db, job_id, ScrapeJobStatus::Completed).await?;
        return Ok(());
    }

    tracing::info!("Processing {} hotels for job {}", hotels.len(), job_id);

    // Process hotels in parallel batches (concurrency = 3)
    let concurrency = state.config.worker_concurrency;
    let chunks: Vec<_> = hotels.chunks(concurrency).collect();

    for chunk in chunks {
        // Check if job was cancelled
        let job = ScrapeJobRepo::get_by_id(&state.db, job_id).await?;
        if job.status == ScrapeJobStatus::Cancelled {
            tracing::info!("Job {} was cancelled, stopping", job_id);
            return Ok(());
        }

        // Process chunk in parallel
        let futures: Vec<_> = chunk
            .iter()
            .map(|hotel| process_hotel(state, &message, hotel))
            .collect();

        join_all(futures).await;
    }

    // Check if all hotels are processed
    let is_complete = ScrapeJobRepo::is_job_complete(&state.db, job_id).await?;

    if is_complete {
        ScrapeJobRepo::update_status(&state.db, job_id, ScrapeJobStatus::Completed).await?;
        tracing::info!("Job {} completed", job_id);

        // Keep analytics current after every job (scheduled or on-demand) —
        // stricter than REQ-003's NF-004 but costs nothing extra and keeps
        // a live demo from showing stale numbers.
        if let Err(e) = MaterializedViewRepo::refresh_all(&state.db).await {
            tracing::warn!("Failed to refresh analytics materialized views: {}", e);
        }
    }

    Ok(())
}

/// Process a single hotel: resolve per-hotel overrides (REQ-001 F-002
/// JobDefaults fallback), then run one scrape pass per `los_variants` entry.
async fn process_hotel(state: &Arc<AppState>, message: &ScrapeJobMessage, hotel: &Hotel) {
    let job_id = message.job_id;
    let hotel_id = hotel.id;

    tracing::debug!("Processing hotel: {} ({})", hotel.name, hotel_id);

    let _ = ScrapeJobRepo::update_hotel_status(
        &state.db,
        job_id,
        hotel_id,
        HotelScrapeStatus::Processing,
        None,
    )
    .await;

    let defaults = JobDefaults {
        checkin_date: message.checkin_date,
        checkout_date: message.checkout_date,
        rooms: message.rooms,
        adults: message.adults,
    };
    let override_row = ScrapeJobRepo::get_hotel_param_override(&state.db, job_id, hotel_id)
        .await
        .ok()
        .flatten();
    let resolved = merge_job_params(defaults, override_row.as_ref());

    let los_variants = if message.los_variants.is_empty() {
        vec![1]
    } else {
        message.los_variants.clone()
    };

    let max_retries = state.config.worker_retry_count;
    let mut any_success = false;
    let mut last_error: Option<String> = None;

    for los_nights in los_variants {
        let checkout_date = resolved
            .checkin_date
            .checked_add_signed(chrono::Duration::days(los_nights as i64))
            .unwrap_or(resolved.checkout_date);

        let params = ScrapeParams {
            hotel_name: hotel.name.clone(),
            city: hotel.city.clone(),
            country: hotel.country.clone(),
            checkin_date: resolved.checkin_date,
            checkout_date,
            rooms: resolved.rooms,
            adults: resolved.adults,
            los_nights,
            device: message.device,
            login_state: message.login_state,
        };

        let cache_key = CacheKeys::hotel_price_v2(
            hotel_id,
            params.checkin_date,
            params.checkout_date,
            params.rooms,
            params.adults,
            method_label(message.method),
            device_label(message.device),
            login_state_label(message.login_state),
            los_nights,
        );

        if !message.force_refresh {
            let mut redis = state.redis.clone();
            if let Ok(Some(cached)) =
                CacheOps::get::<Vec<ScraperResult>>(&mut redis, &cache_key).await
            {
                tracing::debug!("Using cached results for hotel {}", hotel.name);
                save_results(state, job_id, hotel_id, &cached, los_nights, message, &params).await;
                any_success = true;
                continue;
            }
        }

        let mut succeeded = false;
        for attempt in 0..=max_retries {
            if attempt > 0 {
                tracing::debug!("Retry {} for hotel {}", attempt, hotel.name);
            }

            match scrape_hotel_prices(state, &params, message.method).await {
                Ok(results) => {
                    save_results(state, job_id, hotel_id, &results, los_nights, message, &params).await;

                    let mut redis = state.redis.clone();
                    let _ = CacheOps::set(
                        &mut redis,
                        &cache_key,
                        &results,
                        state.config.price_cache_ttl_seconds,
                    )
                    .await;

                    tracing::debug!(
                        "Successfully scraped {} prices for {} ({}n)",
                        results.len(),
                        hotel.name,
                        los_nights
                    );
                    succeeded = true;
                    break;
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    tracing::warn!("Failed to scrape {} ({}n): {}", hotel.name, los_nights, e);
                }
            }
        }

        any_success = any_success || succeeded;
    }

    if any_success {
        let _ = ScrapeJobRepo::update_hotel_status(
            &state.db,
            job_id,
            hotel_id,
            HotelScrapeStatus::Success,
            None,
        )
        .await;
    } else {
        let _ = ScrapeJobRepo::update_hotel_status(
            &state.db,
            job_id,
            hotel_id,
            HotelScrapeStatus::Failed,
            last_error.as_deref(),
        )
        .await;
        tracing::error!("All retries failed for hotel {}", hotel.name);
    }
}

async fn save_results(
    state: &Arc<AppState>,
    job_id: uuid::Uuid,
    hotel_id: uuid::Uuid,
    results: &[ScraperResult],
    los_nights: i32,
    message: &ScrapeJobMessage,
    params: &ScrapeParams,
) {
    for result in results {
        let _ = ScrapeResultRepo::create(
            &state.db,
            job_id,
            hotel_id,
            &result.source,
            &result.room_type,
            result.price_thb,
            result.original_price,
            result.currency.as_deref(),
            result.meal_plan.as_deref(),
            result.cancellation.as_deref(),
            result.source_url.as_deref(),
            los_nights,
            message.device,
            message.login_state,
            result.who_id.as_deref(),
            message.method,
        )
        .await;

        // Dual-write (REQ-002 F-001): also persist to the long-term,
        // queryable hotel_price_history table. Best-effort — a failure
        // here (e.g. rate lookup) must not fail the job, matching the
        // `let _ =` discipline already used for scrape_results above.
        let currency = result.currency.as_deref().unwrap_or("THB");
        if let Ok(exchange_rate_id) = CurrencyRepo::get_or_create_rate(&state.db, currency, "THB").await {
            let _ = PriceHistoryRepo::create(
                &state.db,
                hotel_id,
                &result.source,
                &result.room_type,
                result.price_thb,
                result.original_price,
                result.currency.as_deref(),
                exchange_rate_id,
                result.meal_plan.as_deref(),
                result.cancellation.as_deref(),
                result.source_url.as_deref(),
                params.checkin_date,
                params.checkout_date,
                params.rooms as i16,
                params.adults as i16,
                Some(job_id),
            )
            .await;
        } else {
            tracing::warn!("Failed to resolve exchange rate for {}, skipping price_history write", currency);
        }
    }
}

fn method_label(method: ScrapeMethod) -> &'static str {
    match method {
        ScrapeMethod::Serpapi => "serpapi",
        ScrapeMethod::Chatgpt => "chatgpt",
        ScrapeMethod::Gemini => "gemini",
        ScrapeMethod::Both => "both",
    }
}

fn device_label(device: Device) -> &'static str {
    match device {
        Device::Desktop => "desktop",
        Device::MobileWeb => "mobile_web",
    }
}

fn login_state_label(login_state: LoginState) -> &'static str {
    match login_state {
        LoginState::Public => "public",
        LoginState::Member => "member",
    }
}

/// Scrape hotel prices from all sources for the job's configured method,
/// via the scraper adapter registry (scraper/registry.rs) — adding a new
/// scraper later means implementing `ScraperFactory`, not touching this
/// function. Device/login_state are recorded as configuration metadata on
/// the params (see ScrapeParams) — SerpAPI/Gother have no documented way
/// to actually vary results by those axes today; this is a known gap.
async fn scrape_hotel_prices(
    state: &Arc<AppState>,
    params: &ScrapeParams,
    method: ScrapeMethod,
) -> Result<Vec<ScraperResult>> {
    // Mock fallback: only when method is exactly Serpapi (not Both) and no
    // SERPAPI_KEY is configured — preserves the original no-key dev/demo
    // experience byte-for-byte. Any other method with a missing key just
    // skips that factory (see loop below), never silently substituting
    // fabricated data.
    let serpapi_configured =
        !state.config.serpapi_key.is_empty() && state.config.serpapi_key != "your_serpapi_key_here";
    if method == ScrapeMethod::Serpapi && !serpapi_configured {
        tracing::info!("Using mock scraper (SERPAPI_KEY not configured)");
        let mock = crate::scraper::MockScraper::new();
        return mock.scrape(params).await;
    }

    let mut all_results = Vec::new();

    for factory in state.scraper_registry.iter() {
        if !factory.methods().contains(&method) {
            continue;
        }
        match factory.build(&state.config) {
            Some(scraper) => match scraper.scrape(params).await {
                Ok(results) => all_results.extend(results),
                Err(e) => tracing::warn!("{} scrape failed: {}", scraper.name(), e),
            },
            None => tracing::debug!("A scraper for method {:?} is not configured, skipping", method),
        }
    }

    if all_results.is_empty() {
        anyhow::bail!("No results from any source");
    }

    Ok(all_results)
}
