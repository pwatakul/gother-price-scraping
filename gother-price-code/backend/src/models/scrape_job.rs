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

/// Which scraper(s) produce results for a job (REQ-001 F-020/F-022).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "scrape_method", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ScrapeMethod {
    Serpapi,
    Gemini,
    Both,
}

impl Default for ScrapeMethod {
    fn default() -> Self {
        ScrapeMethod::Serpapi
    }
}

/// Device dimension (REQ-001 F-023).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "device_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Device {
    Desktop,
    MobileWeb,
}

impl Default for Device {
    fn default() -> Self {
        Device::Desktop
    }
}

impl Device {
    pub fn as_str(&self) -> &'static str {
        match self {
            Device::Desktop => "desktop",
            Device::MobileWeb => "mobile_web",
        }
    }
}

/// Login-state dimension (REQ-001 F-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "login_state_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LoginState {
    Public,
    Member,
}

impl Default for LoginState {
    fn default() -> Self {
        LoginState::Public
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
    pub method: ScrapeMethod,
    pub los_variants: Vec<i32>,
    pub device: Device,
    pub login_state: LoginState,
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
    #[serde(default)]
    pub method: ScrapeMethod,
    /// Length-of-stay variants in nights; defaults to a single 1-night pass.
    #[serde(default = "default_los_variants")]
    pub los_variants: Vec<i32>,
    #[serde(default)]
    pub device: Device,
    #[serde(default)]
    pub login_state: LoginState,
}

fn default_los_variants() -> Vec<i32> {
    vec![1]
}

/// Job-level defaults used to fill in any field a per-hotel override
/// (`scrape_job_hotel_params`) leaves blank.
#[derive(Debug, Clone, Copy)]
pub struct JobDefaults {
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i32,
    pub adults: i32,
}

impl From<&CreateScrapeJobRequest> for JobDefaults {
    fn from(req: &CreateScrapeJobRequest) -> Self {
        JobDefaults {
            checkin_date: req.checkin_date,
            checkout_date: req.checkout_date,
            rooms: req.rooms,
            adults: req.adults,
        }
    }
}

impl From<&ScrapeJob> for JobDefaults {
    fn from(job: &ScrapeJob) -> Self {
        JobDefaults {
            checkin_date: job.checkin_date,
            checkout_date: job.checkout_date,
            rooms: job.rooms,
            adults: job.adults,
        }
    }
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
    pub method: ScrapeMethod,
    pub device: Device,
    pub login_state: LoginState,
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
    pub method: ScrapeMethod,
    pub los_variants: Vec<i32>,
    pub device: Device,
    pub login_state: LoginState,
}
