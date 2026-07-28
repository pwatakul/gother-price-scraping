//! AI Prompts
//!
//! Prompt templates for Gemini AI.

use super::RoomInfo;

/// Prompt to normalize room type
pub fn normalize_room_type_prompt(room_type: &str) -> String {
    format!(
        r#"Normalize this hotel room type to a standard category.

Input room type: "{}"

Standard categories:
- Standard Room
- Superior Room
- Deluxe Room
- Deluxe King
- Deluxe Twin
- Premier Room
- Executive Room
- Club Room
- Junior Suite
- Executive Suite
- Deluxe Suite
- Presidential Suite
- Suite
- King Room
- Queen Room
- Twin Room
- Double Room
- Single Room
- Villa
- Pool Villa
- Bungalow

Respond with ONLY the normalized room type category, nothing else."#,
        room_type
    )
}

/// Prompt to extract hotel information from text
pub fn extract_hotel_info_prompt(text: &str) -> String {
    format!(
        r#"Extract hotel information from this text and return as JSON.

Text: "{}"

Return a JSON object with these fields (use null if not found):
{{
  "hotel_name": "string",
  "city": "string",
  "country": "string",
  "room_type": "string or null",
  "price": number or null,
  "currency": "string or null"
}}

Respond with ONLY the JSON object, no explanation."#,
        text
    )
}

/// Prompt to compare room types across sources
pub fn compare_rooms_prompt(rooms: &[RoomInfo]) -> String {
    let rooms_json = serde_json::to_string_pretty(rooms).unwrap_or_default();
    
    format!(
        r#"Compare these hotel room offerings from different sources and group comparable rooms.

Room data:
{}

For apple-to-apple comparison, group rooms that are truly equivalent (same bed type, similar amenities).

Return a JSON array of comparisons:
[
  {{
    "normalized_room_type": "Deluxe King",
    "sources": [
      {{"source": "agoda", "price": 4500}},
      {{"source": "booking", "price": 4600}}
    ],
    "is_comparable": true
  }}
]

Only include rooms that can be fairly compared. Set is_comparable to false if the rooms have significant differences.

Respond with ONLY the JSON array, no explanation."#,
        rooms_json
    )
}

/// Prompt to validate price data
pub fn validate_price_prompt(hotel_name: &str, prices: &[(String, f64)]) -> String {
    let prices_str = prices
        .iter()
        .map(|(source, price)| format!("{}: ฿{:.2}", source, price))
        .collect::<Vec<_>>()
        .join("\n");
    
    format!(
        r#"Validate these hotel prices for reasonability.

Hotel: {}
Prices:
{}

Check for:
1. Unusually low prices (possible errors)
2. Unusually high prices (possible currency issues)
3. Large discrepancies between sources (>50% difference)

Return a JSON object:
{{
  "is_valid": true/false,
  "issues": ["list of issues if any"],
  "suggested_corrections": []
}}

Respond with ONLY the JSON object."#,
        hotel_name, prices_str
    )
}
