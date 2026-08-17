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
use crate::models::scrape_job::{LoginState, ScrapeMethod};
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
            message.device.as_str(),
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
                message.device,
                &result.via_method,
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
        ScrapeMethod::Gemini => "gemini",
        ScrapeMethod::Both => "both",
    }
}

fn login_state_label(login_state: LoginState) -> &'static str {
    match login_state {
        LoginState::Public => "public",
        LoginState::Member => "member",
    }
}

/// What one scraper did during a single hotel scrape, so a failure can
/// name the responsible source instead of a blanket "no results".
#[derive(Debug, PartialEq)]
enum Outcome {
    Ok(usize),
    /// Ran and succeeded, but produced no rows — e.g. Gemini declining to
    /// guess, or SerpAPI returning only non-named providers (ADR-005).
    Empty,
    /// No credential, so the factory never built a scraper.
    NotConfigured,
    /// A fallback source deliberately not run because the primary tier
    /// already returned prices. Distinct from a failure — nothing is wrong.
    SkippedPrimaryHadPrices,
    Failed(String),
}

/// Should a deferred fallback source run? Only under `method=both`, and
/// only when the primary tier found nothing at all. Pure so the precedence
/// rule is testable without a DB, network or registry.
fn should_run_fallback(method: ScrapeMethod, primary_rows: usize) -> bool {
    method == ScrapeMethod::Both && primary_rows == 0
}

/// Build and run one factory, tagging every row it produced with its name
/// so provenance is recorded at the point of production and cannot drift.
async fn run_factory(
    state: &Arc<AppState>,
    factory: &dyn crate::scraper::registry::ScraperFactory,
    params: &ScrapeParams,
) -> (Outcome, Vec<ScraperResult>) {
    let Some(scraper) = factory.build(&state.config) else {
        return (Outcome::NotConfigured, Vec::new());
    };

    match scraper.scrape(params).await {
        Ok(rows) if rows.is_empty() => (Outcome::Empty, Vec::new()),
        Ok(mut rows) => {
            for row in &mut rows {
                row.via_method = factory.name().to_string();
            }
            (Outcome::Ok(rows.len()), rows)
        }
        Err(e) => {
            tracing::warn!("{} scrape failed: {}", factory.name(), e);
            (Outcome::Failed(e.to_string()), Vec::new())
        }
    }
}

