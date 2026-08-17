//! Gother API Scraper
//!
//! Scrapes hotel prices from Gother's internal API.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::{ScrapeParams, ScrapeResult, Scraper};
use crate::normalizer;

/// Gother API scraper
pub struct GotherScraper {
    api_url: String,
    api_key: String,
    client: Client,
}

impl GotherScraper {
    pub fn new(api_url: &str, api_key: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GotherResponse {
    success: bool,
    data: Option<GotherData>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GotherData {
    hotel: Option<GotherHotel>,
    rooms: Vec<GotherRoom>,
}

#[derive(Debug, Deserialize)]
struct GotherHotel {
    name: String,
    #[allow(dead_code)]
    id: String,
}

#[derive(Debug, Deserialize)]
struct GotherRoom {
    room_type: String,
    price: f64,
    currency: String,
    meal_plan: Option<String>,
    cancellation_policy: Option<String>,
    #[allow(dead_code)]
    available: bool,
    /// WHO ID for this rate (REQ-001 F-025). Not confirmed whether Gother's
    /// API actually returns this field today — deserialize as optional and
    /// never fabricate a value if it's absent.
    #[serde(default)]
    who_id: Option<String>,
}

#[async_trait]
impl Scraper for GotherScraper {
    fn name(&self) -> &'static str {
        "gother"
    }

    async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>> {
        let url = format!("{}/api/hotels/search", self.api_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "hotel_name": params.hotel_name,
                "city": params.city,
                "country": params.country,
                "checkin_date": params.checkin_date.to_string(),
                "checkout_date": params.checkout_date.to_string(),
                "rooms": params.rooms,
                "adults": params.adults,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Gother API request failed: {}", response.status());
        }

        let data: GotherResponse = response.json().await?;

        if !data.success {
            anyhow::bail!(
                "Gother API error: {}",
                data.error.unwrap_or_else(|| "Unknown error".to_string())
            );
        }

        let Some(hotel_data) = data.data else {
            anyhow::bail!("No hotel data in response");
        };

        if hotel_data.rooms.is_empty() {
            anyhow::bail!("No rooms available");
        }

        let mut results = Vec::new();

        for room in hotel_data.rooms {
            // Convert price to THB if needed
            let price_thb = normalizer::convert_to_thb(room.price, &room.currency);

            // Normalize room type and meal plan
            let normalized_room = normalizer::normalize_room_type(&room.room_type);
            let normalized_meal = room
                .meal_plan
                .as_ref()
                .map(|m| normalizer::normalize_meal_plan(m));

            results.push(ScrapeResult {
                source: super::providers::GOTHER.to_string(),
                room_type: normalized_room,
                price_thb,
                original_price: Some(room.price),
                currency: Some(room.currency),
                meal_plan: normalized_meal,
                cancellation: room.cancellation_policy,
                source_url: Some(format!(
                    "{}/hotels/{}",
                    self.api_url,
                    hotel_data
                        .hotel
                        .as_ref()
                        .map(|h| h.name.clone())
                        .unwrap_or_default()
                )),
                who_id: room.who_id.clone(),
                // Stamped by the registry loop from the factory name (ADR-011).
                via_method: String::new(),
            });
        }

        Ok(results)
    }
}
