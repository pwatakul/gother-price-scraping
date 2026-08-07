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
    /// Gother-side hotel ID from the master hotel list (HID column). None
    /// for hotels created via the plain hotel_name/city/country import.
    pub hid: Option<i64>,
    pub slug: Option<String>,
    pub update_url: Option<String>,
    pub supplier_type: Option<String>,
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

/// One row of the global "All Hotels" directory (REQ-007) — a hotel plus
/// which group(s) it belongs to and its most recent price, independent
/// of any single group.
#[derive(Debug, Clone, Serialize)]
pub struct HotelWithGroupsAndPrice {
    pub id: Uuid,
    pub name: String,
    pub city: String,
    pub country: String,
    pub hid: Option<i64>,
    pub slug: Option<String>,
    pub supplier_type: Option<String>,
    pub group_names: Vec<String>,
    pub last_price_thb: Option<f64>,
    pub last_price_source: Option<String>,
    pub last_scraped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct HotelListResponse {
    pub hotels: Vec<HotelWithGroupsAndPrice>,
    pub total: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct HotelListQuery {
    pub country: Option<String>,
    pub city: Option<String>,
    pub q: Option<String>,
    #[serde(default = "default_hotel_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_hotel_limit() -> i64 {
    50
}

/// Full detail for one hotel: identity + group memberships + recent
/// price history trend, for the per-hotel tracking page.
#[derive(Debug, Serialize)]
pub struct HotelDetail {
    pub hotel: Hotel,
    pub group_names: Vec<String>,
    pub trend: Vec<crate::models::PriceTrendPoint>,
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

/// One row of the real master hotel list (hotel-list-2200.csv shape):
/// `No, HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH`.
/// `No` and `SEARCH` are not persisted.
#[derive(Debug, Clone, Deserialize)]
pub struct MasterHotelImportRow {
    pub hid: i64,
    pub hotel_name: String,
    pub update_url: Option<String>,
    pub slug: Option<String>,
    pub supplier_type: Option<String>,
    pub country: String,
}

/// Per-hotel search-parameter override for a scrape job, read from an
/// optional separate sheet. `None` fields fall back to job-level defaults
/// (see `excel::job_defaults::merge_job_params`).
#[derive(Debug, Clone, Deserialize)]
pub struct JobHotelParamOverride {
    /// Either `hid` or `hotel_name` must be present to key the override.
    pub hid: Option<i64>,
    pub hotel_name: Option<String>,
    pub checkin_date: Option<chrono::NaiveDate>,
    pub checkout_date: Option<chrono::NaiveDate>,
    pub rooms: Option<i32>,
    pub adults: Option<i32>,
    pub currency: Option<String>,
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

    /// Wink is domestic-only (REQ-001-v1.2 F-022). Country values in the
    /// master hotel list are lowercase free text (e.g. "thailand").
    pub fn is_domestic(&self) -> bool {
        self.country.trim().eq_ignore_ascii_case("thailand")
    }
}
