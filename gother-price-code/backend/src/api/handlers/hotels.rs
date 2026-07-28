//! Hotels Handlers

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::router::AppState;
use crate::db::HotelRepo;
use crate::error::AppResult;
use crate::models::Hotel;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    10
}

/// Search hotels
pub async fn search_hotels(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<Hotel>>> {
    let hotels = HotelRepo::search(&state.db, &query.q, query.limit).await?;
    Ok(Json(hotels))
}
