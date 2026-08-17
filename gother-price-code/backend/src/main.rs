//! Hotel Price Scraper - Main Entry Point
//!
//! This is the main entry point for the Hotel Price Scraper backend.
//! It initializes all services and starts both the HTTP server and worker.

use anyhow::Result;
use sqlx::migrate::Migrator;
use std::path::Path;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ai;
mod api;
mod cache;
mod config;
mod db;
mod error;
mod excel;
mod models;
mod normalizer;
mod queue;
mod scraper;
mod worker;

use config::Config;

/// Creates the initial `admin` account when no users exist yet.
///
/// Deliberately not part of migration 026: an Argon2 hash embeds its salt, so
/// seeding from SQL would commit one fixed password to git and leave
/// ADMIN_PASSWORD with nothing to override.
async fn seed_admin_user(pool: &sqlx::PgPool, config: &Config) -> Result<()> {
    use db::repositories::user_repo::UserRepo;
    use models::user::{Role, DEFAULT_ADMIN_PASSWORD, DEFAULT_ADMIN_USERNAME};

    if UserRepo::count(pool).await? > 0 {
        return Ok(());
    }

    let using_default = config.admin_password.is_none();
    let password = config
        .admin_password
        .as_deref()
        .unwrap_or(DEFAULT_ADMIN_PASSWORD);

    UserRepo::create(pool, DEFAULT_ADMIN_USERNAME, password, Role::Admin).await?;

    if using_default {
        warn!(
            "🔑 Created user '{}' with the DEFAULT password — change it, or set ADMIN_PASSWORD before first start",
            DEFAULT_ADMIN_USERNAME
        );
    } else {
        info!(
            "🔑 Created user '{}' with the password from ADMIN_PASSWORD",
            DEFAULT_ADMIN_USERNAME
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hotel_price_scraper=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🏨 Starting Hotel Price Scraper...");

    // Load configuration
    let config = Config::from_env()?;
    info!("✅ Configuration loaded");

    // SerpAPI is the only source of live, date-specific rates (ADR-007).
    // Without it every scrape fails, and an all-green startup log would
    // hide that until someone runs a job and reads the per-hotel errors.
    let serpapi_configured =
        !config.serpapi_key.is_empty() && config.serpapi_key != "your_serpapi_key_here";
    if config.enable_mock_scraper {
        warn!("⚠️  ENABLE_MOCK_SCRAPER is on — scrapes will return FABRICATED prices");
    } else if !serpapi_configured {
        warn!("⚠️  No live price source: SERPAPI_KEY is not set, so all scrapes will fail");
    }

    // Initialize database pool
    let db_pool = db::create_pool(&config.database_url).await?;
    info!("✅ Database connected");

    // Run migrations at runtime
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&db_pool).await?;
    info!("✅ Migrations applied");

    // Seed the first login account (REQ-009). Only when the table is empty,
    // so a password changed through the UI survives every restart.
    seed_admin_user(&db_pool, &config).await?;

    // Initialize Redis
    let redis_client = cache::create_client(&config.redis_url).await?;
    info!("✅ Redis connected");

    // Initialize RabbitMQ
    let rabbitmq_channel = queue::create_channel(&config.rabbitmq_url).await?;
    info!("✅ RabbitMQ connected");

    // Create application state
    let app_state = api::AppState::new(db_pool.clone(), redis_client.clone(), rabbitmq_channel.clone(), config.clone());

    // Spawn worker in background
    let worker_state = app_state.clone();
    tokio::spawn(async move {
        info!("🔧 Starting background worker...");
        if let Err(e) = worker::run(worker_state).await {
            tracing::error!("Worker error: {:?}", e);
        }
    });

    // Spawn scheduled-scrape scheduler in background (REQ-002 F-005)
    let scheduler_state = std::sync::Arc::new(app_state.clone());
    tokio::spawn(async move {
        info!("⏰ Starting scheduled-scrape scheduler...");
        worker::scheduler::run(scheduler_state).await;
    });

    // Spawn partition manager in background (REQ-005 F-002) — keeps
    // hotel_price_history's rolling partition window topped up
    let partition_state = std::sync::Arc::new(app_state.clone());
    tokio::spawn(async move {
        info!("🗂 Starting hotel_price_history partition manager...");
        worker::partition_manager::run(partition_state).await;
    });

    // Build router
    let app = api::router::create_router(app_state);

    // Start server
    let addr = format!("{}:{}", config.app_host, config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
