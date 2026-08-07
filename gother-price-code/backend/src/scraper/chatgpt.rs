//! ChatGPT Scraper (REQ-001 F-020 — Method 1, bonus)
//!
//! Asks OpenAI's chat completions API for current hotel prices using a
//! strict JSON schema response, then normalizes the result onto the same
//! named-provider set (agoda/trip) used by the SerpAPI scraper.
//!
//! Unlike `MockScraper`, a missing `OPENAI_API_KEY` does not fall back to
//! fabricated prices here — it silently skips (same guard style as
//! `GotherScraper`), because inventing prices under the "ChatGPT" method
//! label would misrepresent Method 1 as working when it isn't.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use super::providers::{AGODA, TRIP};
use super::{ScrapeParams, ScrapeResult, Scraper};

pub struct ChatGptScraper {
    api_key: String,
    model: String,
    client: Client,
}

impl ChatGptScraper {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: Client::new(),
        }
    }

    /// None when no API key is configured — caller should skip this scraper
    /// entirely rather than treat it as a scrape failure.
    pub fn from_config(api_key: &Option<String>, model: &str) -> Option<Self> {
        let key = api_key.as_ref()?;
        if key.trim().is_empty() {
            return None;
        }
        Some(Self::new(key, model))
    }
}

/// Strict JSON schema for the model's response.
#[derive(Debug, Deserialize)]
struct ChatGptHotelPriceJson {
    #[serde(default)]
    rates: Vec<ChatGptRate>,
}

#[derive(Debug, Deserialize)]
struct ChatGptRate {
    provider: String,
    room_type: String,
    price_thb: f64,
    #[serde(default)]
    meal_plan: Option<String>,
    #[serde(default)]
    cancellation: Option<String>,
    #[serde(default)]
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    content: String,
}

#[async_trait]
impl Scraper for ChatGptScraper {
    fn name(&self) -> &'static str {
        "chatgpt"
    }

    async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>> {
        let prompt = format!(
            "What are the current per-night rates for \"{}\" in {}, {} for check-in {} \
             and check-out {}, {} room(s), {} adult(s)? Only report rates you have current \
             knowledge of from Agoda or Trip.com. Respond with JSON only, matching this \
             schema exactly: {{\"rates\": [{{\"provider\": \"agoda\"|\"trip\", \"room_type\": \
             string, \"price_thb\": number, \"meal_plan\": string|null, \"cancellation\": \
             string|null, \"source_url\": string|null}}]}}. If you don't have a confident \
             rate for a provider, omit it — do not guess a number.",
            params.hotel_name,
            params.city,
            params.country,
            params.checkin_date,
            params.checkout_date,
            params.rooms,
            params.adults,
        );

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "response_format": {"type": "json_object"},
                "temperature": 0,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("OpenAI request failed: {}", response.status());
        }

        let completion: ChatCompletionResponse = response.json().await?;
        let Some(choice) = completion.choices.into_iter().next() else {
            anyhow::bail!("OpenAI returned no choices");
        };

        let parsed: ChatGptHotelPriceJson = serde_json::from_str(&choice.message.content)?;

        let mut results = Vec::new();
        for rate in parsed.rates {
            let normalized_provider = match rate.provider.to_lowercase().as_str() {
                p if p.contains("agoda") => AGODA,
                p if p.contains("trip") => TRIP,
                _ => continue, // not a named provider — drop it
            };

            results.push(ScrapeResult {
                source: normalized_provider.to_string(),
                room_type: rate.room_type,
                price_thb: rate.price_thb,
                original_price: Some(rate.price_thb),
                currency: Some("THB".to_string()),
                meal_plan: rate.meal_plan,
                cancellation: rate.cancellation,
                source_url: rate.source_url,
                who_id: None,
            });
        }

        Ok(results)
    }
}
