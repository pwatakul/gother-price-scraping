//! Gemini AI Client
//!
//! Integration with Google's Gemini AI for data enhancement.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::prompts;

/// Gemini AI client
pub struct GeminiClient {
    api_key: String,
    model: String,
    client: Client,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: Client::new(),
        }
    }

    /// Normalize room type using AI
    pub async fn normalize_room_type(&self, room_type: &str) -> Result<String> {
        let prompt = prompts::normalize_room_type_prompt(room_type);
        let response = self.generate(&prompt).await?;
        Ok(response.trim().to_string())
    }

    /// Extract hotel information from text
    pub async fn extract_hotel_info(&self, text: &str) -> Result<HotelExtraction> {
        let prompt = prompts::extract_hotel_info_prompt(text);
        let response = self.generate(&prompt).await?;
        
        // Parse JSON response
        let extraction: HotelExtraction = serde_json::from_str(&response)?;
        Ok(extraction)
    }

    /// Compare room types for apple-to-apple comparison
    pub async fn compare_room_types(&self, rooms: &[RoomInfo]) -> Result<Vec<RoomComparison>> {
        let prompt = prompts::compare_rooms_prompt(rooms);
        let response = self.generate(&prompt).await?;
        
        // Parse JSON response
        let comparisons: Vec<RoomComparison> = serde_json::from_str(&response)?;
        Ok(comparisons)
    }

    /// Generate raw text from an arbitrary prompt (used by `GeminiScraper`).
    pub async fn generate_raw(&self, prompt: &str) -> Result<String> {
        self.generate(prompt).await
    }

    /// Generate text using Gemini
    async fn generate(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.1,
                max_output_tokens: 1024,
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Gemini API error: {}", error_text);
        }

        let data: GeminiResponse = response.json().await?;
        
        let text = data
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        Ok(text)
    }
}

/// Gemini API request structure
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

/// Gemini API response structure
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Deserialize)]
struct ContentResponse {
    parts: Vec<Part>,
}

/// Extracted hotel information
#[derive(Debug, Deserialize)]
pub struct HotelExtraction {
    pub hotel_name: String,
    pub city: String,
    pub country: String,
    pub room_type: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
}

/// Room information for comparison
#[derive(Debug, Serialize)]
pub struct RoomInfo {
    pub source: String,
    pub room_type: String,
    pub price: f64,
    pub meal_plan: Option<String>,
}

/// Room comparison result
#[derive(Debug, Deserialize)]
pub struct RoomComparison {
    pub normalized_room_type: String,
    pub sources: Vec<SourcePrice>,
    pub is_comparable: bool,
}

#[derive(Debug, Deserialize)]
pub struct SourcePrice {
    pub source: String,
    pub price: f64,
}
