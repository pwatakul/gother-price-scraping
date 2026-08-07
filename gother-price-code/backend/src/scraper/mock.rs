//! Mock Scraper
//!
//! A mock scraper for testing without real API keys.
//! Returns realistic-looking fake data.

use async_trait::async_trait;
use rand::Rng;

use super::{ScrapeParams, ScrapeResult, Scraper};

/// Mock scraper for testing
pub struct MockScraper;

impl MockScraper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for MockScraper {
    fn name(&self) -> &'static str {
        "mock"
    }

    async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>> {
        // Simulate API delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let mut rng = rand::thread_rng();
        let mut results = Vec::new();

        // Generate mock prices for the named providers only (agoda/trip/
        // gother). Wink is deliberately never mocked here — it has no real
        // scraper, and REQ-001 F-027 requires it render blank, not fake data.
        let sources = vec![
            ("agoda", "https://www.agoda.com"),
            ("trip", "https://www.trip.com"),
            ("gother", "https://www.gother.com"),
        ];

        let room_types = vec![
            "Deluxe Room",
            "Superior Room",
            "Premier Room",
            "Junior Suite",
        ];

        let meal_plans = vec!["Room Only", "Breakfast Included", "Half Board"];

        // Base price varies by hotel "tier"
        let base_price = if params.hotel_name.to_lowercase().contains("four seasons")
            || params.hotel_name.to_lowercase().contains("mandarin")
            || params.hotel_name.to_lowercase().contains("peninsula")
        {
            rng.gen_range(12000.0..18000.0)
        } else if params.hotel_name.to_lowercase().contains("shangri")
            || params.hotel_name.to_lowercase().contains("dusit")
        {
            rng.gen_range(6000.0..10000.0)
        } else {
            rng.gen_range(3000.0..6000.0)
        };

        for (source, url) in &sources {
            // Each source has slightly different prices
            let price_variance: f64 = rng.gen_range(-500.0..500.0);
            let price = (base_price + price_variance).max(1000.0);

            let room_type = room_types[rng.gen_range(0..room_types.len())];
            let meal_plan = meal_plans[rng.gen_range(0..meal_plans.len())];

            results.push(ScrapeResult {
                source: source.to_string(),
                room_type: room_type.to_string(),
                price_thb: (price * 100.0).round() / 100.0,
                original_price: Some(price),
                currency: Some("THB".to_string()),
                meal_plan: Some(meal_plan.to_string()),
                cancellation: Some("Free cancellation until 24h before".to_string()),
                source_url: Some(format!(
                    "{}/hotel/{}",
                    url,
                    params.hotel_name.replace(' ', "-").to_lowercase()
                )),
                who_id: None,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::scrape_job::{Device, LoginState};
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_mock_scraper() {
        let scraper = MockScraper::new();
        let params = ScrapeParams {
            hotel_name: "Test Hotel".to_string(),
            city: "Bangkok".to_string(),
            country: "Thailand".to_string(),
            checkin_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            checkout_date: NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            rooms: 1,
            adults: 2,
            los_nights: 1,
            device: Device::Desktop,
            login_state: LoginState::Public,
        };

        let results = scraper.scrape(&params).await.unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.source == "agoda"));
        assert!(results.iter().any(|r| r.source == "trip"));
        assert!(results.iter().any(|r| r.source == "gother"));
        assert!(!results.iter().any(|r| r.source == "wink"));
    }
}
