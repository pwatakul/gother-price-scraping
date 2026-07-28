//! Hotel Groups Handlers

use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::db::{HotelGroupRepo, HotelRepo, ScrapeJobRepo};
use crate::error::AppResult;
use crate::excel::ExcelReader;
use crate::models::{
    CreateHotelGroupRequest, CreateHotelRequest, Hotel, HotelGroup, HotelGroupWithCount,
    HotelWithPrice, ScrapeJob, UpdateHotelGroupRequest,
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
) -> AppResult<Json<Vec<ScrapeJob>>> {
    let jobs = ScrapeJobRepo::list_by_group(&state.db, id, query.limit, query.offset).await?;
    Ok(Json(jobs))
}
