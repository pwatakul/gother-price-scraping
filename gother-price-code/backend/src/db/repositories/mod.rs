//! Database Repositories
//!
//! Data access layer for all database operations.

pub mod currency_repo;
pub mod hotel_directory_repo;
pub mod hotel_group_repo;
pub mod hotel_repo;
pub mod materialized_view_repo;
pub mod price_history_repo;
pub mod scheduled_scrape_config_repo;
pub mod scrape_job_repo;
pub mod scrape_result_repo;

pub use currency_repo::*;
pub use hotel_directory_repo::*;
pub use hotel_group_repo::*;
pub use hotel_repo::*;
pub use materialized_view_repo::*;
pub use price_history_repo::*;
pub use scheduled_scrape_config_repo::*;
pub use scrape_job_repo::*;
pub use scrape_result_repo::*;
