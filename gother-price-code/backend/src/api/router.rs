//! API Router
//!
//! Defines all HTTP routes and creates the Axum router.

use axum::{
    http::{header, HeaderValue, Method},
    middleware,
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
    let cors = build_cors(state.config.allowed_origin.as_deref());

    // The only two routes reachable without a session: the health probe (a
    // load balancer has no cookie) and login itself (you can't authenticate to
    // authenticate). Everything else goes in `api_routes` below, which is
    // wrapped in `require_auth` as a whole — so a route added later is
    // protected by default rather than by remembering to protect it.
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/logout", post(handlers::auth::logout));

    // API routes (authenticated)
    let api_routes = Router::new()
        .route("/auth/me", get(handlers::auth::me))
        .route("/auth/change-password", post(handlers::auth::change_password))
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
        // Saved per-group price search (ADR-012): edit it, then run it.
        .route(
            "/hotel-groups/:id/search-config",
            put(handlers::hotel_groups::update_search_config),
        )
        .route(
            "/hotel-groups/:id/search-runs",
            post(handlers::hotel_groups::run_saved_search),
        )
        // Hotels
        .route("/hotels/search", get(handlers::hotels::search_hotels))
        // Hotel Directory (REQ-007 — global "All Hotels" page)
        .route("/hotels", get(handlers::hotel_directory::list_hotels))
        .route("/hotels/countries", get(handlers::hotel_directory::list_countries))
        .route("/hotels/cities", get(handlers::hotel_directory::list_cities))
        .route("/hotels/export", get(handlers::hotel_directory::export_hotels))
        .route("/hotels/:id", get(handlers::hotel_directory::get_hotel_detail))
        .route("/hotels/:id", put(handlers::hotel_directory::update_hotel))
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
        .route(
            "/price-history/hotel/:id/trend/windows",
            get(handlers::price_history::hotel_trend_windows),
        )
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
        // Manual trigger (REQ-008 F-010) — fires the standard grid now,
        // without disturbing the cron cadence.
        .route(
            "/scheduled-scrape-configs/:id/run",
            post(handlers::scheduled_scrape_configs::run_config),
        )
        // Analytics (REQ-003)
        .route("/analytics/overview", get(handlers::analytics::overview))
        .route("/analytics/market-position", get(handlers::analytics::market_position))
        .route("/analytics/heatmap", get(handlers::analytics::heatmap))
        .route("/analytics/win-rate", get(handlers::analytics::win_rate))
        // Gother-independent leaderboard: who is cheapest, and by how much.
        .route(
            "/analytics/provider-benchmark",
            get(handlers::analytics::provider_benchmark),
        )
        .route("/analytics/parity-violations", get(handlers::analytics::parity_violations))
        .route("/analytics/booking-window/:hotel_id", get(handlers::analytics::booking_window))
        .route("/analytics/export", get(handlers::analytics::export));

    let state = Arc::new(state);

    // `route_layer` runs only for routes this router actually matches, so an
    // unknown path still 404s instead of being turned into a 401.
    let api_routes = api_routes.route_layer(middleware::from_fn_with_state(
        state.clone(),
        crate::api::middleware::require_auth,
    ));

    Router::new()
        .nest("/api", public_routes.merge(api_routes))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// CORS policy.
///
/// With `ALLOWED_ORIGIN` set (production) the origin is named exactly and
/// credentials are allowed, so the session cookie can ride along. Methods and
/// headers must then be listed explicitly: tower-http **panics at layer
/// construction** on `allow_credentials(true)` combined with a wildcard, and
/// the CORS spec forbids that pairing because a wildcard plus credentials
/// would let any site act as the signed-in user.
///
/// With it unset (local development) the permissive policy is kept. That
/// branch cannot carry credentials either — browsers reject a credentialed
/// request against a wildcard origin — so nothing is weakened by it.
fn build_cors(allowed_origin: Option<&str>) -> CorsLayer {
    match allowed_origin.and_then(|o| o.parse::<HeaderValue>().ok()) {
        Some(origin) => CorsLayer::new()
            .allow_origin(origin)
            .allow_credentials(true)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE]),
        None => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These assert "does not panic". CorsLayer validates its own combination
    // eagerly, and an invalid one took down the whole server at startup in
    // production rather than failing at compile time — so constructing both
    // branches is the check that matters.

    #[test]
    fn production_cors_is_a_valid_combination() {
        let _ = build_cors(Some("https://34-124-161-138.nip.io"));
    }

    #[test]
    fn development_cors_is_a_valid_combination() {
        let _ = build_cors(None);
    }

    #[test]
    fn unparseable_origin_falls_back_to_permissive_instead_of_panicking() {
        let _ = build_cors(Some("not a valid header value\n"));
    }
}
