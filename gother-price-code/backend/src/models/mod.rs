//! Models Module
//!
//! Shared data structures used across the application.

pub mod analytics;
pub mod hotel;
pub mod hotel_group;
pub mod price_history;
pub mod scheduled_scrape_config;
pub mod scrape_job;
pub mod scrape_result;
pub mod user;

pub use analytics::*;
pub use hotel::*;
pub use hotel_group::*;
pub use price_history::*;
pub use scheduled_scrape_config::*;
pub use scrape_job::*;
pub use scrape_result::*;
pub use user::*;
