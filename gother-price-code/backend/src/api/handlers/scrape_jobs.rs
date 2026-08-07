//! Scrape Jobs Handlers

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::db::{HotelRepo, ScrapeJobRepo, ScrapeResultRepo};
use crate::error::{AppError, AppResult};
use crate::excel::{ExcelReader, ExcelWriter};
use crate::models::{
    CreateScrapeJobRequest, JobHotelParamOverride, ScrapeJob, ScrapeJobMessage,
    ScrapeJobWithProgress, ScrapeResultsResponse,
};
use crate::queue;

fn build_message(job: &ScrapeJob, req: &CreateScrapeJobRequest) -> ScrapeJobMessage {
    ScrapeJobMessage {
        job_id: job.id,
        hotel_group_id: req.hotel_group_id,
        checkin_date: req.checkin_date,
        checkout_date: req.checkout_date,
        rooms: req.rooms,
        adults: req.adults,
        force_refresh: req.force_refresh,
        method: req.method,
        los_variants: req.los_variants.clone(),
        device: req.device,
        login_state: req.login_state,
    }
}

/// Create a scrape job, initialize per-hotel status rows, and publish it
/// to the queue. The single code path for "start a scrape job" — used by
/// both the HTTP handler below and the scheduler (worker/scheduler.rs) so
/// scheduled and on-demand jobs can never drift apart.
pub async fn create_and_publish_job(
    state: &Arc<AppState>,
    req: &CreateScrapeJobRequest,
) -> AppResult<ScrapeJob> {
    let job = ScrapeJobRepo::create(&state.db, req).await?;
    ScrapeJobRepo::init_hotel_statuses(&state.db, job.id, req.hotel_group_id).await?;

    let message = build_message(&job, req);

    queue::publish_job(&state.rabbitmq, &state.config.rabbitmq_queue_name, &message)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(job)
}

/// Create a new scrape job
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateScrapeJobRequest>,
) -> AppResult<Json<ScrapeJob>> {
    let job = create_and_publish_job(&state, &req).await?;
    Ok(Json(job))
}

/// Create a new scrape job with an optional per-hotel search-parameter
/// override sheet (REQ-001 F-002 JobDefaults fallback model). Multipart
/// fields: `job` (JSON body matching CreateScrapeJobRequest) and an
/// optional `overrides` file.
pub async fn create_job_with_overrides(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> AppResult<Json<ScrapeJob>> {
    let mut job_json: Option<String> = None;
    let mut overrides_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to parse form: {}", e)))?
    {
        match field.name() {
            Some("job") => {
                job_json = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::Validation(format!("Failed to read job: {}", e)))?,
                );
            }
            Some("overrides") => {
                overrides_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| {
                            AppError::Validation(format!("Failed to read overrides file: {}", e))
                        })?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let job_json = job_json.ok_or_else(|| AppError::Validation("Missing job field".to_string()))?;
    let req: CreateScrapeJobRequest = serde_json::from_str(&job_json)
        .map_err(|e| AppError::Validation(format!("Invalid job JSON: {}", e)))?;

    let job = ScrapeJobRepo::create(&state.db, &req).await?;
    ScrapeJobRepo::init_hotel_statuses(&state.db, job.id, req.hotel_group_id).await?;

    if let Some(data) = overrides_data {
        let overrides = ExcelReader::read_job_hotel_overrides(&data)?;
        let mut resolved: Vec<(Uuid, JobHotelParamOverride)> = Vec::new();

        for o in overrides {
            let hotel = if let Some(hid) = o.hid {
                HotelRepo::get_by_hid(&state.db, hid).await?
            } else {
                None
            };
            if let Some(hotel) = hotel {
                resolved.push((hotel.id, o));
            }
            // Rows keyed only by hotel_name (no hid) are not resolved here —
            // name matching across duplicates is ambiguous; hid is the
            // reliable key for the master hotel list.
        }

        ScrapeJobRepo::insert_hotel_param_overrides(&state.db, job.id, &resolved).await?;
    }

    let message = build_message(&job, &req);

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
