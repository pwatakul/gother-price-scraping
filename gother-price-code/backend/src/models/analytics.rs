//! Analytics response types (REQ-003), backed by the materialized views
//! created in migration 016.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MarketPositionRow {
    pub hotel_id: Uuid,
    pub source: String,
    pub room_type: String,
    pub price_thb: f64,
    pub checkin_date: NaiveDate,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketOverview {
    pub total_hotels: i64,
    pub gother_cheapest_pct: f64,
    pub avg_gap_thb: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketPositionEntry {
    pub hotel_id: Uuid,
    pub hotel_name: String,
    pub gother_price: Option<f64>,
    pub best_price: Option<f64>,
    pub best_source: Option<String>,
    pub gap_thb: Option<f64>,
    pub gap_pct: Option<f64>,
    pub is_winning: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapCell {
    pub hotel_id: Uuid,
    pub hotel_name: String,
    pub source: String,
    pub price_thb: Option<f64>,
    pub gap_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct WinRateRow {
    pub hotel_id: Uuid,
    pub days_won: i64,
    pub days_total: i64,
    pub win_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct BookingWindowRow {
    pub source: String,
    pub days_in_advance: i32,
    pub avg_price_thb: f64,
    pub min_price_thb: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ParityViolationRow {
    pub hotel_id: Uuid,
    pub hotel_name: String,
    pub gother_price: f64,
    pub best_ota_price: f64,
    pub gap_pct: f64,
}
