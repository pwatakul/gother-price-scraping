//! Scrape Job Model

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Job status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "scrape_job_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ScrapeJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for ScrapeJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeJobStatus::Pending => write!(f, "pending"),
            ScrapeJobStatus::Processing => write!(f, "processing"),
            ScrapeJobStatus::Completed => write!(f, "completed"),
            ScrapeJobStatus::Failed => write!(f, "failed"),
            ScrapeJobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Scrape Job - a price scraping request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScrapeJob {
    pub id: Uuid,
    pub hotel_group_id: Uuid,
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i32,
    pub adults: i32,
    pub status: ScrapeJobStatus,
    pub force_refresh: bool,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Request to create a new scrape job
#[derive(Debug, Deserialize)]
pub struct CreateScrapeJobRequest {
    pub hotel_group_id: Uuid,
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i32,
    pub adults: i32,
    #[serde(default)]
    pub force_refresh: bool,
}

/// Scrape job with progress information
#[derive(Debug, Clone, Serialize)]
pub struct ScrapeJobWithProgress {
    pub id: Uuid,
    pub hotel_group_id: Uuid,
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i32,
    pub adults: i32,
    pub status: ScrapeJobStatus,
    pub progress: ScrapeProgress,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Progress information
#[derive(Debug, Clone, Serialize)]
pub struct ScrapeProgress {
    pub total: i32,
    pub completed: i32,
    pub failed: i32,
    pub pending: i32,
}

/// Message sent to RabbitMQ for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeJobMessage {
    pub job_id: Uuid,
    pub hotel_group_id: Uuid,
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i32,
    pub adults: i32,
    pub force_refresh: bool,
}
