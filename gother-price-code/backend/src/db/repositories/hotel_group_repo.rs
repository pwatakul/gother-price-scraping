//! Hotel Group Repository
//!
//! Database operations for hotel groups.

use crate::error::{AppError, AppResult};
use crate::models::{HotelGroup, HotelGroupWithCount, CreateHotelGroupRequest, UpdateHotelGroupRequest};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Hotel Group Repository
pub struct HotelGroupRepo;

impl HotelGroupRepo {
    /// List all hotel groups with hotel counts
    pub async fn list_all(pool: &PgPool) -> AppResult<Vec<HotelGroupWithCount>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                hg.id,
                hg.name,
                hg.description,
                COUNT(hgm.id)::bigint as hotel_count,
                MAX(sj.created_at) as last_scraped_at,
                hg.created_at
            FROM hotel_groups hg
            LEFT JOIN hotel_group_members hgm ON hg.id = hgm.hotel_group_id
            LEFT JOIN scrape_jobs sj ON hg.id = sj.hotel_group_id
            GROUP BY hg.id
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
        let row = sqlx::query(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM hotel_groups
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Hotel group {} not found", id)))?;

        Ok(HotelGroup {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Create a new hotel group
    pub async fn create(pool: &PgPool, req: &CreateHotelGroupRequest) -> AppResult<HotelGroup> {
        let row = sqlx::query(
            r#"
            INSERT INTO hotel_groups (name, description)
            VALUES ($1, $2)
            RETURNING id, name, description, created_at, updated_at
            "#
        )
        .bind(&req.name)
        .bind(&req.description)
        .fetch_one(pool)
        .await?;

        Ok(HotelGroup {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Update a hotel group
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateHotelGroupRequest,
    ) -> AppResult<HotelGroup> {
        let row = sqlx::query(
            r#"
            UPDATE hotel_groups
            SET 
                name = COALESCE($2, name),
                description = COALESCE($3, description)
            WHERE id = $1
            RETURNING id, name, description, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.description)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Hotel group {} not found", id)))?;

        Ok(HotelGroup {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
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
