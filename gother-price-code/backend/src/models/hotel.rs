//! Hotel Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Hotel - individual hotel record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Hotel {
    pub id: Uuid,
    pub name: String,
    pub city: String,
    pub country: String,
    pub normalized_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Hotel with last price info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelWithPrice {
    pub id: Uuid,
    pub name: String,
    pub city: String,
    pub country: String,
    pub last_price_thb: Option<f64>,
    pub last_price_source: Option<String>,
    pub last_scraped_at: Option<DateTime<Utc>>,
}

/// Request to create a new hotel
#[derive(Debug, Deserialize)]
pub struct CreateHotelRequest {
    pub name: String,
    pub city: String,
    pub country: String,
}

/// Hotel data from Excel import
#[derive(Debug, Clone, Deserialize)]
pub struct HotelImportData {
    pub hotel_name: String,
    pub city: String,
    pub country: String,
}

impl Hotel {
    /// Generate normalized name for matching across OTAs
    pub fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .trim()
            .replace("hotel", "")
            .replace("resort", "")
            .replace("&", "and")
            .replace("  ", " ")
            .trim()
            .to_string()
    }
}
