//! Scheduled Scrape Config Repository (REQ-002 F-003/F-004)

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateScheduledScrapeConfigRequest, ScheduledScrapeConfig, UpdateScheduledScrapeConfigRequest,
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

const COLUMNS: &str = "id, hotel_group_id, name, cron_expression, lookahead_days, los_variants, \
    method, rooms, adults, is_active, last_run_at, next_run_at, created_at, updated_at";

fn from_row(row: &sqlx::postgres::PgRow) -> ScheduledScrapeConfig {
    ScheduledScrapeConfig {
        id: row.get("id"),
        hotel_group_id: row.get("hotel_group_id"),
        name: row.get("name"),
        cron_expression: row.get("cron_expression"),
        lookahead_days: row.get("lookahead_days"),
        los_variants: row.get("los_variants"),
        method: row.get("method"),
        rooms: row.get("rooms"),
        adults: row.get("adults"),
        is_active: row.get("is_active"),
        last_run_at: row.get("last_run_at"),
        next_run_at: row.get("next_run_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub struct ScheduledScrapeConfigRepo;

impl ScheduledScrapeConfigRepo {
    pub async fn create(
        pool: &PgPool,
        req: &CreateScheduledScrapeConfigRequest,
    ) -> AppResult<ScheduledScrapeConfig> {
        let row = sqlx::query(&format!(
            r#"
            INSERT INTO scheduled_scrape_configs
                (hotel_group_id, name, cron_expression, lookahead_days, los_variants, method, rooms, adults, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING {COLUMNS}
            "#
        ))
        .bind(req.hotel_group_id)
        .bind(&req.name)
        .bind(&req.cron_expression)
        .bind(&req.lookahead_days)
        .bind(&req.los_variants)
        .bind(req.method)
        .bind(req.rooms)
        .bind(req.adults)
        .bind(req.is_active)
        .fetch_one(pool)
        .await?;

        Ok(from_row(&row))
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> AppResult<ScheduledScrapeConfig> {
        let row = sqlx::query(&format!("SELECT {COLUMNS} FROM scheduled_scrape_configs WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Scheduled scrape config {} not found", id)))?;

        Ok(from_row(&row))
    }

    pub async fn list_by_group(pool: &PgPool, hotel_group_id: Uuid) -> AppResult<Vec<ScheduledScrapeConfig>> {
        let rows = sqlx::query(&format!(
            "SELECT {COLUMNS} FROM scheduled_scrape_configs WHERE hotel_group_id = $1 ORDER BY created_at DESC"
        ))
        .bind(hotel_group_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.iter().map(from_row).collect())
    }

    /// Every active config — the scheduler tick checks each one's cron
    /// expression against `last_run_at`.
    pub async fn list_active(pool: &PgPool) -> AppResult<Vec<ScheduledScrapeConfig>> {
        let rows = sqlx::query(&format!(
            "SELECT {COLUMNS} FROM scheduled_scrape_configs WHERE is_active = TRUE"
        ))
        .fetch_all(pool)
        .await?;

        Ok(rows.iter().map(from_row).collect())
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateScheduledScrapeConfigRequest,
    ) -> AppResult<ScheduledScrapeConfig> {
        let existing = Self::get_by_id(pool, id).await?;

        let row = sqlx::query(&format!(
            r#"
            UPDATE scheduled_scrape_configs
            SET name = $2, cron_expression = $3, lookahead_days = $4, los_variants = $5,
                method = $6, rooms = $7, adults = $8, is_active = $9
            WHERE id = $1
            RETURNING {COLUMNS}
            "#
        ))
        .bind(id)
        .bind(req.name.clone().or(existing.name))
        .bind(req.cron_expression.clone().unwrap_or(existing.cron_expression))
        .bind(req.lookahead_days.clone().unwrap_or(existing.lookahead_days))
        .bind(req.los_variants.clone().unwrap_or(existing.los_variants))
        .bind(req.method.unwrap_or(existing.method))
        .bind(req.rooms.unwrap_or(existing.rooms))
        .bind(req.adults.unwrap_or(existing.adults))
        .bind(req.is_active.unwrap_or(existing.is_active))
        .fetch_one(pool)
        .await?;

        Ok(from_row(&row))
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM scheduled_scrape_configs WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn mark_run(
        pool: &PgPool,
        id: Uuid,
        last_run_at: DateTime<Utc>,
        next_run_at: Option<DateTime<Utc>>,
    ) -> AppResult<()> {
        sqlx::query("UPDATE scheduled_scrape_configs SET last_run_at = $2, next_run_at = $3 WHERE id = $1")
            .bind(id)
            .bind(last_run_at)
            .bind(next_run_at)
            .execute(pool)
            .await?;
        Ok(())
    }
}
