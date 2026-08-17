//! Scheduled Scrape Config Scheduler (REQ-002 F-005, REQ-008 F-001–F-004)
//!
//! A background loop, separate from the RabbitMQ consumer, that ticks
//! every `TICK_INTERVAL_SECS` and fires scrape jobs for any active
//! `scheduled_scrape_configs` whose cron expression is due. Reuses the
//! exact same job-creation path (`create_and_publish_job`) the HTTP API
//! uses, so scheduled and on-demand jobs never diverge in behavior.
//!
//! Every fire expands to the standard grid — 5 booking windows at fixed
//! stay parameters. The grid is a constant rather than per-config input so
//! that every hotel's series shares one x-axis; see ADR-006. The config's
//! own `lookahead_days`/`los_variants` columns are deliberately not read.

use crate::api::handlers::scrape_jobs::queue_window_jobs as create_and_publish_window_jobs;
use crate::api::AppState;
use crate::db::ScheduledScrapeConfigRepo;
use crate::models::ScheduledScrapeConfig;
use chrono::{DateTime, Utc};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

const TICK_INTERVAL_SECS: u64 = 60;

/// REQ-008-v1.1 F-001 — days before check-in, fixed for every hotel.
pub const STANDARD_BOOKING_WINDOWS: [i64; 5] = [1, 3, 7, 14, 30];
/// REQ-008-v1.1 F-002/F-003 — desktop/public only, and fixed stay params
/// so every observation is comparable. The device and login-state axes are
/// recorded but never varied: SerpAPI exposes no parameter for either, and
/// a desktop-vs-mobile comparison over 69 sources found zero price
/// differences, so varying them doubled cost to duplicate rows. See ADR-010.
const STANDARD_ROOMS: i32 = 1;
const STANDARD_ADULTS: i32 = 2;

/// The booking windows one fire expands to. Pure so the grid can be
/// asserted in tests without a DB or scheduler running.
pub fn standard_grid() -> Vec<i64> {
    STANDARD_BOOKING_WINDOWS.to_vec()
}

pub async fn run(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(TICK_INTERVAL_SECS));
    loop {
        interval.tick().await;
        if let Err(e) = tick(&state).await {
            tracing::warn!("Scheduler tick failed: {}", e);
        }
    }
}

async fn tick(state: &Arc<AppState>) -> anyhow::Result<()> {
    let configs = ScheduledScrapeConfigRepo::list_active(&state.db).await?;
    let now = Utc::now();

    for config in configs {
        if !is_due(&config.cron_expression, config.last_run_at, now) {
            continue;
        }

        tracing::info!("Scheduled config {} is due, firing scrape job(s)", config.id);

        fire_grid(state, &config, now).await;

        let next_run_at = next_run(&config.cron_expression, now);
        if let Err(e) = ScheduledScrapeConfigRepo::mark_run(&state.db, config.id, now, next_run_at).await
        {
            tracing::warn!("Failed to update last_run_at for config {}: {}", config.id, e);
        }
    }

    Ok(())
}

/// Queue one scrape job per cell of the standard grid, relative to `now`,
/// and return how many were queued successfully. Each cell fails
/// independently — one bad job must not cost the other nine.
///
/// Shared by the cron tick and the manual-run endpoint so a scheduled run
/// and a "Run now" produce byte-identical job sets. Deliberately does NOT
/// touch `last_run_at`/`next_run_at`: that bookkeeping belongs to the cron
/// path only, and writing it here would let a manual run silently push out
/// the next scheduled fire (see `is_due`, which measures from last_run_at).
pub async fn fire_grid(
    state: &Arc<AppState>,
    config: &ScheduledScrapeConfig,
    now: DateTime<Utc>,
) -> usize {
    // The scraper method comes from the group's saved search config — one
    // place to set it for both manual and scheduled runs (ADR-012). The
    // stay parameters deliberately do *not*: the grid stays a system
    // constant so every hotel's history shares one x-axis (ADR-006).
    let method = match crate::db::HotelGroupRepo::get_by_id(&state.db, config.hotel_group_id).await
    {
        Ok(group) => group.search_method,
        Err(e) => {
            tracing::warn!(
                "Cannot read search config for group {} ({}); skipping this fire",
                config.hotel_group_id,
                e
            );
            return 0;
        }
    };

    // One job per standard window, via the shared path the manual run
    // also uses — the only difference is which windows are passed.
    let windows: Vec<i32> = standard_grid().iter().map(|w| *w as i32).collect();
    let jobs = create_and_publish_window_jobs(
        state,
        config.hotel_group_id,
        method,
        &windows,
        STANDARD_ROOMS,
        STANDARD_ADULTS,
        now,
        false, // scheduled runs are happy with cached rows
    )
    .await;

    tracing::info!(
        "Scheduled config {} queued {} of {} window jobs",
        config.id,
        jobs.len(),
        windows.len()
    );

    jobs.len()
}

/// The `cron` crate requires a leading seconds field (6-7 fields);
/// REQ-002's examples use standard 5-field cron (`"0 2 * * *"`). Accept
/// both by prepending a `0` seconds field when only 5 are given.
fn normalize(cron_expr: &str) -> String {
    let field_count = cron_expr.split_whitespace().count();
    if field_count == 5 {
        format!("0 {cron_expr}")
    } else {
        cron_expr.to_string()
    }
}

/// Pure function (no DB access) so it's unit-testable: is this cron
/// expression due to run, given when it last ran? A config that has
/// never run is due immediately (fires on the next tick after creation).
pub fn is_due(cron_expr: &str, last_run_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    let Ok(schedule) = cron::Schedule::from_str(&normalize(cron_expr)) else {
        tracing::warn!("Invalid cron expression: {}", cron_expr);
        return false;
    };

    let Some(last_run_at) = last_run_at else {
        return true;
    };

    // Due if the schedule has a fire time between last_run_at and now.
    schedule.after(&last_run_at).take(1).next().map(|t| t <= now).unwrap_or(false)
}

fn next_run(cron_expr: &str, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
    cron::Schedule::from_str(&normalize(cron_expr)).ok().and_then(|s| s.after(&after).take(1).next())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_run_is_always_due() {
        assert!(is_due("0 2 * * *", None, Utc::now()));
    }

    #[test]
    fn not_due_immediately_after_running() {
        let now = Utc::now();
        // Daily at 2am; just ran, shouldn't be due again seconds later.
        assert!(!is_due("0 2 * * *", Some(now), now + chrono::Duration::seconds(5)));
    }

    #[test]
    fn due_after_a_full_day_has_passed() {
        let last_run = Utc::now() - chrono::Duration::days(2);
        assert!(is_due("0 2 * * *", Some(last_run), Utc::now()));
    }

    #[test]
    fn invalid_expression_is_never_due() {
        assert!(!is_due("not a cron expression", None, Utc::now()));
    }

    #[test]
    fn grid_is_one_job_per_standard_booking_window() {
        let grid = standard_grid();

        // One job per window — the device axis was removed in REQ-008-v1.1
        // after it was shown to duplicate rows rather than vary them.
        assert_eq!(grid.len(), 5);

        // REQ-008 F-009: no duplicate window within a single fire.
        let unique: std::collections::HashSet<_> = grid.iter().collect();
        assert_eq!(unique.len(), grid.len());

        for window in STANDARD_BOOKING_WINDOWS {
            assert!(grid.contains(&window), "missing +{window}d");
        }
    }
}
