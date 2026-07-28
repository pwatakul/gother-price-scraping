//! API Module
//!
//! HTTP endpoints and routing using Axum.

pub mod handlers;
pub mod middleware;
pub mod requests;
pub mod responses;
pub mod router;

pub use router::AppState;
