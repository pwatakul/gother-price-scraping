//! Scrape Job Repository
//!
//! Database operations for scrape jobs.

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateScrapeJobRequest, HotelScrapeStatus, ScrapeJob, ScrapeJobStatus,
    ScrapeJobWithProgress, ScrapeProgress,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

const JOB_COLUMNS: &str = "id, hotel_group_id, checkin_date, checkout_date, rooms, adults, \
    status, force_refresh, method, los_variants, device, login_state, created_at, completed_at";

fn job_from_row(row: &sqlx::postgres::PgRow) -> ScrapeJob {
    ScrapeJob {
        id: row.get("id"),
        hotel_group_id: row.get("hotel_group_id"),
        checkin_date: row.get("checkin_date"),
        checkout_date: row.get("checkout_date"),
        rooms: row.get("rooms"),
        adults: row.get("adults"),
        status: row.get("status"),
        force_refresh: row.get("force_refresh"),
        method: row.get("method"),
        los_variants: row.get("los_variants"),
        device: row.get("device"),
        login_state: row.get("login_state"),
        created_at: row.get("created_at"),
        completed_at: row.get("completed_at"),
    }
}

/// Scrape Job Repository
pub struct ScrapeJobRepo;

impl ScrapeJobRepo {
    /// Create a new scrape job
    pub async fn create(pool: &PgPool, req: &CreateScrapeJobRequest) -> AppResult<ScrapeJob> {
        let row = sqlx::query(&format!(
            r#"
            INSERT INTO scrape_jobs (hotel_group_id, checkin_date, checkout_date, rooms, adults,
                                     force_refresh, method, los_variants, device, login_state)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING {JOB_COLUMNS}
            "#
        ))
        .bind(req.hotel_group_id)
        .bind(req.checkin_date)
        .bind(req.checkout_date)
        .bind(req.rooms)
        .bind(req.adults)
        .bind(req.force_refresh)
        .bind(req.method)
        .bind(&req.los_variants)
        .bind(req.device)
        .bind(req.login_state)
        .fetch_one(pool)
        .await?;

        Ok(job_from_row(&row))
    }

