//! Configuration Module
//!
//! Handles loading and validating environment configuration.

use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub database_url: String,
    pub database_max_connections: u32,

    // Redis
    pub redis_url: String,
    pub cache_ttl_seconds: u64,

    // RabbitMQ
    pub rabbitmq_url: String,
    pub rabbitmq_queue_name: String,

    // SerpAPI
    pub serpapi_key: String,
    pub serpapi_base_url: String,

    // Gother API
    pub gother_api_url: String,
    pub gother_api_key: String,

    // Gemini AI (optional)
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,

    /// Opt-in only (ENABLE_MOCK_SCRAPER=true). The mock scraper fabricates
    /// realistic-looking prices; it must never stand in for a missing key
    /// silently, or fake data reaches hotel_price_history looking real.
    /// See ADR-008.
    pub enable_mock_scraper: bool,

    // Application
    pub app_host: String,
    pub app_port: u16,

    // Worker
    pub worker_concurrency: usize,
    pub worker_retry_count: u32,

    // Polling
    pub polling_interval_ms: u64,
    pub price_cache_ttl_seconds: u64,

    // Authentication (REQ-009)
    /// Signing key for session JWTs. Required — see `from_env`.
    pub jwt_secret: String,
    /// Overrides the seeded admin password on first startup. When unset the
    /// well-known default is used and logged at WARN.
    pub admin_password: Option<String>,
    /// Adds `Secure` to the session cookie, so browsers only ever send it
    /// over HTTPS. Defaults to false: local development is served over plain
    /// HTTP, where a Secure cookie would simply never be sent and login would
    /// fail with no visible cause. Set COOKIE_SECURE=true in production.
    pub cookie_secure: bool,
    /// Exact origin allowed to make cross-origin calls, e.g.
    /// `https://34-1-2-3.nip.io`. When unset CORS stays permissive, which is
    /// correct for localhost development.
    pub allowed_origin: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            // Database
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("DATABASE_MAX_CONNECTIONS must be a number")?,

            // Redis
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .context("CACHE_TTL_SECONDS must be a number")?,

            // RabbitMQ
            rabbitmq_url: env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string()),
            rabbitmq_queue_name: env::var("RABBITMQ_QUEUE_NAME")
                .unwrap_or_else(|_| "scrape_jobs".to_string()),

            // SerpAPI
            serpapi_key: env::var("SERPAPI_KEY")
                .unwrap_or_else(|_| "".to_string()),
            serpapi_base_url: env::var("SERPAPI_BASE_URL")
                .unwrap_or_else(|_| "https://serpapi.com/search".to_string()),

            // Gother API
            gother_api_url: env::var("GOTHER_API_URL")
                .unwrap_or_else(|_| "".to_string()),
            gother_api_key: env::var("GOTHER_API_KEY")
                .unwrap_or_else(|_| "".to_string()),

            // Gemini AI
            gemini_api_key: env::var("GEMINI_API_KEY").ok(),
            gemini_model: env::var("GEMINI_MODEL")
                .unwrap_or_else(|_| "gemini-pro".to_string()),

            enable_mock_scraper: env::var("ENABLE_MOCK_SCRAPER")
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(false),

            // Application
            app_host: env::var("APP_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            app_port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("APP_PORT must be a number")?,

            // Worker
            worker_concurrency: env::var("WORKER_CONCURRENCY")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .context("WORKER_CONCURRENCY must be a number")?,
            worker_retry_count: env::var("WORKER_RETRY_COUNT")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .context("WORKER_RETRY_COUNT must be a number")?,

            // Polling
            polling_interval_ms: env::var("POLLING_INTERVAL_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .context("POLLING_INTERVAL_MS must be a number")?,
            price_cache_ttl_seconds: env::var("PRICE_CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .context("PRICE_CACHE_TTL_SECONDS must be a number")?,

            // Authentication — no fallback on purpose. A default signing key
            // would let anyone who has read the source mint a valid session
            // cookie, which makes the login screen decorative. Refusing to
            // boot is the safe failure.
            jwt_secret: read_jwt_secret()?,
            admin_password: env::var("ADMIN_PASSWORD").ok().filter(|p| !p.is_empty()),
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(false),
            allowed_origin: env::var("ALLOWED_ORIGIN").ok().filter(|o| !o.is_empty()),
        })
    }

    /// Minimal config for unit tests — every credential empty, so factories
    /// resolve to "not configured" unless a test opts one in explicitly.
    #[cfg(test)]
    pub fn test_default() -> Self {
        Self {
            database_url: String::new(),
            database_max_connections: 1,
            redis_url: String::new(),
            cache_ttl_seconds: 0,
            rabbitmq_url: String::new(),
            rabbitmq_queue_name: String::new(),
            serpapi_key: String::new(),
            serpapi_base_url: String::new(),
            gother_api_url: String::new(),
            gother_api_key: String::new(),
            gemini_api_key: None,
            gemini_model: String::new(),
            enable_mock_scraper: false,
            app_host: String::new(),
            app_port: 0,
            worker_concurrency: 1,
            worker_retry_count: 0,
            polling_interval_ms: 0,
            price_cache_ttl_seconds: 0,
            jwt_secret: "test-secret-that-is-long-enough-to-pass".to_string(),
            admin_password: None,
            cookie_secure: false,
            allowed_origin: None,
        }
    }
}

/// Minimum length for `JWT_SECRET`. HS256 keys shorter than the 256-bit hash
/// output add no security and are usually a placeholder someone meant to
/// replace.
const MIN_JWT_SECRET_LEN: usize = 32;

fn read_jwt_secret() -> Result<String> {
    let secret = env::var("JWT_SECRET").context(
        "JWT_SECRET must be set — it signs login session cookies. \
         Generate one with: openssl rand -base64 48",
    )?;
    if secret.trim().len() < MIN_JWT_SECRET_LEN {
        anyhow::bail!(
            "JWT_SECRET must be at least {} characters (got {})",
            MIN_JWT_SECRET_LEN,
            secret.trim().len()
        );
    }
    Ok(secret)
}
