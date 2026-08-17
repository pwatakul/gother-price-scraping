//! Scheduled Scrape Config Handlers (REQ-002 F-003/F-004)

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::db::ScheduledScrapeConfigRepo;
use crate::error::AppResult;
use crate::models::{
    CreateScheduledScrapeConfigRequest, ScheduledScrapeConfig, UpdateScheduledScrapeConfigRequest,
};
use crate::worker;

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

/// POST /scheduled-scrape-configs/:id/run — REQ-008 F-010. Fires the
/// standard grid immediately, without waiting for the next cron tick.
///
/// A manual run is *additive* to the schedule: `fire_grid` deliberately
/// leaves `last_run_at`/`next_run_at` alone, so pressing this never delays
/// the next scheduled fire. Runs regardless of `is_active` — pausing stops
/// the cron, but an explicit request is still an explicit request.
pub async fn run_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let config = ScheduledScrapeConfigRepo::get_by_id(&state.db, id).await?;
    let jobs_queued = worker::scheduler::fire_grid(&state, &config, Utc::now()).await;

    Ok(Json(serde_json::json!({ "jobs_queued": jobs_queued })))
}

pub async fn delete_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    ScheduledScrapeConfigRepo::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
