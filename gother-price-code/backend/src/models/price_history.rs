//! Price History Model (REQ-002 / REQ-005)

use crate::models::scrape_job::Device;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A single historical price point — the long-term, queryable counterpart
/// to `scrape_results` (which is scoped to one job).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HotelPriceHistory {
    pub id: Uuid,
    pub hotel_id: Uuid,
    pub source: String,
    pub room_type: String,
    pub price_thb: f64,
    pub original_price: Option<f64>,
    pub currency: Option<String>,
    pub exchange_rate_id: Uuid,
    pub meal_plan: Option<String>,
    pub cancellation: Option<String>,
    pub source_url: Option<String>,
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i16,
    pub adults: i16,
    pub device: Device,
    /// Which scraper produced this row — "serpapi", "gemini", "gother",
    /// "mock". Distinguishes a real scrape from an AI estimate used to
    /// fill a blank; see ADR-011.
    pub via_method: String,
    pub scrape_job_id: Option<Uuid>,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CurrencyExchangeRate {
    pub id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub rate_date: NaiveDate,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Filters for `GET /price-history` (REQ-002 F-007).
#[derive(Debug, Deserialize, Default)]
pub struct PriceHistoryQuery {
    pub hotel_id: Option<Uuid>,
    /// Filter to every hotel that belongs to this group (used for the
    /// per-group "export full price history" button — not a per-job
    /// export, which already exists via GET /scrape-jobs/:id/export).
    pub hotel_group_id: Option<Uuid>,
    pub source: Option<String>,
    pub device: Option<Device>,
    pub checkin_from: Option<NaiveDate>,
    pub checkin_to: Option<NaiveDate>,
    pub scraped_from: Option<DateTime<Utc>>,
    pub scraped_to: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

/// `GET /price-history` response — paginated, with a total count so the
/// frontend can render numbered pages (same pattern as `HotelListResponse`).
#[derive(Debug, Serialize)]
pub struct PriceHistoryListResponse {
    pub rows: Vec<HotelPriceHistory>,
    pub total: i64,
}

/// One point on a per-hotel trend line (REQ-002 F-008 / REQ-003 F-002),
/// backed by `mv_hotel_daily_avg_price`.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PriceTrendPoint {
    pub source: String,
    pub day: DateTime<Utc>,
    /// Days between the scrape and check-in. Carried so a chart can state
    /// which booking window it is showing (ADR-013).
    pub days_in_advance: i32,
    pub avg_price_thb: f64,
    pub min_price_thb: f64,
    pub max_price_thb: f64,
    pub sample_count: i64,
}

/// A booking window present in a hotel's history, with how much data
/// backs it — the UI builds its window selector from these rather than a
/// hardcoded list, because manual runs create arbitrary offsets.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TrendWindow {
    pub days_in_advance: i32,
    pub sample_count: i64,
}
