//! Scheduled Scrape Config Model (REQ-002 F-003/F-004)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Note: the booking windows, devices and stay parameters a run uses are
/// system constants (see `worker::scheduler` and ADR-006), not fields here
/// — a schedule configures only *when* and *how* to scrape, never *what*.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledScrapeConfig {
    pub id: Uuid,
    pub hotel_group_id: Uuid,
    pub name: Option<String>,
    pub cron_expression: String,
    pub is_active: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduledScrapeConfigRequest {
    pub hotel_group_id: Uuid,
    pub name: Option<String>,
    pub cron_expression: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduledScrapeConfigRequest {
    pub name: Option<String>,
    pub cron_expression: Option<String>,
    pub is_active: Option<bool>,
}
