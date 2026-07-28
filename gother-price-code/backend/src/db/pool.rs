//! Database Pool
//!
//! Creates and manages PostgreSQL connection pool.

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Create a new PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Type alias for the database pool
pub type DbPool = PgPool;
