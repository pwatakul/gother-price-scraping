//! Scheduled Scrape Config Scheduler (REQ-002 F-005)
//!
//! A background loop, separate from the RabbitMQ consumer, that ticks
//! every `TICK_INTERVAL_SECS` and fires scrape jobs for any active
//! `scheduled_scrape_configs` whose cron expression is due. Reuses the
//! exact same job-creation path (`create_and_publish_job`) the HTTP API
//! uses, so scheduled and on-demand jobs never diverge in behavior.

use crate::api::handlers::scrape_jobs::create_and_publish_job;
use crate::api::AppState;
use crate::db::ScheduledScrapeConfigRepo;
use crate::models::CreateScrapeJobRequest;
use chrono::{DateTime, Utc};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

const TICK_INTERVAL_SECS: u64 = 60;

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

        for lookahead in &config.lookahead_days {
            let checkin_date = now.date_naive() + chrono::Duration::days(*lookahead as i64);
            let checkout_date = checkin_date + chrono::Duration::days(1);

            let req = CreateScrapeJobRequest {
                hotel_group_id: config.hotel_group_id,
                checkin_date,
                checkout_date,
                rooms: config.rooms as i32,
                adults: config.adults as i32,
                force_refresh: false,
                method: config.method,
                los_variants: config.los_variants.clone(),
                device: Default::default(),
                login_state: Default::default(),
            };

            match create_and_publish_job(state, &req).await {
                Ok(job) => tracing::info!(
                    "Scheduled job {} created for config {} (lookahead {}d)",
                    job.id,
                    config.id,
                    lookahead
                ),
                Err(e) => {
                    tracing::warn!("Failed to create scheduled job for config {}: {}", config.id, e)
                }
            }
        }

        let next_run_at = next_run(&config.cron_expression, now);
        if let Err(e) = ScheduledScrapeConfigRepo::mark_run(&state.db, config.id, now, next_run_at).await
        {
            tracing::warn!("Failed to update last_run_at for config {}: {}", config.id, e);
        }
    }

    Ok(())
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
}