    /// Get a scrape job by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> AppResult<ScrapeJob> {
        let row = sqlx::query(&format!("SELECT {JOB_COLUMNS} FROM scrape_jobs WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Scrape job {} not found", id)))?;

        Ok(job_from_row(&row))
    }

    /// Get a scrape job with progress
    pub async fn get_with_progress(pool: &PgPool, id: Uuid) -> AppResult<ScrapeJobWithProgress> {
        let job = Self::get_by_id(pool, id).await?;

        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'success') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                COUNT(*) FILTER (WHERE status IN ('pending', 'processing')) as pending
            FROM scrape_hotel_status
            WHERE scrape_job_id = $1
            "#
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(ScrapeJobWithProgress {
            id: job.id,
            hotel_group_id: job.hotel_group_id,
            checkin_date: job.checkin_date,
            checkout_date: job.checkout_date,
            rooms: job.rooms,
            adults: job.adults,
            status: job.status,
            method: job.method,
            device: job.device,
            login_state: job.login_state,
            progress: ScrapeProgress {
                total: row.get::<i64, _>("total") as i32,
                completed: row.get::<i64, _>("completed") as i32,
                failed: row.get::<i64, _>("failed") as i32,
                pending: row.get::<i64, _>("pending") as i32,
            },
            created_at: job.created_at,
            completed_at: job.completed_at,
        })
    }

    /// List scrape jobs by group ID
    pub async fn list_by_group(
        pool: &PgPool,
        group_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<ScrapeJob>> {
        let rows = sqlx::query(&format!(
            r#"
            SELECT {JOB_COLUMNS}
            FROM scrape_jobs
            WHERE hotel_group_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(group_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows.iter().map(job_from_row).collect())
    }

    /// Total jobs for a group, so the UI can render page numbers rather
    /// than a bare "next" guess. Job history grows steadily — a scheduled
    /// grid adds 5 rows per fire — so this list is genuinely paginated
    /// server-side rather than fetched whole.
    pub async fn count_by_group(pool: &PgPool, group_id: Uuid) -> AppResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as total FROM scrape_jobs WHERE hotel_group_id = $1")
            .bind(group_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("total"))
    }

    /// Update job status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: ScrapeJobStatus,
    ) -> AppResult<ScrapeJob> {
        let completed_at = if status == ScrapeJobStatus::Completed
            || status == ScrapeJobStatus::Failed
            || status == ScrapeJobStatus::Cancelled
        {
            Some(chrono::Utc::now())
        } else {
            None
        };

        let status_str = match status {
            ScrapeJobStatus::Pending => "pending",
            ScrapeJobStatus::Processing => "processing",
            ScrapeJobStatus::Completed => "completed",
            ScrapeJobStatus::Failed => "failed",
            ScrapeJobStatus::Cancelled => "cancelled",
        };

        let row = sqlx::query(&format!(
            r#"
            UPDATE scrape_jobs
            SET status = $2::scrape_job_status, completed_at = COALESCE($3, completed_at)
            WHERE id = $1
            RETURNING {JOB_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(status_str)
        .bind(completed_at)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scrape job {} not found", id)))?;

        Ok(job_from_row(&row))
    }

    /// Cancel a running job
    pub async fn cancel(pool: &PgPool, id: Uuid) -> AppResult<ScrapeJob> {
        Self::update_status(pool, id, ScrapeJobStatus::Cancelled).await
    }

    /// Initialize hotel status records for a job
    pub async fn init_hotel_statuses(pool: &PgPool, job_id: Uuid, group_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO scrape_hotel_status (scrape_job_id, hotel_id, status)
            SELECT $1, hotel_id, 'pending'
            FROM hotel_group_members
            WHERE hotel_group_id = $2
            "#
        )
        .bind(job_id)
        .bind(group_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Insert per-hotel search-parameter overrides for a job (REQ-001 F-002
    /// JobDefaults fallback model). Rows with all-`None` fields are skipped.
    pub async fn insert_hotel_param_overrides(
        pool: &PgPool,
        job_id: Uuid,
        overrides: &[(Uuid, crate::models::JobHotelParamOverride)],
    ) -> AppResult<()> {
        for (hotel_id, o) in overrides {
            if o.checkin_date.is_none()
                && o.checkout_date.is_none()
                && o.rooms.is_none()
                && o.adults.is_none()
                && o.currency.is_none()
            {
                continue;
            }

            sqlx::query(
                r#"
                INSERT INTO scrape_job_hotel_params
                    (scrape_job_id, hotel_id, checkin_date, checkout_date, rooms, adults, currency)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (scrape_job_id, hotel_id) DO UPDATE SET
                    checkin_date = EXCLUDED.checkin_date,
                    checkout_date = EXCLUDED.checkout_date,
                    rooms = EXCLUDED.rooms,
                    adults = EXCLUDED.adults,
                    currency = EXCLUDED.currency
                "#
            )
            .bind(job_id)
            .bind(hotel_id)
            .bind(o.checkin_date)
            .bind(o.checkout_date)
            .bind(o.rooms)
            .bind(o.adults)
            .bind(&o.currency)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Fetch the per-hotel override row for a job/hotel pair, if any.
    pub async fn get_hotel_param_override(
        pool: &PgPool,
        job_id: Uuid,
        hotel_id: Uuid,
    ) -> AppResult<Option<crate::models::JobHotelParamOverride>> {
        let row = sqlx::query(
            r#"
            SELECT checkin_date, checkout_date, rooms, adults, currency
            FROM scrape_job_hotel_params
            WHERE scrape_job_id = $1 AND hotel_id = $2
            "#
        )
        .bind(job_id)
        .bind(hotel_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| crate::models::JobHotelParamOverride {
            hid: None,
            hotel_name: None,
            checkin_date: r.get("checkin_date"),
            checkout_date: r.get("checkout_date"),
            rooms: r.get("rooms"),
            adults: r.get("adults"),
            currency: r.get("currency"),
        }))
    }

    /// Update hotel scrape status
    pub async fn update_hotel_status(
        pool: &PgPool,
        job_id: Uuid,
        hotel_id: Uuid,
        status: HotelScrapeStatus,
        error_message: Option<&str>,
    ) -> AppResult<()> {
        let status_str = match status {
            HotelScrapeStatus::Pending => "pending",
            HotelScrapeStatus::Processing => "processing",
            HotelScrapeStatus::Success => "success",
            HotelScrapeStatus::Failed => "failed",
        };

        sqlx::query(
            r#"
            UPDATE scrape_hotel_status
            SET status = $3::hotel_scrape_status, error_message = $4, retry_count = retry_count + 1
            WHERE scrape_job_id = $1 AND hotel_id = $2
            "#
        )
        .bind(job_id)
        .bind(hotel_id)
        .bind(status_str)
        .bind(error_message)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if all hotels are processed
    pub async fn is_job_complete(pool: &PgPool, job_id: Uuid) -> AppResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM scrape_hotel_status
            WHERE scrape_job_id = $1 AND status IN ('pending', 'processing')
            "#
        )
        .bind(job_id)
        .fetch_one(pool)
        .await?;

        let pending: i64 = row.get("count");
        Ok(pending == 0)
    }
}