/// Render per-scraper outcomes into one human-readable line. Pure, so the
/// wording is unit-testable without a DB or network (same precedent as
/// `is_due`, `standard_grid`, `partition_ranges`).
fn summarize_outcomes(outcomes: &[(&str, Outcome)]) -> String {
    // A disabled mock scraper is the normal, desired state — reporting
    // "mock: not configured" in a user-facing error just adds noise and
    // invites someone to "fix" it by turning fabricated data back on.
    // Anything else it does (including producing rows) stays visible.
    let visible: Vec<_> = outcomes
        .iter()
        .filter(|(name, outcome)| !(*name == "mock" && *outcome == Outcome::NotConfigured))
        .collect();

    if visible.is_empty() {
        return "no scrapers are registered for this method".to_string();
    }

    visible
        .iter()
        .map(|(name, outcome)| match outcome {
            Outcome::Ok(n) => format!("{name}: {n} price(s)"),
            Outcome::Empty => format!("{name}: returned no rates"),
            Outcome::NotConfigured => format!("{name}: not configured"),
            Outcome::SkippedPrimaryHadPrices => {
                format!("{name}: skipped (primary source had prices)")
            }
            Outcome::Failed(e) => format!("{name}: failed ({e})"),
        })
        .collect::<Vec<_>>()
        .join("; ")
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
    // A missing credential yields "not configured", never fabricated data.
    // The mock scraper is a normal registry entry gated on
    // ENABLE_MOCK_SCRAPER (ADR-008).
    let mut all_results: Vec<ScraperResult> = Vec::new();
    let mut outcomes: Vec<(&'static str, Outcome)> = Vec::new();

    // Two tiers (ADR-011). Under `method=both` SerpAPI is authoritative and
    // Gemini only fills total blanks, so a real scraped price is never sat
    // next to an AI estimate for the same hotel and date. A fallback chosen
    // explicitly (`method=gemini`) is not deferred — it *is* the primary.
    let (deferred, primary): (Vec<_>, Vec<_>) = state
        .scraper_registry
        .iter()
        .filter(|f| f.methods().contains(&method))
        .partition(|f| f.is_fallback() && method == ScrapeMethod::Both);

    for factory in primary {
        let (outcome, rows) = run_factory(state, factory.as_ref(), params).await;
        all_results.extend(rows);
        outcomes.push((factory.name(), outcome));
    }

    let primary_rows = all_results.len();
    for factory in deferred {
        if !should_run_fallback(method, primary_rows) {
            outcomes.push((factory.name(), Outcome::SkippedPrimaryHadPrices));
            continue;
        }
        tracing::info!(
            "No prices from the primary tier for {}; trying fallback {}",
            params.hotel_name,
            factory.name()
        );
        let (outcome, rows) = run_factory(state, factory.as_ref(), params).await;
        all_results.extend(rows);
        outcomes.push((factory.name(), outcome));
    }

    if all_results.is_empty() {
        // Say which source did what — "No results from any source" alone
        // cannot distinguish a missing API key from a provider that simply
        // had no rates, which has cost real debugging time.
        anyhow::bail!("No prices found — {}", summarize_outcomes(&outcomes));
    }

    Ok(all_results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_unconfigured_source() {
        let out = summarize_outcomes(&[("serpapi", Outcome::NotConfigured)]);
        assert_eq!(out, "serpapi: not configured");
    }

    /// The distinction that matters most: "no API key" must not read the
    /// same as "the provider had nothing".
    #[test]
    fn distinguishes_not_configured_from_empty() {
        let out = summarize_outcomes(&[
            ("serpapi", Outcome::NotConfigured),
            ("gemini", Outcome::Empty),
        ]);
        assert_eq!(out, "serpapi: not configured; gemini: returned no rates");
    }

    #[test]
    fn includes_the_underlying_error_when_a_scraper_fails() {
        let out = summarize_outcomes(&[("serpapi", Outcome::Failed("HTTP 401".into()))]);
        assert!(out.contains("serpapi: failed"), "got: {out}");
        assert!(out.contains("HTTP 401"), "underlying cause must survive: {out}");
    }

    #[test]
    fn reports_counts_for_sources_that_did_return_rows() {
        let out = summarize_outcomes(&[("serpapi", Outcome::Ok(2)), ("gother", Outcome::NotConfigured)]);
        assert_eq!(out, "serpapi: 2 price(s); gother: not configured");
    }

    /// Precedence rule (ADR-011): Gemini fills blanks, it never competes
    /// with a scraped price.
    #[test]
    fn fallback_runs_only_for_both_and_only_when_primary_found_nothing() {
        assert!(should_run_fallback(ScrapeMethod::Both, 0));
        assert!(!should_run_fallback(ScrapeMethod::Both, 1));
        assert!(!should_run_fallback(ScrapeMethod::Both, 17));
    }

    /// Choosing a fallback source explicitly must not defer it — with
    /// `method=gemini` there is no primary tier to wait for.
    #[test]
    fn explicit_method_never_defers() {
        assert!(!should_run_fallback(ScrapeMethod::Gemini, 0));
        assert!(!should_run_fallback(ScrapeMethod::Serpapi, 0));
    }

    #[test]
    fn skipped_fallback_reads_as_a_skip_not_a_failure() {
        let out = summarize_outcomes(&[
            ("serpapi", Outcome::Ok(3)),
            ("gemini", Outcome::SkippedPrimaryHadPrices),
        ]);
        assert_eq!(out, "serpapi: 3 price(s); gemini: skipped (primary source had prices)");
    }

    #[test]
    fn handles_no_matching_scrapers() {
        assert_eq!(
            summarize_outcomes(&[]),
            "no scrapers are registered for this method"
        );
    }

    /// A disabled mock scraper is the desired state, so it should not
    /// clutter an error the user reads — but if it actually ran, say so.
    #[test]
    fn hides_disabled_mock_but_reports_an_active_one() {
        let hidden = summarize_outcomes(&[
            ("serpapi", Outcome::Empty),
            ("mock", Outcome::NotConfigured),
        ]);
        assert_eq!(hidden, "serpapi: returned no rates");

        let shown = summarize_outcomes(&[("mock", Outcome::Empty)]);
        assert_eq!(shown, "mock: returned no rates");
    }
}
