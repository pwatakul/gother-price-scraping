//! Analytics Handlers (REQ-003)
//!
//! All endpoints read from the materialized views created in migration
//! 016 — no live scrape is ever triggered by viewing analytics.

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
use crate::db::MaterializedViewRepo;
use crate::error::AppResult;
use crate::models::{
    BookingWindowRow, HeatmapCell, MarketOverview, MarketPositionEntry, ParityViolationRow,
    ProviderBenchmarkRow, WinRateRow,
};

#[derive(Deserialize)]
pub struct GroupFilter {
    pub hotel_group_id: Option<Uuid>,
}

/// GET /analytics/overview — F-001
pub async fn overview(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<GroupFilter>,
) -> AppResult<Json<MarketOverview>> {
    let overview = MaterializedViewRepo::overview(&state.db, filter.hotel_group_id).await?;
    Ok(Json(overview))
}

/// GET /analytics/market-position — F-003
pub async fn market_position(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<GroupFilter>,
) -> AppResult<Json<Vec<MarketPositionEntry>>> {
    let entries = MaterializedViewRepo::position_table(&state.db, filter.hotel_group_id).await?;
    Ok(Json(entries))
}

/// GET /analytics/heatmap — F-004
pub async fn heatmap(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<GroupFilter>,
) -> AppResult<Json<Vec<HeatmapCell>>> {
    let cells = MaterializedViewRepo::heatmap(&state.db, filter.hotel_group_id).await?;
    Ok(Json(cells))
}

/// GET /analytics/win-rate — F-005
pub async fn win_rate(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<WinRateRow>>> {
    let rows = MaterializedViewRepo::win_rate(&state.db).await?;
    Ok(Json(rows))
}

/// GET /analytics/provider-benchmark — who is cheapest, and by how much.
/// Gother-independent, so it works before the Gother API is connected.
pub async fn provider_benchmark(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<GroupFilter>,
) -> AppResult<Json<Vec<ProviderBenchmarkRow>>> {
    let rows = MaterializedViewRepo::provider_benchmark(&state.db, filter.hotel_group_id).await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct ParityQuery {
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

fn default_threshold() -> f64 {
    5.0
}

/// GET /analytics/parity-violations — F-013
pub async fn parity_violations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ParityQuery>,
) -> AppResult<Json<Vec<ParityViolationRow>>> {
    let rows = MaterializedViewRepo::parity_violations(&state.db, query.threshold).await?;
    Ok(Json(rows))
}

/// GET /analytics/booking-window/:hotel_id — F-014
pub async fn booking_window(
    State(state): State<Arc<AppState>>,
    Path(hotel_id): Path<Uuid>,
) -> AppResult<Json<Vec<BookingWindowRow>>> {
    let rows = MaterializedViewRepo::booking_window(&state.db, hotel_id).await?;
    Ok(Json(rows))
}

/// GET /analytics/export — F-009, Excel download of the current market
/// position table.
pub async fn export(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<GroupFilter>,
) -> AppResult<Response<Body>> {
    let entries = MaterializedViewRepo::position_table(&state.db, filter.hotel_group_id).await?;

    let mut csv = String::from("hotel_name,gother_price,best_price,best_source,gap_thb,gap_pct,is_winning\n");
    for e in &entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            e.hotel_name.replace(',', " "),
            e.gother_price.map(|p| p.to_string()).unwrap_or_default(),
            e.best_price.map(|p| p.to_string()).unwrap_or_default(),
            e.best_source.clone().unwrap_or_default(),
            e.gap_thb.map(|p| p.to_string()).unwrap_or_default(),
            e.gap_pct.map(|p| p.to_string()).unwrap_or_default(),
            e.is_winning,
        ));
    }

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/csv")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"market-position.csv\"")
        .body(Body::from(csv))
        .unwrap())
}
