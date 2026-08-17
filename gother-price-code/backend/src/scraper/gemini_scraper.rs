//! Gemini Scraper — a real price-scraping method (`ScrapeMethod::Gemini`).
//!
//! Asks Gemini for current hotel rates via `GeminiClient::generate`,
//! requesting strict JSON, then normalizes the result onto the same
//! named-provider set (agoda/trip) used by the SerpAPI/ChatGPT scrapers.
//!
//! Same no-fabrication rule as `ChatGptScraper`: a missing `GEMINI_API_KEY`
//! causes this scraper to be skipped entirely, never replaced with mock
//! data — the "Gemini" method label must mean the price genuinely came
//! from Gemini, or nothing at all.

use async_trait::async_trait;
use serde::Deserialize;

use super::providers;
use super::{ScrapeParams, ScrapeResult, Scraper};
use crate::ai::GeminiClient;

pub struct GeminiScraper {
    client: GeminiClient,
}

impl GeminiScraper {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: GeminiClient::new(api_key, model),
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

#[derive(Debug, Deserialize)]
struct GeminiHotelPriceJson {
    #[serde(default)]
    rates: Vec<GeminiRate>,
}

#[derive(Debug, Deserialize)]
struct GeminiRate {
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

#[async_trait]
impl Scraper for GeminiScraper {
    fn name(&self) -> &'static str {
        "gemini"
    }

    async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>> {
        let prompt = format!(
            "What are the current per-night rates for \"{}\" in {}, {} for check-in {} \
             and check-out {}, {} room(s), {} adult(s)? Only report rates you have current \
             knowledge of from Agoda or Trip.com. Respond with JSON only (no markdown code \
             fences, no commentary), matching this schema exactly: \
             {{\"rates\": [{{\"provider\": \"agoda\"|\"trip\", \"room_type\": string, \
             \"price_thb\": number, \"meal_plan\": string|null, \"cancellation\": string|null, \
             \"source_url\": string|null}}]}}. If you don't have a confident rate for a \
             provider, omit it — do not guess a number.",
            params.hotel_name,
            params.city,
            params.country,
            params.checkin_date,
            params.checkout_date,
            params.rooms,
            params.adults,
        );

        let raw = self.client.generate_raw(&prompt).await?;
        let json_text = strip_markdown_fence(&raw);
        let parsed: GeminiHotelPriceJson = serde_json::from_str(json_text)?;

        let mut results = Vec::new();
        for rate in parsed.rates {
            // Same allowlist as every other scraper (ADR-009). Deliberately
            // does not pass hotel details, so a model naming the hotel's own
            // site cannot be promoted to a `direct` rate on its say-so.
            let Some(normalized_provider) =
                providers::normalize_source(&rate.provider, "", "", "")
            else {
                continue; // not an allowlisted provider — drop it
            };

            results.push(ScrapeResult {
                source: normalized_provider,
                room_type: rate.room_type,
                price_thb: rate.price_thb,
                original_price: Some(rate.price_thb),
                currency: Some("THB".to_string()),
                meal_plan: rate.meal_plan,
                cancellation: rate.cancellation,
                source_url: rate.source_url,
                who_id: None,
                // Stamped by the registry loop from the factory name (ADR-011).
                via_method: String::new(),
            });
        }

        Ok(results)
    }
}

/// Gemini (unlike OpenAI's `response_format: json_object`) has no
/// guaranteed-JSON mode on all models — strip a ```json ... ``` fence if
/// the model wrapped its output in one despite being asked not to.
fn strip_markdown_fence(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.trim().strip_suffix("```").unwrap_or(rest).trim()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.trim().strip_suffix("```").unwrap_or(rest).trim()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_json_fence() {
        assert_eq!(strip_markdown_fence("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(strip_markdown_fence("{\"a\":1}"), "{\"a\":1}");
        assert_eq!(strip_markdown_fence("```\n{\"a\":1}\n```"), "{\"a\":1}");
    }
}
