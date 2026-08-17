//! Analytics response types (REQ-003), backed by the materialized views
//! created in migration 016.

use crate::models::scrape_job::Device;
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
    /// The stay every provider on this row was compared on. Stated in the
    /// UI so a reader can see the comparison is like-for-like (ADR-013).
    pub checkin_date: chrono::NaiveDate,
    pub gother_price: Option<f64>,
    pub best_price: Option<f64>,
    pub best_source: Option<String>,
    pub gap_thb: Option<f64>,
    pub gap_pct: Option<f64>,
    pub is_winning: bool,
    /// Cheapest provider for this stay. Unlike the Gother columns above,
    /// these are populated today.
    pub cheapest_source: Option<String>,
    pub cheapest_price: Option<f64>,
    pub provider_count: i64,
    /// Dearest vs cheapest, as a % of the cheapest.
    pub spread_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapCell {
    pub hotel_id: Uuid,
    pub hotel_name: String,
    pub source: String,
    /// The stay this cell belongs to — all cells in a row share it.
    pub checkin_date: chrono::NaiveDate,
    pub price_thb: Option<f64>,
    /// Versus Gother; null until Gother has a data source.
    pub gap_pct: Option<f64>,
    /// Cheapest provider for this hotel's stay — the winner highlight.
    pub is_cheapest: bool,
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
    pub device: Device,
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

/// How one provider compares against the cheapest competitor for the same
/// hotel and date. Deliberately Gother-independent: every existing metric
/// is defined relative to Gother's price, which has no data source yet, so
/// this answers "who is cheapest, and by how much" from scraped data alone.
///
/// `median_premium_pct` is a median rather than a mean on purpose — a
/// single bad scrape (a five-star resort returned at THB 52) drags an
/// average into nonsense, and price data has real outliers.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProviderBenchmarkRow {
    pub source: String,
    /// Stay-level comparisons this provider took part in (hotel +
    /// check-in date pairs), so thin coverage is visible next to a high
    /// win rate.
    pub quotes_compared: i64,
    /// Hotels where this provider quoted a price at all — coverage differs
    /// per provider, so a high cheapest_pct on thin coverage means less.
    pub hotels_covered: i64,
    pub times_cheapest: i64,
    pub cheapest_pct: f64,
    /// Median % above the cheapest provider for the same hotel. 0 means it
    /// is typically the cheapest.
    pub median_premium_pct: f64,
}
