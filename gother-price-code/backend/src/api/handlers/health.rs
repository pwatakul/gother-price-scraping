//! Health Check Handler

use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::api::router::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub redis: String,
    pub rabbitmq: String,
}

/// Health check endpoint
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    // Check database
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    // Check Redis
    let mut redis_conn = state.redis.clone();
    let redis_status = match redis::cmd("PING")
        .query_async::<_, String>(&mut redis_conn)
        .await
    {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    // Check RabbitMQ
    let rabbitmq_status = if state.rabbitmq.status().connected() {
        "connected"
    } else {
        "disconnected"
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
        redis: redis_status.to_string(),
        rabbitmq: rabbitmq_status.to_string(),
    })
}
