//! Scrape Job Processor
//!
//! Processes hotel price scraping jobs.

use anyhow::Result;
use futures::future::join_all;
use std::sync::Arc;

use crate::api::AppState;
use crate::cache::{CacheKeys, CacheOps};
use crate::db::{HotelRepo, ScrapeJobRepo, ScrapeResultRepo};
use crate::models::{Hotel, HotelScrapeStatus, ScrapeJobMessage, ScrapeJobStatus};
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
    }

    Ok(())
}

/// Process a single hotel
async fn process_hotel(state: &Arc<AppState>, message: &ScrapeJobMessage, hotel: &Hotel) {
    let job_id = message.job_id;
    let hotel_id = hotel.id;

    tracing::debug!("Processing hotel: {} ({})", hotel.name, hotel_id);

    // Update status to processing
    let _ = ScrapeJobRepo::update_hotel_status(
        &state.db,
        job_id,
        hotel_id,
        HotelScrapeStatus::Processing,
        None,
    )
    .await;

    // Check cache first (unless force_refresh)
    if !message.force_refresh {
        let cache_key = CacheKeys::hotel_price(
            hotel_id,
            message.checkin_date,
            message.checkout_date,
            message.rooms,
            message.adults,
        );

        let mut redis = state.redis.clone();
        if let Ok(Some(cached)) = CacheOps::get::<Vec<ScraperResult>>(&mut redis, &cache_key).await
        {
            tracing::debug!("Using cached results for hotel {}", hotel.name);

            // Save cached results to database
            for result in cached {
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
                )
                .await;
            }

            let _ = ScrapeJobRepo::update_hotel_status(
                &state.db,
                job_id,
                hotel_id,
                HotelScrapeStatus::Success,
                None,
            )
            .await;

            return;
        }
    }

    // Scrape from APIs
    let params = ScrapeParams {
        hotel_name: hotel.name.clone(),
        city: hotel.city.clone(),
        country: hotel.country.clone(),
        checkin_date: message.checkin_date,
        checkout_date: message.checkout_date,
        rooms: message.rooms,
        adults: message.adults,
    };

    // Try scraping with retry
    let max_retries = state.config.worker_retry_count;
    let mut last_error: Option<String> = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            tracing::debug!("Retry {} for hotel {}", attempt, hotel.name);
        }

        match scrape_hotel_prices(state, &params).await {
            Ok(results) => {
                // Save results to database
                for result in &results {
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
                    )
                    .await;
                }

                // Cache the results
                let cache_key = CacheKeys::hotel_price(
                    hotel_id,
                    message.checkin_date,
                    message.checkout_date,
                    message.rooms,
                    message.adults,
                );
                let mut redis = state.redis.clone();
                let _ = CacheOps::set(
                    &mut redis,
                    &cache_key,
                    &results,
                    state.config.price_cache_ttl_seconds,
                )
                .await;

                // Update status to success
                let _ = ScrapeJobRepo::update_hotel_status(
                    &state.db,
                    job_id,
                    hotel_id,
                    HotelScrapeStatus::Success,
                    None,
                )
                .await;

                tracing::debug!("Successfully scraped {} prices for {}", results.len(), hotel.name);
                return;
            }
            Err(e) => {
                last_error = Some(e.to_string());
                tracing::warn!("Failed to scrape {}: {}", hotel.name, e);
            }
        }
    }

    // All retries failed
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

/// Scrape hotel prices from all sources
async fn scrape_hotel_prices(state: &Arc<AppState>, params: &ScrapeParams) -> Result<Vec<ScraperResult>> {
    let mut all_results = Vec::new();

    // Check if we should use mock scraper (for testing without API keys)
    let use_mock = state.config.serpapi_key.is_empty() 
        || state.config.serpapi_key == "your_serpapi_key_here";

    if use_mock {
        tracing::info!("Using mock scraper (no API keys configured)");
        let mock = crate::scraper::MockScraper::new();
        return mock.scrape(params).await;
    }

    // Scrape from SerpAPI (Google Hotels)
    let serpapi = crate::scraper::SerpApiScraper::new(&state.config.serpapi_key);
    match serpapi.scrape(params).await {
        Ok(results) => {
            all_results.extend(results);
        }
        Err(e) => {
            tracing::warn!("SerpAPI scrape failed: {}", e);
        }
    }

    // Scrape from Gother API
    if !state.config.gother_api_url.is_empty() 
        && state.config.gother_api_key != "your_gother_api_key_here" 
    {
        let gother = crate::scraper::GotherScraper::new(
            &state.config.gother_api_url,
            &state.config.gother_api_key,
        );
        match gother.scrape(params).await {
            Ok(results) => {
                all_results.extend(results);
            }
            Err(e) => {
                tracing::warn!("Gother API scrape failed: {}", e);
            }
        }
    }

    if all_results.is_empty() {
        anyhow::bail!("No results from any source");
    }

    Ok(all_results)
}
