//! Hotel Directory Repository (REQ-007)
//!
//! Global, cross-group hotel listing — separate from `HotelRepo` (which is
//! scoped to hotel-group operations) to avoid bloating that file. All
//! price columns are cast `::float8` (same NUMERIC-decode rule as
//! elsewhere in this codebase).

use crate::error::AppResult;
use crate::models::{HotelListQuery, HotelWithGroupsAndPrice};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct HotelDirectoryRepo;

impl HotelDirectoryRepo {
    pub async fn list(pool: &PgPool, filters: &HotelListQuery) -> AppResult<(Vec<HotelWithGroupsAndPrice>, i64)> {
        let search = filters.q.as_ref().map(|q| format!("%{}%", q.to_lowercase()));

        let rows = sqlx::query(
            r#"
            SELECT
                h.id, h.name, h.city, h.country, h.hid, h.slug, h.supplier_type,
                COALESCE(
                    ARRAY_AGG(DISTINCT hg.name) FILTER (WHERE hg.name IS NOT NULL),
                    ARRAY[]::text[]
                ) as group_names,
                (
                    SELECT sr.price_thb::float8 FROM scrape_results sr
                    WHERE sr.hotel_id = h.id ORDER BY sr.scraped_at DESC LIMIT 1
                ) as last_price_thb,
                (
                    SELECT sr.source FROM scrape_results sr
                    WHERE sr.hotel_id = h.id ORDER BY sr.scraped_at DESC LIMIT 1
                ) as last_price_source,
                (
                    SELECT sr.scraped_at FROM scrape_results sr
                    WHERE sr.hotel_id = h.id ORDER BY sr.scraped_at DESC LIMIT 1
                ) as last_scraped_at
            FROM hotels h
            LEFT JOIN hotel_group_members hgm ON hgm.hotel_id = h.id
            LEFT JOIN hotel_groups hg ON hg.id = hgm.hotel_group_id
            WHERE ($1::text IS NULL OR LOWER(h.country) = LOWER($1))
              AND ($2::text IS NULL OR LOWER(h.city) = LOWER($2))
              AND ($3::text IS NULL OR LOWER(h.name) LIKE $3)
            GROUP BY h.id
            ORDER BY h.name
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(&filters.country)
        .bind(&filters.city)
        .bind(&search)
        .bind(filters.limit)
        .bind(filters.offset)
        .fetch_all(pool)
        .await?;

        let total_row = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT h.id) as total
            FROM hotels h
            WHERE ($1::text IS NULL OR LOWER(h.country) = LOWER($1))
              AND ($2::text IS NULL OR LOWER(h.city) = LOWER($2))
              AND ($3::text IS NULL OR LOWER(h.name) LIKE $3)
            "#,
        )
        .bind(&filters.country)
        .bind(&filters.city)
        .bind(&search)
        .fetch_one(pool)
        .await?;

        let hotels = rows
            .iter()
            .map(|row| HotelWithGroupsAndPrice {
                id: row.get("id"),
                name: row.get("name"),
                city: row.get("city"),
                country: row.get("country"),
                hid: row.get("hid"),
                slug: row.get("slug"),
                supplier_type: row.get("supplier_type"),
                group_names: row.get("group_names"),
                last_price_thb: row.get("last_price_thb"),
                last_price_source: row.get("last_price_source"),
                last_scraped_at: row.get("last_scraped_at"),
            })
            .collect();

        Ok((hotels, total_row.get("total")))
    }

    pub async fn distinct_countries(pool: &PgPool) -> AppResult<Vec<String>> {
        let rows = sqlx::query("SELECT DISTINCT country FROM hotels WHERE country != '' ORDER BY country")
            .fetch_all(pool)
            .await?;
        Ok(rows.iter().map(|r| r.get("country")).collect())
    }

    pub async fn distinct_cities(pool: &PgPool, country: Option<&str>) -> AppResult<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT city FROM hotels
            WHERE city != '' AND ($1::text IS NULL OR LOWER(country) = LOWER($1))
            ORDER BY city
            "#,
        )
        .bind(country)
        .fetch_all(pool)
        .await?;
        Ok(rows.iter().map(|r| r.get("city")).collect())
    }

    pub async fn group_names_for_hotel(pool: &PgPool, hotel_id: Uuid) -> AppResult<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT hg.name FROM hotel_group_members hgm
            JOIN hotel_groups hg ON hg.id = hgm.hotel_group_id
            WHERE hgm.hotel_id = $1
            "#,
        )
        .bind(hotel_id)
        .fetch_all(pool)
        .await?;
        Ok(rows.iter().map(|r| r.get("name")).collect())
    }
}
