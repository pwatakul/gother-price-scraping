//! API Router
//!
//! Defines all HTTP routes and creates the Axum router.

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::scraper::registry::{default_registry, ScraperFactory};

use super::handlers;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub rabbitmq: lapin::Channel,
    pub config: Config,
    /// The scraper adapter registry (see scraper/registry.rs) — built
    /// once at startup, shared behind an Arc.
    pub scraper_registry: Arc<Vec<Box<dyn ScraperFactory>>>,
}

impl AppState {
    /// Convenience constructor so callers don't have to build the
    /// registry themselves.
    pub fn new(db: PgPool, redis: redis::aio::ConnectionManager, rabbitmq: lapin::Channel, config: Config) -> Self {
        Self { db, redis, rabbitmq, config, scraper_registry: Arc::new(default_registry()) }
    }
}

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api_routes = Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        // Hotel Groups
        .route("/hotel-groups", get(handlers::hotel_groups::list_groups))
        .route("/hotel-groups", post(handlers::hotel_groups::create_group))
        .route("/hotel-groups/:id", get(handlers::hotel_groups::get_group))
        .route("/hotel-groups/:id", put(handlers::hotel_groups::update_group))
        .route("/hotel-groups/:id", delete(handlers::hotel_groups::delete_group))
        .route("/hotel-groups/:id/import", post(handlers::hotel_groups::import_hotels))
        .route("/hotel-groups/:id/import-master", post(handlers::hotel_groups::import_master_hotels))
        .route("/hotel-groups/:id/hotels", post(handlers::hotel_groups::add_hotel))
        .route("/hotel-groups/:group_id/hotels/:hotel_id", delete(handlers::hotel_groups::remove_hotel))
        .route("/hotel-groups/:id/jobs", get(handlers::hotel_groups::list_jobs))
        // Hotels
        .route("/hotels/search", get(handlers::hotels::search_hotels))
        // Hotel Directory (REQ-007 — global "All Hotels" page)
        .route("/hotels", get(handlers::hotel_directory::list_hotels))
        .route("/hotels/countries", get(handlers::hotel_directory::list_countries))
        .route("/hotels/cities", get(handlers::hotel_directory::list_cities))
        .route("/hotels/export", get(handlers::hotel_directory::export_hotels))
        .route("/hotels/:id", get(handlers::hotel_directory::get_hotel_detail))
        // Scrape Jobs
        .route("/scrape-jobs", post(handlers::scrape_jobs::create_job))
        .route("/scrape-jobs/with-overrides", post(handlers::scrape_jobs::create_job_with_overrides))
        .route("/scrape-jobs/:id", get(handlers::scrape_jobs::get_job))
        .route("/scrape-jobs/:id", delete(handlers::scrape_jobs::cancel_job))
        .route("/scrape-jobs/:id/results", get(handlers::scrape_jobs::get_results))
        .route("/scrape-jobs/:id/export", get(handlers::scrape_jobs::export_excel))
        // Templates
        .route("/templates/hotel-import", get(handlers::templates::download_template))
        // Price History (REQ-002 F-007/F-008, REQ-005 F-006)
        .route("/price-history", get(handlers::price_history::list_price_history))
        .route("/price-history/hotel/:id/trend", get(handlers::price_history::hotel_trend))
        .route("/export/price-history", get(handlers::price_history::export_price_history))
        // Scheduled Scrape Configs (REQ-002 F-003/F-004)
        .route(
            "/scheduled-scrape-configs",
            post(handlers::scheduled_scrape_configs::create_config),
        )
        .route(
            "/scheduled-scrape-configs",
            get(handlers::scheduled_scrape_configs::list_configs),
        )
        .route(
            "/scheduled-scrape-configs/:id",
            put(handlers::scheduled_scrape_configs::update_config),
        )
        .route(
            "/scheduled-scrape-configs/:id",
            delete(handlers::scheduled_scrape_configs::delete_config),
        )
        // Analytics (REQ-003)
        .route("/analytics/overview", get(handlers::analytics::overview))
        .route("/analytics/market-position", get(handlers::analytics::market_position))
        .route("/analytics/heatmap", get(handlers::analytics::heatmap))
        .route("/analytics/win-rate", get(handlers::analytics::win_rate))
        .route("/analytics/parity-violations", get(handlers::analytics::parity_violations))
        .route("/analytics/booking-window/:hotel_id", get(handlers::analytics::booking_window))
        .route("/analytics/export", get(handlers::analytics::export));

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}
