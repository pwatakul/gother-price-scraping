//! SerpAPI Scraper
//!
//! Scrapes hotel prices using SerpAPI's Google Hotels API.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::providers::{AGODA, TRIP};
use super::{ScrapeParams, ScrapeResult, Scraper};
use crate::normalizer;

/// SerpAPI Google Hotels scraper
pub struct SerpApiScraper {
    api_key: String,
    client: Client,
    base_url: String,
}

impl SerpApiScraper {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
            base_url: "https://serpapi.com/search".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SerpApiResponse {
    #[serde(default)]
    properties: Vec<SerpApiProperty>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerpApiProperty {
    name: Option<String>,
    #[serde(default)]
    prices: Vec<SerpApiPrice>,
    link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerpApiPrice {
    source: Option<String>,
    rate_per_night: Option<SerpApiRate>,
    #[serde(rename = "type")]
    room_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerpApiRate {
    lowest: Option<String>,
    extracted_lowest: Option<f64>,
}

#[async_trait]
impl Scraper for SerpApiScraper {
    fn name(&self) -> &'static str {
        "serpapi"
    }

    async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>> {
        // Build search query
        let query = format!(
            "{} {} {}",
            params.hotel_name, params.city, params.country
        );

        // Make API request
        let response = self
            .client
            .get(&self.base_url)
            .query(&[
                ("engine", "google_hotels"),
                ("q", &query),
                ("check_in_date", &params.checkin_date.to_string()),
                ("check_out_date", &params.checkout_date.to_string()),
                ("adults", &params.adults.to_string()),
                ("rooms", &params.rooms.to_string()),
                ("currency", "THB"),
                ("gl", "th"),
                ("hl", "en"),
                ("api_key", &self.api_key),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("SerpAPI request failed: {}", response.status());
        }

        let data: SerpApiResponse = response.json().await?;

        if let Some(error) = data.error {
            anyhow::bail!("SerpAPI error: {}", error);
        }

        // Find matching property
        let normalized_search = params.hotel_name.to_lowercase();
        let property = data
            .properties
            .iter()
            .find(|p| {
                p.name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&normalized_search))
                    .unwrap_or(false)
            })
            .or_else(|| data.properties.first());

        let Some(property) = property else {
            anyhow::bail!("Hotel not found in search results");
        };

        // Extract prices
        let mut results = Vec::new();

        for price in &property.prices {
            let Some(rate) = &price.rate_per_night else {
                continue;
            };

            let Some(price_value) = rate.extracted_lowest else {
                continue;
            };

            let source = price
                .source
                .as_ref()
                .map(|s| s.to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());

            let room_type = price
                .room_type
                .clone()
                .unwrap_or_else(|| "Standard Room".to_string());

            // Normalize room type
            let normalized_room = normalizer::normalize_room_type(&room_type);

            // Only Agoda/Trip are named providers per REQ-001-v1.2 F-022;
            // drop booking/expedia/hotels.com/official/anything else so
            // comparisons stay apples-to-apples (never fabricate a row).
            let Some(normalized_source) = normalize_source_name(&source) else {
                continue;
            };

            results.push(ScrapeResult {
                source: normalized_source,
                room_type: normalized_room,
                price_thb: price_value,
                original_price: Some(price_value),
                currency: Some("THB".to_string()),
                meal_plan: None, // SerpAPI doesn't always provide this
                cancellation: None,
                source_url: property.link.clone(),
                who_id: None,
            });
        }

        // An empty result after filtering is a legitimate "no matching
        // named-provider rate" outcome (REQ-001 F-027 blank case), not a
        // scrape failure — only bail if SerpAPI returned nothing at all.
        if results.is_empty() && property.prices.is_empty() {
            anyhow::bail!("No prices found");
        }

        Ok(results)
    }
}

/// Normalize OTA source names onto the brief's named-provider set
/// (Gother/Agoda/Trip/Wink). Returns None for anything else so the caller
/// can drop it.
fn normalize_source_name(source: &str) -> Option<String> {
    let lower = source.to_lowercase();

    if lower.contains("agoda") {
        Some(AGODA.to_string())
    } else if lower.contains("trip.com") || lower.contains("ctrip") {
        Some(TRIP.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_named_providers_only() {
        assert_eq!(normalize_source_name("Agoda.com"), Some(AGODA.to_string()));
        assert_eq!(normalize_source_name("Trip.com"), Some(TRIP.to_string()));
        assert_eq!(normalize_source_name("Ctrip"), Some(TRIP.to_string()));
    }

    #[test]
    fn drops_non_named_providers() {
        assert_eq!(normalize_source_name("Booking.com"), None);
        assert_eq!(normalize_source_name("Expedia"), None);
        assert_eq!(normalize_source_name("Hotels.com"), None);
        assert_eq!(normalize_source_name("Official Site"), None);
    }
}
