//! SerpAPI Scraper
//!
//! Scrapes hotel prices using SerpAPI's Google Hotels API.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::providers;
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

/// SerpAPI's google_hotels engine returns two different shapes:
///
/// - a **list** result (`properties: [...]`) for broad queries like
///   "hotels in Bangkok";
/// - a **single-property** result for a query that resolves to exactly one
///   hotel, where the property's own fields (`name`, `prices`, `link`) sit
///   at the *top level* of the response and `properties` is absent.
///
/// Since `scrape()` always queries one named hotel, the single-property
/// shape is the common case — reading only `properties` made every scrape
/// return "Hotel not found in search results" even with a valid API key.
/// `#[serde(flatten)]` captures the top-level shape without duplicating
/// the field list.
#[derive(Debug, Deserialize)]
struct SerpApiResponse {
    #[serde(default)]
    properties: Vec<SerpApiProperty>,
    #[serde(flatten)]
    single: SerpApiProperty,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerpApiProperty {
    name: Option<String>,
    #[serde(default)]
    prices: Vec<SerpApiPrice>,
    link: Option<String>,
    /// Identifies the property for a follow-up request. List results carry
    /// this but no prices, so it is the only way to reach their rates.
    property_token: Option<String>,
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

impl SerpApiScraper {
    /// One SerpAPI google_hotels request. Shared by the initial search and
    /// the property_token follow-up so both send identical stay parameters
    /// — a mismatch there would return prices for a different stay.
    async fn search(
        &self,
        query: &str,
        params: &ScrapeParams,
        property_token: Option<&str>,
    ) -> anyhow::Result<SerpApiResponse> {
        let mut request = self.client.get(&self.base_url).query(&[
            ("engine", "google_hotels"),
            ("q", query),
            ("check_in_date", &params.checkin_date.to_string()),
            ("check_out_date", &params.checkout_date.to_string()),
            ("adults", &params.adults.to_string()),
            ("rooms", &params.rooms.to_string()),
            ("currency", "THB"),
            ("gl", "th"),
            ("hl", "en"),
            ("api_key", &self.api_key),
        ]);
        if let Some(token) = property_token {
            request = request.query(&[("property_token", token)]);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            anyhow::bail!("SerpAPI request failed: {}", response.status());
        }

        let data: SerpApiResponse = response.json().await?;
        if let Some(error) = data.error {
            anyhow::bail!("SerpAPI error: {}", error);
        }
        Ok(data)
    }
}

#[async_trait]
impl Scraper for SerpApiScraper {
    fn name(&self) -> &'static str {
        "serpapi"
    }

    async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>> {
        let query = format!("{} {} {}", params.hotel_name, params.city, params.country);

        let data = self.search(&query, params, None).await?;

        let Some(selected) = select_property(&data, &params.hotel_name) else {
            // Naming the nearest candidate turns a dead end into something
            // actionable: the hotel can be renamed to match via the edit
            // dialog. Guessing at the first result instead would risk
            // storing another hotel's prices under this one.
            let closest = data
                .properties
                .iter()
                .filter_map(|p| p.name.as_deref())
                .next()
                .unwrap_or("none");
            anyhow::bail!("Hotel not found in search results (closest: {closest})");
        };

        // A list result identifies properties but carries no prices — the
        // rates only exist behind a second lookup by property_token. Which
        // shape SerpAPI returns varies per request, not per hotel, so this
        // is a normal path rather than an edge case.
        let followed;
        let property = if selected.prices.is_empty() && selected.property_token.is_some() {
            let token = selected.property_token.clone().unwrap();
            tracing::debug!("Following property_token for {}", params.hotel_name);
            followed = self.search(&query, params, Some(&token)).await?;
            // The token response is always the single-property shape.
            &followed.single
        } else {
            selected
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

            // Map onto the allowlist (REQ-001-v1.4 F-022 / ADR-009).
            // Unrecognized resellers are dropped rather than stored under a
            // guessed name. Hotel/city/country let the hotel's own site be
            // recognized as `direct` — SerpAPI has no flag for it.
            let Some(normalized_source) = providers::normalize_source(
                &source,
                &params.hotel_name,
                &params.city,
                &params.country,
            ) else {
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
                // Stamped by the registry loop from the factory name (ADR-011).
                via_method: String::new(),
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

/// Comparison key for hotel names: lowercase, letters and digits only.
///
/// SerpAPI's spelling rarely matches ours exactly — it lists "Hua Hin
/// Grand Hotel and Plaza" as "Huahin Grand Hotel and Plaza" — so spacing
/// and punctuation have to be ignored to match at all.
///
/// Deliberately *not* `Hotel::normalize_name`: that one strips the words
/// "hotel" and "resort" for deduplication, which would make "Hua Hin Grand
/// Hotel" and "Hua Hin Grand" look like the same property. Matching needs
/// the stricter key.
fn match_key(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Pick the property to read prices from, handling both response shapes
/// (see `SerpApiResponse`). Pure, so both shapes are unit-testable.
///
/// Returns `None` when nothing in a list result matches by name. There is
/// deliberately no "just take the first result" fallback: the first entry
/// of a 20-property list is usually a different hotel, and using it would
/// store that hotel's prices under this one — indistinguishable from
/// correct data once written.
fn select_property<'a>(data: &'a SerpApiResponse, hotel_name: &str) -> Option<&'a SerpApiProperty> {
    let needle = match_key(hotel_name);

    // Either direction: our name may be longer than SerpAPI's listing
    // ("Grande Centre Point Lumphini Bangkok" vs "Grande Centre Point
    // Lumphini") or shorter.
    let matched = data.properties.iter().find(|p| {
        p.name
            .as_ref()
            .map(|n| {
                let key = match_key(n);
                !key.is_empty() && !needle.is_empty() && (key.contains(&needle) || needle.contains(&key))
            })
            .unwrap_or(false)
    });

    if matched.is_some() {
        return matched;
    }

    // Single-property shape: the response *is* the property. Only trust it
    // if the top level actually carried data, so an empty response bails.
    if data.single.name.is_some() || !data.single.prices.is_empty() {
        return Some(&data.single);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> SerpApiResponse {
        serde_json::from_str(json).expect("fixture should deserialize")
    }

    /// Shape actually returned for a query naming one hotel — property
    /// fields at the top level, no `properties` array. Trimmed from a real
    /// response for "Anantara Riverside Bangkok".
    const SINGLE_PROPERTY: &str = r#"{
        "type": "hotel",
        "name": "Anantara Riverside Bangkok Resort",
        "link": "https://www.anantara.com/riverside-bangkok",
        "rate_per_night": {"lowest": "THB 6,018", "extracted_lowest": 6018},
        "prices": [
            {"source": "Anantara.com", "rate_per_night": {"lowest": "THB 5,858", "extracted_lowest": 5858.0}},
            {"source": "Trip.com", "type": "Deluxe Room", "rate_per_night": {"lowest": "THB 6,759", "extracted_lowest": 6759.0}},
            {"source": "Booking.com", "rate_per_night": {"lowest": "THB 6,704", "extracted_lowest": 6704.0}}
        ]
    }"#;

    /// Shape returned for a broad query — properties array, nothing useful
    /// at the top level.
    const PROPERTY_LIST: &str = r#"{
        "properties": [
            {"name": "Hotel Royal Bangkok", "prices": []},
            {"name": "Siam Kempinski Hotel Bangkok", "link": "https://example.test/kempinski",
             "prices": [{"source": "Agoda", "rate_per_night": {"lowest": "THB 8,100", "extracted_lowest": 8100.0}}]}
        ]
    }"#;

    #[test]
    fn reads_single_property_response_shape() {
        let data = parse(SINGLE_PROPERTY);
        assert!(data.properties.is_empty(), "fixture has no properties array");

        let p = select_property(&data, "Anantara Riverside Bangkok Resort")
            .expect("top-level property should be found");
        assert_eq!(p.name.as_deref(), Some("Anantara Riverside Bangkok Resort"));
        assert_eq!(p.prices.len(), 3);
    }

    #[test]
    fn prefers_name_match_within_property_list() {
        let data = parse(PROPERTY_LIST);
        let p = select_property(&data, "Siam Kempinski Hotel Bangkok").unwrap();
        assert_eq!(p.name.as_deref(), Some("Siam Kempinski Hotel Bangkok"));
    }

    /// The guard against the dangerous failure: with no name match, the
    /// first entry of a list is a *different hotel*, and using it would
    /// store its prices under ours, indistinguishable from real data.
    #[test]
    fn unmatched_name_selects_nothing_rather_than_guessing() {
        let data = parse(PROPERTY_LIST);
        assert!(select_property(&data, "A Hotel That Is Not Listed").is_none());
    }

    /// SerpAPI writes "Huahin"; the hotel is stored as "Hua Hin". Spacing
    /// must not decide whether a scrape works.
    #[test]
    fn matches_across_spacing_and_punctuation() {
        let data = parse(
            r#"{"properties":[{"name":"Huahin Grand Hotel and Plaza","prices":[],"property_token":"tok1"}]}"#,
        );
        let p = select_property(&data, "Hua Hin Grand Hotel and Plaza").unwrap();
        assert_eq!(p.name.as_deref(), Some("Huahin Grand Hotel and Plaza"));
        assert_eq!(p.property_token.as_deref(), Some("tok1"));
    }

    /// Our name is often longer than the listing — matching has to work in
    /// both directions.
    #[test]
    fn matches_when_our_name_is_longer_than_the_listing() {
        let data = parse(
            r#"{"properties":[{"name":"Grande Centre Point Lumphini","prices":[],"property_token":"tok2"}]}"#,
        );
        let p = select_property(&data, "Grande Centre Point Lumphini Bangkok").unwrap();
        assert_eq!(p.name.as_deref(), Some("Grande Centre Point Lumphini"));
    }

    /// A list entry carries a token but no prices — that pairing is the
    /// signal to make the follow-up request.
    #[test]
    fn list_entries_expose_a_token_and_no_prices() {
        let data = parse(
            r#"{"properties":[{"name":"Grande Centre Point Lumphini","prices":[],"property_token":"tok3"}]}"#,
        );
        let p = select_property(&data, "Grande Centre Point Lumphini").unwrap();
        assert!(p.prices.is_empty(), "list results carry no prices");
        assert!(p.property_token.is_some(), "so the token must be available");
    }

    #[test]
    fn empty_response_selects_nothing() {
        let data = parse(r#"{"search_metadata": {"status": "Success"}}"#);
        assert!(select_property(&data, "Anything").is_none());
    }

    /// End-to-end over the selected property: the fixture's three sources
    /// are the hotel's own site, Trip.com and Booking.com — under ADR-009
    /// all three are kept, with the hotel's own site labelled `direct`.
    #[test]
    fn single_property_maps_sources_onto_the_allowlist() {
        let data = parse(SINGLE_PROPERTY);
        let p = select_property(&data, "Anantara Riverside Bangkok Resort").unwrap();
        let kept: Vec<_> = p
            .prices
            .iter()
            .filter_map(|pr| {
                providers::normalize_source(
                    pr.source.as_deref()?,
                    "Anantara Riverside Bangkok Resort",
                    "Bangkok",
                    "Thailand",
                )
            })
            .collect();
        assert_eq!(
            kept,
            vec![
                providers::DIRECT.to_string(),
                providers::TRIP.to_string(),
                providers::BOOKING.to_string(),
            ]
        );
    }
}
