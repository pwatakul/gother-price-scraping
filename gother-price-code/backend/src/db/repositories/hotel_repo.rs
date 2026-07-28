//! Hotel Repository
//!
//! Database operations for hotels.

use crate::error::{AppError, AppResult};
use crate::models::{CreateHotelRequest, Hotel, HotelWithPrice};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Hotel Repository
pub struct HotelRepo;

impl HotelRepo {
    /// Get hotels by group ID with last price info
    pub async fn get_by_group_id(pool: &PgPool, group_id: Uuid) -> AppResult<Vec<HotelWithPrice>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                h.id,
                h.name,
                h.city,
                h.country,
                (
                    SELECT sr.price_thb 
                    FROM scrape_results sr
                    JOIN scrape_jobs sj ON sr.scrape_job_id = sj.id
                    WHERE sr.hotel_id = h.id AND sj.hotel_group_id = $1
                    ORDER BY sr.scraped_at DESC
                    LIMIT 1
                ) as last_price_thb,
                (
                    SELECT sr.source 
                    FROM scrape_results sr
                    JOIN scrape_jobs sj ON sr.scrape_job_id = sj.id
                    WHERE sr.hotel_id = h.id AND sj.hotel_group_id = $1
                    ORDER BY sr.scraped_at DESC
                    LIMIT 1
                ) as last_price_source,
                (
                    SELECT sr.scraped_at 
                    FROM scrape_results sr
                    JOIN scrape_jobs sj ON sr.scrape_job_id = sj.id
                    WHERE sr.hotel_id = h.id AND sj.hotel_group_id = $1
                    ORDER BY sr.scraped_at DESC
                    LIMIT 1
                ) as last_scraped_at
            FROM hotels h
            JOIN hotel_group_members hgm ON h.id = hgm.hotel_id
            WHERE hgm.hotel_group_id = $1
            ORDER BY h.name
            "#
        )
        .bind(group_id)
        .fetch_all(pool)
        .await?;

        let hotels = rows
            .iter()
            .map(|row| HotelWithPrice {
                id: row.get("id"),
                name: row.get("name"),
                city: row.get("city"),
                country: row.get("country"),
                last_price_thb: row.get("last_price_thb"),
                last_price_source: row.get("last_price_source"),
                last_scraped_at: row.get("last_scraped_at"),
            })
            .collect();

        Ok(hotels)
    }

    /// Get a single hotel by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> AppResult<Hotel> {
        let row = sqlx::query(
            r#"
            SELECT id, name, city, country, normalized_name, created_at, updated_at
            FROM hotels
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Hotel {} not found", id)))?;

        Ok(Hotel {
            id: row.get("id"),
            name: row.get("name"),
            city: row.get("city"),
            country: row.get("country"),
            normalized_name: row.get("normalized_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Create a new hotel
    pub async fn create(pool: &PgPool, req: &CreateHotelRequest) -> AppResult<Hotel> {
        let normalized_name = Hotel::normalize_name(&req.name);

        let row = sqlx::query(
            r#"
            INSERT INTO hotels (name, city, country, normalized_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, city, country, normalized_name, created_at, updated_at
            "#
        )
        .bind(&req.name)
        .bind(&req.city)
        .bind(&req.country)
        .bind(&normalized_name)
        .fetch_one(pool)
        .await?;

        Ok(Hotel {
            id: row.get("id"),
            name: row.get("name"),
            city: row.get("city"),
            country: row.get("country"),
            normalized_name: row.get("normalized_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Find or create a hotel by name and location
    pub async fn find_or_create(
        pool: &PgPool,
        name: &str,
        city: &str,
        country: &str,
    ) -> AppResult<Hotel> {
        let normalized_name = Hotel::normalize_name(name);

        // Try to find existing hotel
        let existing = sqlx::query(
            r#"
            SELECT id, name, city, country, normalized_name, created_at, updated_at
            FROM hotels
            WHERE normalized_name = $1 AND city = $2 AND country = $3
            "#
        )
        .bind(&normalized_name)
        .bind(city)
        .bind(country)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = existing {
            return Ok(Hotel {
                id: row.get("id"),
                name: row.get("name"),
                city: row.get("city"),
                country: row.get("country"),
                normalized_name: row.get("normalized_name"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        // Create new hotel
        let row = sqlx::query(
            r#"
            INSERT INTO hotels (name, city, country, normalized_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, city, country, normalized_name, created_at, updated_at
            "#
        )
        .bind(name)
        .bind(city)
        .bind(country)
        .bind(&normalized_name)
        .fetch_one(pool)
        .await?;

        Ok(Hotel {
            id: row.get("id"),
            name: row.get("name"),
            city: row.get("city"),
            country: row.get("country"),
            normalized_name: row.get("normalized_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Search hotels by name
    pub async fn search(pool: &PgPool, query: &str, limit: i64) -> AppResult<Vec<Hotel>> {
        let search_term = format!("%{}%", query.to_lowercase());

        let rows = sqlx::query(
            r#"
            SELECT id, name, city, country, normalized_name, created_at, updated_at
            FROM hotels
            WHERE LOWER(name) LIKE $1 OR LOWER(city) LIKE $1
            ORDER BY name
            LIMIT $2
            "#
        )
        .bind(&search_term)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let hotels = rows
            .iter()
            .map(|row| Hotel {
                id: row.get("id"),
                name: row.get("name"),
                city: row.get("city"),
                country: row.get("country"),
                normalized_name: row.get("normalized_name"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(hotels)
    }

    /// Get hotels for a scrape job (by group)
    pub async fn get_for_scrape_job(pool: &PgPool, group_id: Uuid) -> AppResult<Vec<Hotel>> {
        let rows = sqlx::query(
            r#"
            SELECT h.id, h.name, h.city, h.country, h.normalized_name, h.created_at, h.updated_at
            FROM hotels h
            JOIN hotel_group_members hgm ON h.id = hgm.hotel_id
            WHERE hgm.hotel_group_id = $1
            ORDER BY h.name
            "#
        )
        .bind(group_id)
        .fetch_all(pool)
        .await?;

        let hotels = rows
            .iter()
            .map(|row| Hotel {
                id: row.get("id"),
                name: row.get("name"),
                city: row.get("city"),
                country: row.get("country"),
                normalized_name: row.get("normalized_name"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(hotels)
    }
}
