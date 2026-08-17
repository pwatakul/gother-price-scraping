//! Scraper Registry — the adapter/plugin pattern.
//!
//! To add a new scraper later: implement `Scraper` (unchanged trait) +
//! a small `ScraperFactory` wrapper below + one line in
//! `default_registry()`. No changes needed in `worker/jobs/scrape_job.rs`.
//! See docs/decisions/ADR-001-scraper-choice.md for the full walkthrough.

use crate::config::Config;
use crate::models::scrape_job::ScrapeMethod;
use crate::scraper::{GeminiScraper, GotherScraper, MockScraper, Scraper, SerpApiScraper};

/// Builds a scraper for a given config, or returns None if unconfigured.
/// `None` means "this source is not configured" — never fabricate data for
/// a missing key. Callers report the skip by name so a failed scrape can
/// say *which* source was unavailable (see `summarize_outcomes`).
pub trait ScraperFactory: Send + Sync {
    /// Stable identifier, used in per-scraper outcome reporting. Matches
    /// the built scraper's `Scraper::name()`.
    fn name(&self) -> &'static str;
    /// Which ScrapeMethod values this factory participates in.
    fn methods(&self) -> &'static [ScrapeMethod];
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>>;

    /// A fallback source is deferred under `method=both` and runs only if
    /// the primary tier produced nothing — it fills blanks, it never
    /// competes with a real scrape. Choosing it explicitly (e.g.
    /// `method=gemini`) still runs it normally. See ADR-011.
    fn is_fallback(&self) -> bool {
        false
    }
}

pub struct SerpApiFactory;
impl ScraperFactory for SerpApiFactory {
    fn name(&self) -> &'static str {
        "serpapi"
    }
    fn methods(&self) -> &'static [ScrapeMethod] {
        &[ScrapeMethod::Serpapi, ScrapeMethod::Both]
    }
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>> {
        let key = &config.serpapi_key;
        if key.is_empty() || key == "your_serpapi_key_here" {
            return None;
        }
        Some(Box::new(SerpApiScraper::new(key)))
    }
}

pub struct GeminiFactory;
impl ScraperFactory for GeminiFactory {
    fn name(&self) -> &'static str {
        "gemini"
    }
    fn methods(&self) -> &'static [ScrapeMethod] {
        // Both now includes Gemini (fixes a pre-existing exact-equality
        // bug where method==Gemini was checked instead of matches!(.., Both)).
        &[ScrapeMethod::Gemini, ScrapeMethod::Both]
    }
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>> {
        GeminiScraper::from_config(&config.gemini_api_key, &config.gemini_model)
            .map(|s| Box::new(s) as Box<dyn Scraper>)
    }
    /// Gemini is knowledge-based, not a scrape, and has been measured
    /// fabricating cross-OTA prices. Under `both` it only fills blanks.
    fn is_fallback(&self) -> bool {
        true
    }
}

pub struct GotherFactory;
impl ScraperFactory for GotherFactory {
    fn name(&self) -> &'static str {
        "gother"
    }
    fn methods(&self) -> &'static [ScrapeMethod] {
        // Gother is cross-cutting today — attempted regardless of the
        // job's chosen method, unchanged from the pre-registry behavior.
        &[ScrapeMethod::Serpapi, ScrapeMethod::Gemini, ScrapeMethod::Both]
    }
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>> {
        if config.gother_api_url.is_empty() || config.gother_api_key == "your_gother_api_key_here" {
            return None;
        }
        Some(Box::new(GotherScraper::new(&config.gother_api_url, &config.gother_api_key)))
    }
}

/// Fabricates realistic-looking prices for demos. Unlike every other
/// factory, being "configured" is an explicit opt-in (ENABLE_MOCK_SCRAPER)
/// rather than the presence of a credential — a missing API key must never
/// silently resolve to invented data. See ADR-008.
pub struct MockFactory;
impl ScraperFactory for MockFactory {
    fn name(&self) -> &'static str {
        "mock"
    }
    fn methods(&self) -> &'static [ScrapeMethod] {
        &[ScrapeMethod::Serpapi, ScrapeMethod::Gemini, ScrapeMethod::Both]
    }
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>> {
        if !config.enable_mock_scraper {
            return None;
        }
        tracing::warn!("ENABLE_MOCK_SCRAPER is on — returning FABRICATED prices, not real scrapes");
        Some(Box::new(MockScraper::new()))
    }
}

pub fn default_registry() -> Vec<Box<dyn ScraperFactory>> {
    vec![
        Box::new(SerpApiFactory),
        Box::new(GeminiFactory),
        Box::new(GotherFactory),
        Box::new(MockFactory),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeFactory {
        methods: &'static [ScrapeMethod],
        configured: bool,
    }
    impl ScraperFactory for FakeFactory {
        fn name(&self) -> &'static str {
            "fake"
        }
        fn methods(&self) -> &'static [ScrapeMethod] {
            self.methods
        }
        fn build(&self, _config: &Config) -> Option<Box<dyn Scraper>> {
            if self.configured {
                panic!("test should never actually build/call a real scraper");
            }
            None
        }
    }

    #[test]
    fn default_registry_has_one_factory_per_scraper() {
        let registry = default_registry();
        assert_eq!(registry.len(), 4);
    }

    /// The mock scraper must be unreachable unless explicitly enabled —
    /// this is the guard against fabricated data reaching the database.
    #[test]
    fn mock_factory_is_off_unless_explicitly_enabled() {
        let mut config = Config::test_default();
        assert!(!config.enable_mock_scraper, "must default to off");
        assert!(MockFactory.build(&config).is_none());

        config.enable_mock_scraper = true;
        assert!(MockFactory.build(&config).is_some());
    }

    #[test]
    fn factory_is_skipped_when_method_does_not_match() {
        let factory = FakeFactory { methods: &[ScrapeMethod::Gemini], configured: true };
        assert!(!factory.methods().contains(&ScrapeMethod::Serpapi));
    }

    #[test]
    fn both_includes_gemini() {
        assert!(GeminiFactory.methods().contains(&ScrapeMethod::Both));
    }
}
