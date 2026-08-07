//! Price History Repository (REQ-002 / REQ-005)
//!
//! NOTE: `price_thb`/`original_price` are Postgres NUMERIC columns — sqlx
//! cannot decode NUMERIC directly into f64, so every SELECT here casts
//! explicitly with `::float8` (same fix applied to scrape_result_repo.rs
//! after a real production bug in Part A — do not drop these casts).

use crate::error::AppResult;
use crate::models::{HotelPriceHistory, PriceHistoryQuery, PriceTrendPoint};
use chrono::{Datelike, Months, NaiveDate, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PriceHistoryRepo;

/// Pure (no DB access) so it's unit-testable: the partition table name +
/// [start, end) date range for each of the next `months_ahead` months,
/// starting at the month containing `today`.
pub fn partition_ranges(today: NaiveDate, months_ahead: i32) -> Vec<(String, NaiveDate, NaiveDate)> {
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    (0..months_ahead.max(0) as u32)
        .map(|i| {
            let start = month_start + Months::new(i);
            let end = start + Months::new(1);
            let name = format!("hotel_price_history_{}", start.format("%Y_%m"));
            (name, start, end)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
impl PriceHistoryRepo {
    /// REQ-005 F-002 — idempotently ensure a partition exists for the
    /// current month + each of the next `months_ahead - 1` months.
    /// `CREATE TABLE IF NOT EXISTS` makes repeated calls (e.g. once daily
    /// from worker::partition_manager) safe no-ops once a partition
    /// already exists. Only the generated, controlled table name is
    /// interpolated into SQL — the dates are real bound parameters.
    pub async fn ensure_future_partitions(pool: &PgPool, months_ahead: i32) -> AppResult<()> {
        let today = Utc::now().date_naive();
        for (name, start, end) in partition_ranges(today, months_ahead) {
            let sql = format!(
                "CREATE TABLE IF NOT EXISTS {name} PARTITION OF hotel_price_history FOR VALUES FROM ($1) TO ($2)"
            );
            sqlx::query(&sql).bind(start).bind(end).execute(pool).await?;
        }
        Ok(())
    }

    pub async fn create(
        pool: &PgPool,
        hotel_id: Uuid,
        source: &str,
        room_type: &str,
        price_thb: f64,
        original_price: Option<f64>,
        currency: Option<&str>,
        exchange_rate_id: Uuid,
        meal_plan: Option<&str>,
        cancellation: Option<&str>,
        source_url: Option<&str>,
        checkin_date: NaiveDate,
        checkout_date: NaiveDate,
        rooms: i16,
        adults: i16,
        scrape_job_id: Option<Uuid>,
    ) -> AppResult<HotelPriceHistory> {
        let row = sqlx::query(
            r#"
            INSERT INTO hotel_price_history
                (hotel_id, source, room_type, price_thb, original_price, currency,
                 exchange_rate_id, meal_plan, cancellation, source_url,
                 checkin_date, checkout_date, rooms, adults, scrape_job_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, hotel_id, source, room_type, price_thb::float8 as price_thb,
                      original_price::float8 as original_price, currency, exchange_rate_id,
                      meal_plan, cancellation, source_url, checkin_date, checkout_date,
                      rooms, adults, scrape_job_id, scraped_at
            "#,
        )
        .bind(hotel_id)
        .bind(source)
        .bind(room_type)
        .bind(price_thb)
        .bind(original_price)
        .bind(currency)
        .bind(exchange_rate_id)
        .bind(meal_plan)
        .bind(cancellation)
        .bind(source_url)
        .bind(checkin_date)
        .bind(checkout_date)
        .bind(rooms)
        .bind(adults)
        .bind(scrape_job_id)
        .fetch_one(pool)
        .await?;

        Ok(HotelPriceHistory {
            id: row.get("id"),
            hotel_id: row.get("hotel_id"),
            source: row.get("source"),
            room_type: row.get("room_type"),
            price_thb: row.get("price_thb"),
            original_price: row.get("original_price"),
            currency: row.get("currency"),
            exchange_rate_id: row.get("exchange_rate_id"),
            meal_plan: row.get("meal_plan"),
            cancellation: row.get("cancellation"),
            source_url: row.get("source_url"),
            checkin_date: row.get("checkin_date"),
            checkout_date: row.get("checkout_date"),
            rooms: row.get("rooms"),
            adults: row.get("adults"),
            scrape_job_id: row.get("scrape_job_id"),
            scraped_at: row.get("scraped_at"),
        })
    }

    /// REQ-002 F-007 — filtered query over raw history rows.
    pub async fn query(pool: &PgPool, filters: &PriceHistoryQuery) -> AppResult<Vec<HotelPriceHistory>> {
        let rows = sqlx::query(
            r#"
            SELECT id, hotel_id, source, room_type, price_thb::float8 as price_thb,
                   original_price::float8 as original_price, currency, exchange_rate_id,
                   meal_plan, cancellation, source_url, checkin_date, checkout_date,
                   rooms, adults, scrape_job_id, scraped_at
            FROM hotel_price_history
            WHERE ($1::uuid IS NULL OR hotel_id = $1)
              AND ($2::text IS NULL OR source = $2)
              AND ($3::date IS NULL OR checkin_date >= $3)
              AND ($4::date IS NULL OR checkin_date <= $4)
              AND ($5::timestamptz IS NULL OR scraped_at >= $5)
              AND ($6::timestamptz IS NULL OR scraped_at <= $6)
              AND ($7::uuid IS NULL OR hotel_id IN (
                  SELECT hotel_id FROM hotel_group_members WHERE hotel_group_id = $7
              ))
            ORDER BY scraped_at DESC
            LIMIT $8 OFFSET $9
            "#,
        )
        .bind(filters.hotel_id)
        .bind(&filters.source)
        .bind(filters.checkin_from)
        .bind(filters.checkin_to)
        .bind(filters.scraped_from)
        .bind(filters.scraped_to)
        .bind(filters.hotel_group_id)
        .bind(filters.limit)
        .bind(filters.offset)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| HotelPriceHistory {
                id: row.get("id"),
                hotel_id: row.get("hotel_id"),
                source: row.get("source"),
                room_type: row.get("room_type"),
                price_thb: row.get("price_thb"),
                original_price: row.get("original_price"),
                currency: row.get("currency"),
                exchange_rate_id: row.get("exchange_rate_id"),
                meal_plan: row.get("meal_plan"),
                cancellation: row.get("cancellation"),
                source_url: row.get("source_url"),
                checkin_date: row.get("checkin_date"),
                checkout_date: row.get("checkout_date"),
                rooms: row.get("rooms"),
                adults: row.get("adults"),
                scrape_job_id: row.get("scrape_job_id"),
                scraped_at: row.get("scraped_at"),
            })
            .collect())
    }

    /// Total row count for the same filter set as `query()` (mirrors its
    /// WHERE clause exactly) — used to drive pagination on the hotel-page
    /// full price-history table.
    pub async fn count(pool: &PgPool, filters: &PriceHistoryQuery) -> AppResult<i64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as total
            FROM hotel_price_history
            WHERE ($1::uuid IS NULL OR hotel_id = $1)
              AND ($2::text IS NULL OR source = $2)
              AND ($3::date IS NULL OR checkin_date >= $3)
              AND ($4::date IS NULL OR checkin_date <= $4)
              AND ($5::timestamptz IS NULL OR scraped_at >= $5)
              AND ($6::timestamptz IS NULL OR scraped_at <= $6)
              AND ($7::uuid IS NULL OR hotel_id IN (
                  SELECT hotel_id FROM hotel_group_members WHERE hotel_group_id = $7
              ))
            "#,
        )
        .bind(filters.hotel_id)
        .bind(&filters.source)
        .bind(filters.checkin_from)
        .bind(filters.checkin_to)
        .bind(filters.scraped_from)
        .bind(filters.scraped_to)
        .bind(filters.hotel_group_id)
        .fetch_one(pool)
        .await?;

        Ok(row.get("total"))
    }

    /// REQ-002 F-008 / REQ-003 F-002 — per-hotel trend, backed by
    /// mv_hotel_daily_avg_price (fast, pre-aggregated).
    pub async fn trend_for_hotel(
        pool: &PgPool,
        hotel_id: Uuid,
        source_filter: Option<&str>,
        days: i32,
    ) -> AppResult<Vec<PriceTrendPoint>> {
        let rows = sqlx::query(
            r#"
            SELECT source, day, avg_price_thb::float8 as avg_price_thb,
                   min_price_thb::float8 as min_price_thb, max_price_thb::float8 as max_price_thb,
                   sample_count
            FROM mv_hotel_daily_avg_price
            WHERE hotel_id = $1
              AND day >= NOW() - ($2 || ' days')::interval
              AND ($3::text IS NULL OR source = $3)
            ORDER BY day ASC
            "#,
        )
        .bind(hotel_id)
        .bind(days.to_string())
        .bind(source_filter)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| PriceTrendPoint {
                source: row.get("source"),
                day: row.get("day"),
                avg_price_thb: row.get("avg_price_thb"),
                min_price_thb: row.get("min_price_thb"),
                max_price_thb: row.get("max_price_thb"),
                sample_count: row.get("sample_count"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_expected_names_and_ranges() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
        let ranges = partition_ranges(today, 4);

        assert_eq!(ranges.len(), 4);
        assert_eq!(ranges[0].0, "hotel_price_history_2026_08");
        assert_eq!(ranges[0].1, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        assert_eq!(ranges[0].2, NaiveDate::from_ymd_opt(2026, 9, 1).unwrap());
        assert_eq!(ranges[3].0, "hotel_price_history_2026_11");
    }

    #[test]
    fn handles_year_rollover() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 20).unwrap();
        let ranges = partition_ranges(today, 4);

        assert_eq!(
            ranges.iter().map(|(n, _, _)| n.clone()).collect::<Vec<_>>(),
            vec![
                "hotel_price_history_2026_11",
                "hotel_price_history_2026_12",
                "hotel_price_history_2027_01",
                "hotel_price_history_2027_02",
            ]
        );
        assert_eq!(ranges[2].1, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
    }

    #[test]
    fn zero_months_ahead_is_empty() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
        assert!(partition_ranges(today, 0).is_empty());
    }
}
