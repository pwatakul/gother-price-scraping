//! API Response Types
//!
//! Response DTOs for API endpoints.

// Re-export from models for convenience
pub use crate::models::{
    Hotel,
    HotelGroup,
    HotelGroupWithCount,
    HotelWithPrice,
    ScrapeJob,
    ScrapeJobWithProgress,
    ScrapeResultsResponse,
};
