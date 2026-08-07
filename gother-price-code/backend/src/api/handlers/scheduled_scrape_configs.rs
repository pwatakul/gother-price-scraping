//! Scheduled Scrape Config Handlers (REQ-002 F-003/F-004)

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::db::ScheduledScrapeConfigRepo;
use crate::error::AppResult;
use crate::models::{
    CreateScheduledScrapeConfigRequest, ScheduledScrapeConfig, UpdateScheduledScrapeConfigRequest,
};

pub async fn create_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateScheduledScrapeConfigRequest>,
) -> AppResult<Json<ScheduledScrapeConfig>> {
    let config = ScheduledScrapeConfigRepo::create(&state.db, &req).await?;
    Ok(Json(config))
}

#[derive(Deserialize)]
pub struct ListConfigsQuery {
    pub hotel_group_id: Uuid,
}

pub async fn list_configs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListConfigsQuery>,
) -> AppResult<Json<Vec<ScheduledScrapeConfig>>> {
    let configs = ScheduledScrapeConfigRepo::list_by_group(&state.db, query.hotel_group_id).await?;
    Ok(Json(configs))
}

pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateScheduledScrapeConfigRequest>,
) -> AppResult<Json<ScheduledScrapeConfig>> {
    let config = ScheduledScrapeConfigRepo::update(&state.db, id, &req).await?;
    Ok(Json(config))
}

pub async fn delete_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    ScheduledScrapeConfigRepo::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
