//! Scrape Result Repository
//!
//! Database operations for scrape results.

use crate::error::AppResult;
use crate::models::scrape_job::{Device, LoginState, ScrapeMethod};
use crate::models::{
    HotelInfo, HotelPriceComparison, HotelScrapeStatus, PriceEntry, ScrapeResult,
    ScrapeJobInfo, ScrapeResultsResponse, ScrapeResultsSummary,
};
use crate::scraper::providers::{is_direct_contract, GOTHER};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Scrape Result Repository
pub struct ScrapeResultRepo;

impl ScrapeResultRepo {
    /// Save a scrape result
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        job_id: Uuid,
        hotel_id: Uuid,
        source: &str,
        room_type: &str,
        price_thb: f64,
        original_price: Option<f64>,
        currency: Option<&str>,
        meal_plan: Option<&str>,
        cancellation: Option<&str>,
        source_url: Option<&str>,
        los_nights: i32,
        device: Device,
        login_state: LoginState,
        who_id: Option<&str>,
        via_method: ScrapeMethod,
    ) -> AppResult<ScrapeResult> {
        let row = sqlx::query(
            r#"
            INSERT INTO scrape_results
                (scrape_job_id, hotel_id, source, room_type, price_thb,
                 original_price, currency, meal_plan, cancellation, source_url,
                 los_nights, device, login_state, who_id, via_method)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, scrape_job_id, hotel_id, source, room_type, price_thb::float8 as price_thb,
                      original_price::float8 as original_price, currency, meal_plan, cancellation,
                      source_url, scraped_at, los_nights, device, login_state, who_id, via_method
            "#
        )
        .bind(job_id)
        .bind(hotel_id)
        .bind(source)
        .bind(room_type)
        .bind(price_thb)
        .bind(original_price)
        .bind(currency)
        .bind(meal_plan)
        .bind(cancellation)
        .bind(source_url)
        .bind(los_nights)
        .bind(device)
        .bind(login_state)
        .bind(who_id)
        .bind(via_method)
        .fetch_one(pool)
        .await?;

        Ok(ScrapeResult {
            id: row.get("id"),
            scrape_job_id: row.get("scrape_job_id"),
            hotel_id: row.get("hotel_id"),
            source: row.get("source"),
            room_type: row.get("room_type"),
            price_thb: row.get("price_thb"),
            original_price: row.get("original_price"),
            currency: row.get("currency"),
            meal_plan: row.get("meal_plan"),
            cancellation: row.get("cancellation"),
            source_url: row.get("source_url"),
            scraped_at: row.get("scraped_at"),
            los_nights: row.get("los_nights"),
            device: row.get("device"),
            login_state: row.get("login_state"),
            who_id: row.get("who_id"),
            via_method: row.get("via_method"),
        })
    }

    /// Get all results for a job with hotel comparisons
    pub async fn get_job_results(pool: &PgPool, job_id: Uuid) -> AppResult<ScrapeResultsResponse> {
        // Get job info
        let job_row = sqlx::query(
            r#"
            SELECT id, checkin_date, checkout_date, rooms, adults,
                   status::text as status, method, device, login_state,
                   created_at, completed_at
            FROM scrape_jobs
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .fetch_one(pool)
        .await?;

        let job = ScrapeJobInfo {
            id: job_row.get("id"),
            checkin_date: job_row.get::<chrono::NaiveDate, _>("checkin_date").to_string(),
            checkout_date: job_row.get::<chrono::NaiveDate, _>("checkout_date").to_string(),
            rooms: job_row.get("rooms"),
            adults: job_row.get("adults"),
            status: job_row.get("status"),
            method: job_row.get("method"),
            device: job_row.get("device"),
            login_state: job_row.get("login_state"),
            created_at: job_row.get("created_at"),
            completed_at: job_row.get("completed_at"),
        };

        // Get all hotel statuses
        let hotel_rows = sqlx::query(
            r#"
            SELECT
                h.id as hotel_id,
                h.name as hotel_name,
                h.city,
                h.country,
                h.hid,
                shs.status::text as status,
                shs.error_message
            FROM scrape_hotel_status shs
            JOIN hotels h ON h.id = shs.hotel_id
            WHERE shs.scrape_job_id = $1
            ORDER BY h.name
            "#
        )
        .bind(job_id)
        .fetch_all(pool)
        .await?;

        // Get all price results
        let price_rows = sqlx::query(
            r#"
            SELECT id, scrape_job_id, hotel_id, source, room_type, price_thb::float8 as price_thb,
                   original_price::float8 as original_price, currency, meal_plan, cancellation,
                   source_url, scraped_at, los_nights, device, login_state, who_id, via_method
            FROM scrape_results
            WHERE scrape_job_id = $1
            ORDER BY hotel_id, price_thb
            "#
        )
        .bind(job_id)
        .fetch_all(pool)
        .await?;

        // Build comparison results
        let mut results: Vec<HotelPriceComparison> = Vec::new();
        let mut total_best_price = 0.0;
        let mut successful_count = 0;
        let mut failed_count = 0;

        for hotel_row in &hotel_rows {
            let hotel_id: Uuid = hotel_row.get("hotel_id");
            let status_str: String = hotel_row.get("status");

            let status = match status_str.as_str() {
                "success" => HotelScrapeStatus::Success,
                "failed" => HotelScrapeStatus::Failed,
                "processing" => HotelScrapeStatus::Processing,
                _ => HotelScrapeStatus::Pending,
            };

            // Get prices for this hotel
            let hotel_price_rows: Vec<_> = price_rows
                .iter()
                .filter(|r| r.get::<Uuid, _>("hotel_id") == hotel_id)
                .collect();

            let gother_room_type: Option<String> = hotel_price_rows
                .iter()
                .find(|r| r.get::<String, _>("source") == GOTHER)
                .map(|r| r.get("room_type"));
            let gother_cancellation: Option<Option<String>> = hotel_price_rows
                .iter()
                .find(|r| r.get::<String, _>("source") == GOTHER)
                .map(|r| r.get("cancellation"));

            let hotel_prices: Vec<PriceEntry> = hotel_price_rows
                .iter()
                .map(|r| {
                    let source: String = r.get("source");
                    let room_type: String = r.get("room_type");
                    let cancellation: Option<String> = r.get("cancellation");

                    let mismatch_warning = if source != GOTHER {
                        let mut mismatches = Vec::new();
                        if let Some(g_room) = &gother_room_type {
                            if g_room != &room_type {
                                mismatches.push("room type differs from Gother");
                            }
                        }
                        if let Some(g_cancel) = &gother_cancellation {
                            if g_cancel != &cancellation {
                                mismatches.push("cancellation policy differs from Gother");
                            }
                        }
                        if mismatches.is_empty() {
                            None
                        } else {
                            Some(mismatches.join("; "))
                        }
                    } else {
                        None
                    };

                    PriceEntry {
                        source: source.clone(),
                        room_type,
                        price_thb: r.get("price_thb"),
                        original_price: r.get("original_price"),
                        currency: r.get("currency"),
                        meal_plan: r.get("meal_plan"),
                        cancellation,
                        source_url: r.get("source_url"),
                        scraped_at: r.get("scraped_at"),
                        los_nights: r.get("los_nights"),
                        who_id: r.get("who_id"),
                        is_direct_contract: is_direct_contract(&source),
                        mismatch_warning,
                    }
                })
                .collect();

            // Find best price and gother price
            let best = hotel_prices.iter().min_by(|a, b| {
                a.price_thb.partial_cmp(&b.price_thb).unwrap()
            });
            let gother = hotel_prices.iter().find(|p| p.source == GOTHER);

            let best_price = best.map(|b| b.price_thb);
            let best_source = best.map(|b| b.source.clone());
            let gother_price = gother.map(|g| g.price_thb);
            let price_difference = match (best_price, gother_price) {
                (Some(bp), Some(gp)) => Some(gp - bp),
                _ => None,
            };
            // (gother - best) / best * 100. None (never 0) when either
            // price is missing — REQ-001 F-027.
            let price_diff_percent = match (best_price, gother_price) {
                (Some(bp), Some(gp)) if bp != 0.0 => Some((gp - bp) / bp * 100.0),
                _ => None,
            };

            if status == HotelScrapeStatus::Success {
                successful_count += 1;
                if let Some(bp) = best_price {
                    total_best_price += bp;
                }
            } else if status == HotelScrapeStatus::Failed {
                failed_count += 1;
            }

            results.push(HotelPriceComparison {
                hotel: HotelInfo {
                    id: hotel_id,
                    name: hotel_row.get("hotel_name"),
                    city: hotel_row.get("city"),
                    country: hotel_row.get("country"),
                    hid: hotel_row.get("hid"),
                },
                status,
                error_message: hotel_row.get("error_message"),
                prices: hotel_prices,
                best_source,
                best_price,
                gother_price,
                price_difference,
                price_diff_percent,
            });
        }

        let avg_best_price = if successful_count > 0 {
            Some(total_best_price / successful_count as f64)
        } else {
            None
        };

        Ok(ScrapeResultsResponse {
            job,
            summary: ScrapeResultsSummary {
                total_hotels: results.len() as i32,
                successful: successful_count,
                failed: failed_count,
                avg_best_price,
            },
            results,
        })
    }
}
