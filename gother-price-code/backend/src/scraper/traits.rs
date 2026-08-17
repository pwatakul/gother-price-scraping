//! Scraper Traits
//!
//! Common types and traits for scrapers.

use crate::models::scrape_job::{Device, LoginState};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Parameters for scraping hotel prices
#[derive(Debug, Clone)]
pub struct ScrapeParams {
    pub hotel_name: String,
    pub city: String,
    pub country: String,
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i32,
    pub adults: i32,
    pub los_nights: i32,
    pub device: Device,
    pub login_state: LoginState,
}

/// Result from a price scrape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeResult {
    pub source: String,
    pub room_type: String,
    pub price_thb: f64,
    pub original_price: Option<f64>,
    pub currency: Option<String>,
    pub meal_plan: Option<String>,
    pub cancellation: Option<String>,
    pub source_url: Option<String>,
    /// WHO ID, only ever populated by the Gother scraper (REQ-001 F-025).
    #[serde(default)]
    pub who_id: Option<String>,
    /// Which scraper actually produced this row ("serpapi"/"gemini"/…).
    /// Scrapers leave this blank; the registry loop stamps it from the
    /// factory name so provenance cannot disagree with reality (ADR-011).
    /// `serde(default)` keeps Redis entries cached before this field
    /// deserializable.
    #[serde(default)]
    pub via_method: String,
}

/// Trait for price scrapers
#[async_trait]
pub trait Scraper: Send + Sync {
    /// Get the scraper name
    fn name(&self) -> &'static str;

    /// Scrape prices for a hotel
    async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>>;
}
