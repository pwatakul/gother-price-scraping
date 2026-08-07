//! Scrape Result Model

use crate::models::scrape_job::{Device, LoginState, ScrapeMethod};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Scrape Result - price data from an OTA
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScrapeResult {
    pub id: Uuid,
    pub scrape_job_id: Uuid,
    pub hotel_id: Uuid,
    pub source: String,
    pub room_type: String,
    pub price_thb: f64,
    pub original_price: Option<f64>,
    pub currency: Option<String>,
    pub meal_plan: Option<String>,
    pub cancellation: Option<String>,
    pub source_url: Option<String>,
    pub scraped_at: DateTime<Utc>,
    pub los_nights: i32,
    pub device: Device,
    pub login_state: LoginState,
    pub who_id: Option<String>,
    pub via_method: ScrapeMethod,
}

/// Hotel scrape status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "hotel_scrape_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HotelScrapeStatus {
    Pending,
    Processing,
    Success,
    Failed,
}

impl std::fmt::Display for HotelScrapeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotelScrapeStatus::Pending => write!(f, "pending"),
            HotelScrapeStatus::Processing => write!(f, "processing"),
            HotelScrapeStatus::Success => write!(f, "success"),
            HotelScrapeStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Scrape Hotel Status - tracks success/failure per hotel per job
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScrapeHotelStatus {
    pub id: Uuid,
    pub scrape_job_id: Uuid,
    pub hotel_id: Uuid,
    pub status: HotelScrapeStatus,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price data for a single hotel with all sources
#[derive(Debug, Clone, Serialize)]
pub struct HotelPriceComparison {
    pub hotel: HotelInfo,
    pub status: HotelScrapeStatus,
    pub error_message: Option<String>,
    pub prices: Vec<PriceEntry>,
    pub best_source: Option<String>,
    pub best_price: Option<f64>,
    pub gother_price: Option<f64>,
    pub price_difference: Option<f64>,
    /// `(gother_price - best_price) / best_price * 100`. None when either
    /// price is missing (REQ-001 F-027 — never render as 0).
    pub price_diff_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotelInfo {
    pub id: Uuid,
    pub name: String,
    pub city: String,
    pub country: String,
    pub hid: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceEntry {
    pub source: String,
    pub room_type: String,
    pub price_thb: f64,
    pub original_price: Option<f64>,
    pub currency: Option<String>,
    pub meal_plan: Option<String>,
    pub cancellation: Option<String>,
    pub source_url: Option<String>,
    pub scraped_at: DateTime<Utc>,
    pub los_nights: i32,
    /// WHO ID for Gother-sourced rates (REQ-001 F-025). Always None for
    /// non-Gother sources; None for Gother rows too until the upstream API
    /// is confirmed to return one.
    pub who_id: Option<String>,
    /// Wink/HyperGuest direct-contract rate flag (REQ-001 F-026).
    pub is_direct_contract: bool,
    /// Set when this entry's room_type/meal_plan/cancellation differs from
    /// Gother's entry for the same hotel (REQ-001 F-011 ⚠️ badge).
    pub mismatch_warning: Option<String>,
}

/// Complete scrape results response
#[derive(Debug, Clone, Serialize)]
pub struct ScrapeResultsResponse {
    pub job: ScrapeJobInfo,
    pub summary: ScrapeResultsSummary,
    pub results: Vec<HotelPriceComparison>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrapeJobInfo {
    pub id: Uuid,
    pub checkin_date: String,
    pub checkout_date: String,
    pub rooms: i32,
    pub adults: i32,
    pub status: String,
    pub method: ScrapeMethod,
    pub device: Device,
    pub login_state: LoginState,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrapeResultsSummary {
    pub total_hotels: i32,
    pub successful: i32,
    pub failed: i32,
    pub avg_best_price: Option<f64>,
}
