//! Scraper Module
//!
//! External API clients for price scraping.

pub mod gother;
pub mod mock;
pub mod serpapi;
pub mod traits;

pub use gother::*;
pub use mock::*;
pub use serpapi::*;
pub use traits::*;
