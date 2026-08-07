//! Scraper Registry — the adapter/plugin pattern.
//!
//! To add a new scraper later: implement `Scraper` (unchanged trait) +
//! a small `ScraperFactory` wrapper below + one line in
//! `default_registry()`. No changes needed in `worker/jobs/scrape_job.rs`.
//! See docs/decisions/ADR-001-scraper-choice.md for the full walkthrough.

use crate::config::Config;
use crate::models::scrape_job::ScrapeMethod;
use crate::scraper::{ChatGptScraper, GeminiScraper, GotherScraper, Scraper, SerpApiScraper};

/// Builds a scraper for a given config, or returns None if unconfigured.
/// `None` means "skip silently" — never fabricate data for a missing key.
pub trait ScraperFactory: Send + Sync {
    /// Which ScrapeMethod values this factory participates in.
    fn methods(&self) -> &'static [ScrapeMethod];
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>>;
}

pub struct SerpApiFactory;
impl ScraperFactory for SerpApiFactory {
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

pub struct ChatGptFactory;
impl ScraperFactory for ChatGptFactory {
    fn methods(&self) -> &'static [ScrapeMethod] {
        &[ScrapeMethod::Chatgpt, ScrapeMethod::Both]
    }
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>> {
        ChatGptScraper::from_config(&config.openai_api_key, &config.openai_model)
            .map(|s| Box::new(s) as Box<dyn Scraper>)
    }
}

pub struct GeminiFactory;
impl ScraperFactory for GeminiFactory {
    fn methods(&self) -> &'static [ScrapeMethod] {
        // Both now includes Gemini (fixes a pre-existing exact-equality
        // bug where method==Gemini was checked instead of matches!(.., Both)).
        &[ScrapeMethod::Gemini, ScrapeMethod::Both]
    }
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>> {
        GeminiScraper::from_config(&config.gemini_api_key, &config.gemini_model)
            .map(|s| Box::new(s) as Box<dyn Scraper>)
    }
}

pub struct GotherFactory;
impl ScraperFactory for GotherFactory {
    fn methods(&self) -> &'static [ScrapeMethod] {
        // Gother is cross-cutting today — attempted regardless of the
        // job's chosen method, unchanged from the pre-registry behavior.
        &[ScrapeMethod::Serpapi, ScrapeMethod::Chatgpt, ScrapeMethod::Gemini, ScrapeMethod::Both]
    }
    fn build(&self, config: &Config) -> Option<Box<dyn Scraper>> {
        if config.gother_api_url.is_empty() || config.gother_api_key == "your_gother_api_key_here" {
            return None;
        }
        Some(Box::new(GotherScraper::new(&config.gother_api_url, &config.gother_api_key)))
    }
}

pub fn default_registry() -> Vec<Box<dyn ScraperFactory>> {
    vec![Box::new(SerpApiFactory), Box::new(ChatGptFactory), Box::new(GeminiFactory), Box::new(GotherFactory)]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeFactory {
        methods: &'static [ScrapeMethod],
        configured: bool,
    }
    impl ScraperFactory for FakeFactory {
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
