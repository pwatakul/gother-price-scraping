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

use super::handlers;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub rabbitmq: lapin::Channel,
    pub config: Config,
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
        .route("/hotel-groups/:id/hotels", post(handlers::hotel_groups::add_hotel))
        .route("/hotel-groups/:group_id/hotels/:hotel_id", delete(handlers::hotel_groups::remove_hotel))
        .route("/hotel-groups/:id/jobs", get(handlers::hotel_groups::list_jobs))
        // Hotels
        .route("/hotels/search", get(handlers::hotels::search_hotels))
        // Scrape Jobs
        .route("/scrape-jobs", post(handlers::scrape_jobs::create_job))
        .route("/scrape-jobs/:id", get(handlers::scrape_jobs::get_job))
        .route("/scrape-jobs/:id", delete(handlers::scrape_jobs::cancel_job))
        .route("/scrape-jobs/:id/results", get(handlers::scrape_jobs::get_results))
        .route("/scrape-jobs/:id/export", get(handlers::scrape_jobs::export_excel))
        // Templates
        .route("/templates/hotel-import", get(handlers::templates::download_template));

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}
