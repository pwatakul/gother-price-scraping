//! Hotel Directory Handlers (REQ-007 — global "All Hotels" page)

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::db::{HotelDirectoryRepo, HotelRepo, PriceHistoryRepo};
use crate::error::{AppError, AppResult};
use crate::models::{Hotel, HotelDetail, HotelListQuery, HotelListResponse, UpdateHotelRequest};

/// GET /hotels — paginated, filtered, cross-group hotel listing
pub async fn list_hotels(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HotelListQuery>,
) -> AppResult<Json<HotelListResponse>> {
    let (hotels, total) = HotelDirectoryRepo::list(&state.db, &query).await?;
    Ok(Json(HotelListResponse { hotels, total }))
}

pub async fn list_countries(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
    let countries = HotelDirectoryRepo::distinct_countries(&state.db).await?;
    Ok(Json(countries))
}

#[derive(Deserialize)]
pub struct CityQuery {
    pub country: Option<String>,
}

pub async fn list_cities(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CityQuery>,
) -> AppResult<Json<Vec<String>>> {
    let cities = HotelDirectoryRepo::distinct_cities(&state.db, query.country.as_deref()).await?;
    Ok(Json(cities))
}

/// GET /hotels/:id — full tracking detail for one hotel
pub async fn get_hotel_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<HotelDetail>> {
    let hotel = HotelRepo::get_by_id(&state.db, id)
        .await
        .map_err(|_| AppError::NotFound(format!("Hotel {} not found", id)))?;
    let group_names = HotelDirectoryRepo::group_names_for_hotel(&state.db, id).await?;
    // Default to the window with the most data so the detail page opens on
    // a like-for-like comparison rather than a mixed average (ADR-013).
    let windows = PriceHistoryRepo::trend_windows_for_hotel(&state.db, id).await?;
    let default_window = windows.first().map(|w| w.days_in_advance);
    let trend =
        PriceHistoryRepo::trend_for_hotel(&state.db, id, None, 90, default_window).await?;

    Ok(Json(HotelDetail { hotel, group_names, trend }))
}

/// GET /hotels/export — same filters as the list endpoint, streams CSV
pub async fn export_hotels(
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<HotelListQuery>,
) -> AppResult<Response<Body>> {
    query.limit = 10_000; // export everything matching the filter, not one page
    query.offset = 0;
    let (hotels, _) = HotelDirectoryRepo::list(&state.db, &query).await?;

    let mut csv = String::from("name,city,country,hid,groups,last_price_thb,last_price_source,last_scraped_at\n");
    for h in &hotels {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            h.name.replace(',', " "),
            h.city,
            h.country,
            h.hid.map(|v| v.to_string()).unwrap_or_default(),
            h.group_names.join(" | ").replace(',', " "),
            h.last_price_thb.map(|p| p.to_string()).unwrap_or_default(),
            h.last_price_source.clone().unwrap_or_default(),
            h.last_scraped_at.map(|t| t.to_string()).unwrap_or_default(),
        ));
    }

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/csv")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"hotels.csv\"")
        .body(Body::from(csv))
        .unwrap())
}

/// PUT /hotels/:id — edit a hotel's name, city or country.
///
/// Renaming changes what the scraper searches for (the SerpAPI query is
/// built from name + city + country), so a vague name will quietly stop
/// matching. `normalized_name` is recomputed by the repo.
pub async fn update_hotel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHotelRequest>,
) -> AppResult<Json<Hotel>> {
    let hotel = HotelRepo::update(&state.db, id, &req).await?;
    Ok(Json(hotel))
}
