//! Scrape Result Model

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
}

#[derive(Debug, Clone, Serialize)]
pub struct HotelInfo {
    pub id: Uuid,
    pub name: String,
    pub city: String,
    pub country: String,
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
