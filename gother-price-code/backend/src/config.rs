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

    // OpenAI / ChatGPT scraper (Method 1, optional/bonus)
    pub openai_api_key: Option<String>,
    pub openai_model: String,

    // Application
    pub app_host: String,
    pub app_port: u16,

    // Worker
    pub worker_concurrency: usize,
    pub worker_retry_count: u32,

    // Polling
    pub polling_interval_ms: u64,
    pub price_cache_ttl_seconds: u64,
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

            // OpenAI / ChatGPT scraper
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            openai_model: env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),

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
        })
    }
}
