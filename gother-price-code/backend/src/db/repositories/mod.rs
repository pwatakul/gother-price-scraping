//! Database Repositories
//!
//! Data access layer for all database operations.

pub mod hotel_group_repo;
pub mod hotel_repo;
pub mod scrape_job_repo;
pub mod scrape_result_repo;

pub use hotel_group_repo::*;
pub use hotel_repo::*;
pub use scrape_job_repo::*;
pub use scrape_result_repo::*;
