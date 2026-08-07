//! Price History Model (REQ-002 / REQ-005)

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
    pub avg_price_thb: f64,
    pub min_price_thb: f64,
    pub max_price_thb: f64,
    pub sample_count: i64,
}
