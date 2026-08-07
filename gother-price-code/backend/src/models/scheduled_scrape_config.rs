//! Scheduled Scrape Config Model (REQ-002 F-003/F-004)

use crate::models::scrape_job::ScrapeMethod;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledScrapeConfig {
    pub id: Uuid,
    pub hotel_group_id: Uuid,
    pub name: Option<String>,
    pub cron_expression: String,
    pub lookahead_days: Vec<i32>,
    pub los_variants: Vec<i32>,
    pub method: ScrapeMethod,
    pub rooms: i16,
    pub adults: i16,
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
    pub lookahead_days: Vec<i32>,
    #[serde(default = "default_los_variants")]
    pub los_variants: Vec<i32>,
    #[serde(default)]
    pub method: ScrapeMethod,
    #[serde(default = "default_rooms")]
    pub rooms: i16,
    #[serde(default = "default_adults")]
    pub adults: i16,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_los_variants() -> Vec<i32> {
    vec![1]
}
fn default_rooms() -> i16 {
    1
}
fn default_adults() -> i16 {
    2
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduledScrapeConfigRequest {
    pub name: Option<String>,
    pub cron_expression: Option<String>,
    pub lookahead_days: Option<Vec<i32>>,
    pub los_variants: Option<Vec<i32>>,
    pub method: Option<ScrapeMethod>,
    pub rooms: Option<i16>,
    pub adults: Option<i16>,
    pub is_active: Option<bool>,
}
