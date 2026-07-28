//! Models Module
//!
//! Shared data structures used across the application.

pub mod hotel;
pub mod hotel_group;
pub mod scrape_job;
pub mod scrape_result;

pub use hotel::*;
pub use hotel_group::*;
pub use scrape_job::*;
pub use scrape_result::*;
