//! Known provider names and source normalization (REQ-001-v1.4 F-022).
//!
//! Every scraper normalizes its raw source strings through this module, so
//! the allowlist lives in exactly one place. Anything unrecognized is
//! dropped rather than stored under a guessed name — see ADR-005's
//! no-fabrication rule and ADR-009 for why this list is what it is.

pub const GOTHER: &str = "gother";
pub const AGODA: &str = "agoda";
pub const TRIP: &str = "trip";
pub const WINK: &str = "wink";
pub const BOOKING: &str = "booking";
pub const EXPEDIA: &str = "expedia";
pub const PRICELINE: &str = "priceline";
pub const TRAVELOKA: &str = "traveloka";
pub const KLOOK: &str = "klook";
/// The hotel's own website. Not a fixed brand — detected per hotel by
/// `normalize_source`, since SerpAPI exposes no "official rate" flag.
pub const DIRECT: &str = "direct";

pub const KNOWN_PROVIDERS: [&str; 10] = [
    GOTHER, AGODA, TRIP, WINK, BOOKING, EXPEDIA, PRICELINE, TRAVELOKA, KLOOK, DIRECT,
];

/// Providers that are domestic-only (REQ-001 F-022: Wink is Thailand-only).
pub const DOMESTIC_ONLY_PROVIDERS: [&str; 1] = [WINK];

/// Providers that represent a direct-contract rate (REQ-001 F-026:
/// Wink/HyperGuest). No scraper currently produces "hyperguest" rows, but
/// the name is recognized here so the flag is ready when one does.
///
/// Note `DIRECT` is deliberately absent: F-026 means a contract *with
/// Gother*, which a hotel's own public website is not.
pub const DIRECT_CONTRACT_PROVIDERS: [&str; 2] = [WINK, "hyperguest"];

pub fn is_direct_contract(source: &str) -> bool {
    DIRECT_CONTRACT_PROVIDERS.contains(&source)
}

/// Words that carry no brand identity, so they must never be what makes a
/// source look like the hotel's own site. Without this, a source called
/// "Bangkok Hotels" matches half the properties in the city.
const GENERIC_WORDS: [&str; 12] = [
    "hotel", "hotels", "resort", "resorts", "spa", "the", "by", "and", "a", "at", "de", "residences",
];

/// Strip a domain suffix and punctuation, lowercase, e.g.
/// "Mandarinoriental.com" -> "mandarinoriental", " Agoda " -> "agoda".
fn brand_token(source: &str) -> String {
    let lower = source.trim().to_lowercase();
    // Drop a trailing TLD-ish segment ("trip.com", "ca.KAYAK.com" -> keep
    // the meaningful part) without mangling names that merely contain dots.
    let without_tld = lower
        .strip_suffix(".com")
        .or_else(|| lower.strip_suffix(".net"))
        .or_else(|| lower.strip_suffix(".co.uk"))
        .unwrap_or(&lower);
    without_tld
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_string()
}

/// Split into meaningful lowercase tokens, dropping generic hotel words
/// and any geography supplied by the caller.
fn distinctive_tokens(text: &str, geo: &[&str]) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 2)
        .filter(|t| !GENERIC_WORDS.contains(t))
        .filter(|t| !geo.iter().any(|g| g.eq_ignore_ascii_case(t)))
        .map(|t| t.to_string())
        .collect()
}

/// Is this source the hotel's own website?
///
/// SerpAPI gives no "official rate" flag, so this is a heuristic: the
/// source name resembles the hotel's own name ("Conrad Bangkok",
/// "Mandarinoriental.com", "Anantara.com"). It is deliberately
/// conservative — a false positive files a competitor's rate as the
/// hotel's own price, which is worse than dropping the row.
fn is_hotel_direct(source: &str, hotel_name: &str, city: &str, country: &str) -> bool {
    let geo: Vec<&str> = vec![city, country];
    let hotel_tokens = distinctive_tokens(hotel_name, &geo);
    if hotel_tokens.is_empty() {
        return false;
    }

    let brand = brand_token(source);
    if brand.is_empty() {
        return false;
    }

    // "mandarinoriental" vs "Mandarin Oriental Bangkok" — the collapsed
    // hotel name starts with the source's brand.
    let collapsed: String = hotel_tokens.concat();
    if collapsed.starts_with(&brand) || brand.starts_with(&collapsed) {
        return true;
    }

    // Otherwise require every distinctive token of the source to be a
    // distinctive token of the hotel ("Conrad Bangkok" -> {conrad}).
    let source_tokens = distinctive_tokens(source, &geo);
    !source_tokens.is_empty()
        && source_tokens.iter().all(|t| hotel_tokens.contains(t))
}

