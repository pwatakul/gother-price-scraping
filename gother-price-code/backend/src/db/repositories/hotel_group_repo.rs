//! Hotel Group Repository
//!
//! Database operations for hotel groups.

use crate::error::{AppError, AppResult};
use crate::models::{HotelGroup, HotelGroupWithCount, CreateHotelGroupRequest, UpdateGroupSearchConfigRequest, UpdateHotelGroupRequest};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Hotel Group Repository
pub struct HotelGroupRepo;

/// Every column of hotel_groups, including the saved search config
/// (ADR-012). Kept in one place so the three read paths cannot drift.
const COLUMNS: &str = "id, name, description, search_method, search_days_ahead, \
    search_rooms, search_adults, created_at, updated_at";

fn from_row(row: &sqlx::postgres::PgRow) -> HotelGroup {
    HotelGroup {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        search_method: row.get("search_method"),
        search_days_ahead: row.get("search_days_ahead"),
        search_rooms: row.get("search_rooms"),
        search_adults: row.get("search_adults"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

impl HotelGroupRepo {
    /// List all hotel groups with hotel counts
    pub async fn list_all(pool: &PgPool) -> AppResult<Vec<HotelGroupWithCount>> {
        // Correlated subqueries rather than two LEFT JOINs + GROUP BY.
        // Joining members *and* jobs produced a cartesian product, so
        // COUNT(hgm.id) returned members × jobs — a 20-hotel group showed
        // 300 once it had 15 scrape jobs. Counting per-table keeps each
        // aggregate independent, and avoids the row fan-out entirely as
        // job history grows (REQ-005 targets 2200 hotels).
        let rows = sqlx::query(
            r#"
            SELECT
                hg.id,
                hg.name,
                hg.description,
                (SELECT COUNT(*) FROM hotel_group_members m
                  WHERE m.hotel_group_id = hg.id)::bigint as hotel_count,
                (SELECT MAX(j.created_at) FROM scrape_jobs j
                  WHERE j.hotel_group_id = hg.id) as last_scraped_at,
                hg.created_at
            FROM hotel_groups hg
            ORDER BY hg.created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        let groups = rows
            .iter()
            .map(|row| HotelGroupWithCount {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                hotel_count: row.get::<i64, _>("hotel_count"),
                last_scraped_at: row.get("last_scraped_at"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(groups)
    }

    /// Get a single hotel group by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> AppResult<HotelGroup> {
        let row = sqlx::query(&format!(
            r#"
            SELECT {COLUMNS}
            FROM hotel_groups
            WHERE id = $1
            "#
        ))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Hotel group {} not found", id)))?;

        Ok(from_row(&row))
    }

    /// Create a new hotel group
    pub async fn create(pool: &PgPool, req: &CreateHotelGroupRequest) -> AppResult<HotelGroup> {
        let row = sqlx::query(&format!(
            r#"
            INSERT INTO hotel_groups (name, description)
            VALUES ($1, $2)
            RETURNING {COLUMNS}
            "#
        ))
        .bind(&req.name)
        .bind(&req.description)
        .fetch_one(pool)
        .await?;

        Ok(from_row(&row))
    }

    /// Update a hotel group
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateHotelGroupRequest,
    ) -> AppResult<HotelGroup> {
        let row = sqlx::query(&format!(
            r#"
            UPDATE hotel_groups
            SET 
                name = COALESCE($2, name),
                description = COALESCE($3, description)
            WHERE id = $1
            RETURNING {COLUMNS}
            "#
        ))
        .bind(id)
        .bind(&req.name)
        .bind(&req.description)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Hotel group {} not found", id)))?;

        Ok(from_row(&row))
    }

    /// Update only the saved price-search config (ADR-012). Separate from
    /// `update` so renaming a group cannot clobber its search settings.
    pub async fn update_search_config(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateGroupSearchConfigRequest,
    ) -> AppResult<HotelGroup> {
        let row = sqlx::query(&format!(
            r#"
            UPDATE hotel_groups
            SET search_method     = COALESCE($2, search_method),
                search_days_ahead = COALESCE($3, search_days_ahead),
                search_rooms      = COALESCE($4, search_rooms),
                search_adults     = COALESCE($5, search_adults)
            WHERE id = $1
            RETURNING {COLUMNS}
            "#
        ))
        .bind(id)
        .bind(req.search_method)
        .bind(req.search_days_ahead.as_deref())
        .bind(req.search_rooms)
        .bind(req.search_adults)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Hotel group {} not found", id)))?;

        Ok(from_row(&row))
    }

    /// Delete a hotel group
    pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM hotel_groups WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Hotel group {} not found", id)));
        }

        Ok(())
    }

    /// Add a hotel to a group
    pub async fn add_hotel(pool: &PgPool, group_id: Uuid, hotel_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO hotel_group_members (hotel_group_id, hotel_id)
            VALUES ($1, $2)
            ON CONFLICT (hotel_group_id, hotel_id) DO NOTHING
            "#
        )
        .bind(group_id)
        .bind(hotel_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Remove a hotel from a group
    pub async fn remove_hotel(pool: &PgPool, group_id: Uuid, hotel_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM hotel_group_members
            WHERE hotel_group_id = $1 AND hotel_id = $2
            "#
        )
        .bind(group_id)
        .bind(hotel_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Hotel not found in this group".to_string(),
            ));
        }

        Ok(())
    }
}
