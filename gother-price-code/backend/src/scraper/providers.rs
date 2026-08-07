//! Known provider names (REQ-001-v1.2 F-022).
//!
//! The CEO brief names exactly four sources: Gother, Agoda, Trip, and Wink
//! (Wink domestic-only). All scrapers must normalize onto these names;
//! anything else (e.g. SerpAPI's raw "booking"/"expedia"/"hotels.com")
//! is dropped so comparisons stay apples-to-apples.

pub const GOTHER: &str = "gother";
pub const AGODA: &str = "agoda";
pub const TRIP: &str = "trip";
pub const WINK: &str = "wink";

pub const KNOWN_PROVIDERS: [&str; 4] = [GOTHER, AGODA, TRIP, WINK];

/// Providers that are domestic-only (REQ-001 F-022: Wink is Thailand-only).
pub const DOMESTIC_ONLY_PROVIDERS: [&str; 1] = [WINK];

/// Providers that represent a direct-contract rate (REQ-001 F-026:
/// Wink/HyperGuest). No scraper currently produces "hyperguest" rows, but
/// the name is recognized here so the flag is ready when one does.
pub const DIRECT_CONTRACT_PROVIDERS: [&str; 2] = [WINK, "hyperguest"];

pub fn is_direct_contract(source: &str) -> bool {
    DIRECT_CONTRACT_PROVIDERS.contains(&source)
}
