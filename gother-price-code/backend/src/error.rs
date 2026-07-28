//! Error Module
//!
//! Custom error types for the application.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Queue error: {0}")]
    Queue(#[from] lapin::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Excel error: {0}")]
    Excel(String),

    #[error("Scraper error: {0}")]
    Scraper(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Error response structure
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {}", e),
            ),
            AppError::Redis(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CACHE_ERROR",
                format!("Cache error: {}", e),
            ),
            AppError::Queue(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "QUEUE_ERROR",
                format!("Queue error: {}", e),
            ),
            AppError::HttpClient(e) => (
                StatusCode::BAD_GATEWAY,
                "EXTERNAL_API_ERROR",
                format!("External API error: {}", e),
            ),
            AppError::Excel(msg) => (StatusCode::BAD_REQUEST, "EXCEL_ERROR", msg.clone()),
            AppError::Scraper(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SCRAPER_ERROR",
                msg.clone(),
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
        };

        let body = Json(ErrorResponse {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                details: None,
            },
        });

        (status, body).into_response()
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
