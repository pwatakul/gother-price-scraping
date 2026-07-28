//! Scrape Jobs Handlers

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::db::{ScrapeJobRepo, ScrapeResultRepo};
use crate::error::{AppError, AppResult};
use crate::excel::ExcelWriter;
use crate::models::{
    CreateScrapeJobRequest, ScrapeJob, ScrapeJobMessage, ScrapeJobWithProgress,
    ScrapeResultsResponse,
};
use crate::queue;

/// Create a new scrape job
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateScrapeJobRequest>,
) -> AppResult<Json<ScrapeJob>> {
    // Create job in database
    let job = ScrapeJobRepo::create(&state.db, &req).await?;

    // Initialize hotel statuses
    ScrapeJobRepo::init_hotel_statuses(&state.db, job.id, req.hotel_group_id).await?;

    // Create message for queue
    let message = ScrapeJobMessage {
        job_id: job.id,
        hotel_group_id: req.hotel_group_id,
        checkin_date: req.checkin_date,
        checkout_date: req.checkout_date,
        rooms: req.rooms,
        adults: req.adults,
        force_refresh: req.force_refresh,
    };

    // Publish to RabbitMQ
    queue::publish_job(&state.rabbitmq, &state.config.rabbitmq_queue_name, &message)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(job))
}

/// Get job status with progress
pub async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ScrapeJobWithProgress>> {
    let job = ScrapeJobRepo::get_with_progress(&state.db, id).await?;
    Ok(Json(job))
}

/// Cancel a running job
pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ScrapeJob>> {
    let job = ScrapeJobRepo::cancel(&state.db, id).await?;
    Ok(Json(job))
}

/// Get scrape results
pub async fn get_results(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ScrapeResultsResponse>> {
    let results = ScrapeResultRepo::get_job_results(&state.db, id).await?;
    Ok(Json(results))
}

/// Export results as Excel
pub async fn export_excel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Response<Body>> {
    let results = ScrapeResultRepo::get_job_results(&state.db, id).await?;
    let excel_data = ExcelWriter::write_results(&results)?;

    let filename = format!("hotel-price-report-{}.xlsx", id);

    let response = Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(excel_data))
        .unwrap();

    Ok(response)
}
