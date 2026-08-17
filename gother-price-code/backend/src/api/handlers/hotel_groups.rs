//! Hotel Groups Handlers

use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use chrono::Utc;

use crate::api::handlers::scrape_jobs::queue_window_jobs;
use crate::api::router::AppState;
use crate::db::{HotelGroupRepo, HotelRepo, ScrapeJobRepo};
use crate::error::AppResult;
use crate::excel::ExcelReader;
use crate::models::{
    CreateHotelGroupRequest, CreateHotelRequest, CreateScrapeJobRequest, Hotel, HotelGroup,
    HotelGroupWithCount, HotelWithPrice, ScrapeJob, UpdateGroupSearchConfigRequest,
    UpdateHotelGroupRequest,
};

/// List all hotel groups
pub async fn list_groups(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<HotelGroupWithCount>>> {
    let groups = HotelGroupRepo::list_all(&state.db).await?;
    Ok(Json(groups))
}

/// Create a new hotel group
pub async fn create_group(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> AppResult<Json<HotelGroup>> {
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut excel_data: Option<Vec<u8>> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to parse form: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "name" => {
                name = Some(field.text().await.map_err(|e| {
                    crate::error::AppError::Validation(format!("Failed to read name: {}", e))
                })?);
            }
            "description" => {
                description = Some(field.text().await.map_err(|e| {
                    crate::error::AppError::Validation(format!("Failed to read description: {}", e))
                })?);
            }
            "file" => {
                excel_data = Some(field.bytes().await.map_err(|e| {
                    crate::error::AppError::Validation(format!("Failed to read file: {}", e))
                })?.to_vec());
            }
            _ => {}
        }
    }

    let name = name.ok_or_else(|| {
        crate::error::AppError::Validation("Name is required".to_string())
    })?;

    // Create the group
    let req = CreateHotelGroupRequest { name, description };
    let group = HotelGroupRepo::create(&state.db, &req).await?;

    // If Excel file provided, import hotels
    if let Some(data) = excel_data {
        let hotels = ExcelReader::read_hotels(&data)?;
        
        for hotel_data in hotels {
            let hotel = HotelRepo::find_or_create(
                &state.db,
                &hotel_data.hotel_name,
                &hotel_data.city,
                &hotel_data.country,
            )
            .await?;
            
            HotelGroupRepo::add_hotel(&state.db, group.id, hotel.id).await?;
        }
    }

    Ok(Json(group))
}

/// Get hotel group details
#[derive(Deserialize)]
pub struct GetGroupResponse {
    pub group: HotelGroup,
    pub hotels: Vec<HotelWithPrice>,
}

pub async fn get_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let group = HotelGroupRepo::get_by_id(&state.db, id).await?;
    let hotels = HotelRepo::get_by_group_id(&state.db, id).await?;

    Ok(Json(serde_json::json!({
        "group": group,
        "hotels": hotels
    })))
}

/// Update hotel group
pub async fn update_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHotelGroupRequest>,
) -> AppResult<Json<HotelGroup>> {
    let group = HotelGroupRepo::update(&state.db, id, &req).await?;
    Ok(Json(group))
}

/// PUT /hotel-groups/:id/search-config — edit the saved price search.
pub async fn update_search_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGroupSearchConfigRequest>,
) -> AppResult<Json<HotelGroup>> {
    let group = HotelGroupRepo::update_search_config(&state.db, id, &req).await?;
    Ok(Json(group))
}

/// POST /hotel-groups/:id/search-runs — run the saved search now.
///
/// Queues one job per configured booking window; returns how many were
/// queued rather than a single job, since a run is now a set.
pub async fn run_saved_search(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let group = HotelGroupRepo::get_by_id(&state.db, id).await?;

    // One job per configured booking window, at one night each — the same
    // shared path the scheduler uses, so a manual run and a scheduled run
    // produce identical job shapes for the same window (ADR-012/ADR-013).
    // force_refresh: an explicit press means "get me current prices".
    let jobs = queue_window_jobs(
        &state,
        group.id,
        group.search_method,
        &group.search_days_ahead,
        group.search_rooms as i32,
        group.search_adults as i32,
        Utc::now(),
        true,
    )
    .await;

    let job_ids: Vec<Uuid> = jobs.iter().map(|j| j.id).collect();
    Ok(Json(serde_json::json!({
        "jobs_queued": jobs.len(),
        "job_ids": job_ids,
    })))
}

/// Delete hotel group
pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    HotelGroupRepo::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// Import hotels from Excel
pub async fn import_hotels(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // Verify group exists
    let _ = HotelGroupRepo::get_by_id(&state.db, id).await?;

    let mut imported_count = 0;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to parse form: {}", e))
    })? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| {
                crate::error::AppError::Validation(format!("Failed to read file: {}", e))
            })?;

            let hotels = ExcelReader::read_hotels(&data)?;
            
            for hotel_data in hotels {
                let hotel = HotelRepo::find_or_create(
                    &state.db,
                    &hotel_data.hotel_name,
                    &hotel_data.city,
                    &hotel_data.country,
                )
                .await?;
                
                HotelGroupRepo::add_hotel(&state.db, id, hotel.id).await?;
                imported_count += 1;
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "imported_count": imported_count
    })))
}

/// Import hotels from the real master hotel-list format (REQ-001-v1.2
/// F-021), separate from the plain hotel_name/city/country `import_hotels`
/// endpoint above so neither import path risks the other.
pub async fn import_master_hotels(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let _ = HotelGroupRepo::get_by_id(&state.db, id).await?;

    let mut imported_count = 0;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to parse form: {}", e))
    })? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| {
                crate::error::AppError::Validation(format!("Failed to read file: {}", e))
            })?;

            let rows = ExcelReader::read_master_hotel_list(&data)?;

            for row in rows {
                let hotel = HotelRepo::find_or_create_by_hid(&state.db, &row).await?;
                HotelGroupRepo::add_hotel(&state.db, id, hotel.id).await?;
                imported_count += 1;
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "imported_count": imported_count
    })))
}

/// Add a single hotel to group
pub async fn add_hotel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateHotelRequest>,
) -> AppResult<Json<Hotel>> {
    // Verify group exists
    let _ = HotelGroupRepo::get_by_id(&state.db, id).await?;

    // Find or create hotel
    let hotel = HotelRepo::find_or_create(&state.db, &req.name, &req.city, &req.country).await?;

    // Add to group
    HotelGroupRepo::add_hotel(&state.db, id, hotel.id).await?;

    Ok(Json(hotel))
}

/// Remove hotel from group
pub async fn remove_hotel(
    State(state): State<Arc<AppState>>,
    Path((group_id, hotel_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    HotelGroupRepo::remove_hotel(&state.db, group_id, hotel_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// List scrape jobs for a group
#[derive(Deserialize)]
pub struct ListJobsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListJobsQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let jobs = ScrapeJobRepo::list_by_group(&state.db, id, query.limit, query.offset).await?;
    // `total` is the unpaginated count, so the UI can show page numbers
    // instead of guessing whether another page exists.
    let total = ScrapeJobRepo::count_by_group(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "jobs": jobs, "total": total })))
}
