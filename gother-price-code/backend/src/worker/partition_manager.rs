//! Partition Manager (REQ-005 F-002)
//!
//! Closes the gap flagged in REQ-005-v1.1: migration `014` only created
//! hotel_price_history partitions for the current month + next 3 months
//! once, at migration time. This loop keeps that rolling 4-month window
//! topped up going forward — application-level, idempotent, no
//! `pg_partman` dependency (REQ-005 explicitly decided against adding
//! that extension for this pass).

use crate::api::AppState;
use crate::db::PriceHistoryRepo;
use std::sync::Arc;
use std::time::Duration;

const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
const MONTHS_AHEAD: i32 = 4;

pub async fn run(state: Arc<AppState>) {
    // tokio::time::interval's first tick() fires immediately, so
    // partitions are ensured once at startup, then once daily after.
    let mut interval = tokio::time::interval(Duration::from_secs(CHECK_INTERVAL_SECS));
    loop {
        interval.tick().await;
        match PriceHistoryRepo::ensure_future_partitions(&state.db, MONTHS_AHEAD).await {
            Ok(()) => tracing::debug!("hotel_price_history partitions ensured through +{} months", MONTHS_AHEAD),
            Err(e) => tracing::warn!("Failed to ensure hotel_price_history partitions: {}", e),
        }
    }
}
