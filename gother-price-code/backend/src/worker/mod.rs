//! Worker Module
//!
//! Background job processor for scrape jobs.

pub mod jobs;
pub mod partition_manager;
pub mod processor;
pub mod scheduler;

pub use processor::*;