/// Map a raw scraper source string onto an allowlisted provider, or None
/// to drop it.
///
/// Matching is on the whole brand token, never a substring: real SerpAPI
/// responses for Thai hotels include "EaseMyTrip.com", "Clicktrip.com" and
/// "Tripening Hotels", all of which a `contains("trip")` test would file
/// as Trip.com — attributing a competitor's rate to a named provider.
///
/// `hotel_name`/`city`/`country` are only used to recognize the hotel's
/// own website; pass empty strings when that is not applicable.
pub fn normalize_source(
    source: &str,
    hotel_name: &str,
    city: &str,
    country: &str,
) -> Option<String> {
    match brand_token(source).as_str() {
        "agoda" => return Some(AGODA.to_string()),
        "trip" | "ctrip" => return Some(TRIP.to_string()),
        "booking" => return Some(BOOKING.to_string()),
        "expedia" => return Some(EXPEDIA.to_string()),
        "priceline" => return Some(PRICELINE.to_string()),
        "traveloka" => return Some(TRAVELOKA.to_string()),
        "klook" => return Some(KLOOK.to_string()),
        "gother" => return Some(GOTHER.to_string()),
        "wink" => return Some(WINK.to_string()),
        _ => {}
    }

    if is_hotel_direct(source, hotel_name, city, country) {
        return Some(DIRECT.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(source: &str) -> Option<String> {
        normalize_source(source, "", "", "")
    }

    #[test]
    fn maps_allowlisted_brands() {
        for (raw, expected) in [
            ("Agoda", AGODA),
            ("Trip.com", TRIP),
            ("Ctrip", TRIP),
            ("Booking.com", BOOKING),
            ("Expedia.com", EXPEDIA),
            ("Priceline", PRICELINE),
            ("Traveloka.com", TRAVELOKA),
            ("klook", KLOOK),
        ] {
            assert_eq!(norm(raw), Some(expected.to_string()), "input: {raw:?}");
        }
    }

    /// Regression: all of these appear in real SerpAPI responses for Thai
    /// hotels and are *not* the brands their substrings suggest.
    #[test]
    fn does_not_confuse_lookalike_brands() {
        for raw in [
            "EaseMyTrip.com",
            "Clicktrip.com",
            "Tripening Hotels",
            "Bluepillow.com",
            "Etrip.net",
            "Tripadvisor.com",
            "Hotelscombined.com",
        ] {
            assert_eq!(norm(raw), None, "should be dropped: {raw:?}");
        }
    }

    #[test]
    fn drops_unknown_resellers() {
        for raw in ["Evendo", "Zzzello", "Reserving", "SKYLARK", "hutchgo", "müv AI", "Wego"] {
            assert_eq!(norm(raw), None, "should be dropped: {raw:?}");
        }
    }

    #[test]
    fn recognizes_the_hotels_own_site() {
        // Source name equals the hotel name.
        assert_eq!(
            normalize_source("Conrad Bangkok", "Conrad Bangkok", "Bangkok", "Thailand"),
            Some(DIRECT.to_string())
        );
        // Domain form of the brand.
        assert_eq!(
            normalize_source(
                "Mandarinoriental.com",
                "Mandarin Oriental Bangkok",
                "Bangkok",
                "Thailand"
            ),
            Some(DIRECT.to_string())
        );
        // Brand shorter than the full hotel name.
        assert_eq!(
            normalize_source(
                "Anantara.com",
                "Anantara Riverside Bangkok Resort",
                "Bangkok",
                "Thailand"
            ),
            Some(DIRECT.to_string())
        );
    }

    /// The dangerous direction: a competitor must never be filed as the
    /// hotel's own rate.
    #[test]
    fn does_not_mistake_competitors_for_direct() {
        for raw in ["Tripening Hotels", "Bangkok Hotels", "Booking.com", "Evendo", "Luxury Escapes"] {
            let got = normalize_source(raw, "Conrad Bangkok", "Bangkok", "Thailand");
            assert_ne!(got, Some(DIRECT.to_string()), "wrongly direct: {raw:?}");
        }
    }

    /// Geography alone must not make a source look like the hotel.
    #[test]
    fn city_and_country_are_not_distinctive() {
        assert_eq!(
            normalize_source("Bangkok", "Conrad Bangkok", "Bangkok", "Thailand"),
            None
        );
        assert_eq!(
            normalize_source("Thailand Hotels", "Conrad Bangkok", "Bangkok", "Thailand"),
            None
        );
    }

    #[test]
    fn known_providers_covers_every_constant() {
        for p in [GOTHER, AGODA, TRIP, WINK, BOOKING, EXPEDIA, PRICELINE, TRAVELOKA, KLOOK, DIRECT] {
            assert!(KNOWN_PROVIDERS.contains(&p), "missing from KNOWN_PROVIDERS: {p}");
        }
    }

    #[test]
    fn hotel_direct_is_not_a_direct_contract() {
        // F-026 means a contract with Gother; a hotel's public site is not.
        assert!(!is_direct_contract(DIRECT));
        assert!(is_direct_contract(WINK));
    }
}
