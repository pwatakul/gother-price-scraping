//! Price History Handlers (REQ-002 F-007/F-008)

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::db::PriceHistoryRepo;
use crate::error::AppResult;
use crate::models::{PriceHistoryListResponse, PriceHistoryQuery, PriceTrendPoint};

/// GET /price-history — filtered, paginated raw history rows (used by the
/// full price-history table on the hotel detail page).
pub async fn list_price_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PriceHistoryQuery>,
) -> AppResult<Json<PriceHistoryListResponse>> {
    let rows = PriceHistoryRepo::query(&state.db, &query).await?;
    let total = PriceHistoryRepo::count(&state.db, &query).await?;
    Ok(Json(PriceHistoryListResponse { rows, total }))
}

#[derive(serde::Deserialize)]
pub struct TrendQuery {
    pub source: Option<String>,
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    90
}

/// GET /price-history/hotel/:id/trend
pub async fn hotel_trend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TrendQuery>,
) -> AppResult<Json<Vec<PriceTrendPoint>>> {
    let points =
        PriceHistoryRepo::trend_for_hotel(&state.db, id, query.source.as_deref(), query.days).await?;
    Ok(Json(points))
}

#[derive(serde::Deserialize)]
pub struct ExportQuery {
    #[serde(flatten)]
    pub filters: PriceHistoryQuery,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "csv".to_string()
}

/// GET /export/price-history — REQ-005 F-006, raw history export for
/// external BI/analyst use.
pub async fn export_price_history(
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ExportQuery>,
) -> AppResult<Response<Body>> {
    // Export means "everything matching the filter", not one paginated
    // page — override whatever limit/offset came in (list_price_history's
    // default of 100 would otherwise silently truncate a real export).
    query.filters.limit = 100_000;
    query.filters.offset = 0;
    let rows = PriceHistoryRepo::query(&state.db, &query.filters).await?;

    if query.format == "json" {
        let body = serde_json::to_vec(&rows).unwrap_or_default();
        return Ok(Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_DISPOSITION, "attachment; filename=\"price-history.json\"")
            .body(Body::from(body))
            .unwrap());
    }

    let mut csv = String::from(
        "hotel_id,source,room_type,price_thb,original_price,currency,checkin_date,checkout_date,rooms,adults,scraped_at\n",
    );
    for r in &rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            r.hotel_id,
            r.source,
            r.room_type.replace(',', " "),
            r.price_thb,
            r.original_price.map(|p| p.to_string()).unwrap_or_default(),
            r.currency.clone().unwrap_or_default(),
            r.checkin_date,
            r.checkout_date,
            r.rooms,
            r.adults,
            r.scraped_at,
        ));
    }

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/csv")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"price-history.csv\"")
        .body(Body::from(csv))
        .unwrap())
}
