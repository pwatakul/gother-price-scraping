//! Materialized View Repository (REQ-003 / REQ-005)
//!
//! Reads from the 5 views created in migration 016, plus `refresh_all`
//! which is called after every scrape job completes (see
//! worker/jobs/scrape_job.rs). All price columns are cast `::float8`
//! (views are built on NUMERIC columns — same sqlx decode rule as
//! elsewhere in this codebase).

use crate::error::AppResult;
use crate::models::{
    BookingWindowRow, HeatmapCell, MarketOverview, MarketPositionEntry, MarketPositionRow,
    ParityViolationRow, WinRateRow,
};
use crate::scraper::providers::GOTHER;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

pub struct MaterializedViewRepo;

impl MaterializedViewRepo {
    /// Refresh every analytics view. Uses CONCURRENTLY (non-blocking reads
    /// during refresh) — each view has a unique index specifically to
    /// support this. Runs sequentially; failures are propagated (the
    /// caller logs and continues — one bad refresh shouldn't crash the
    /// worker).
    pub async fn refresh_all(pool: &PgPool) -> AppResult<()> {
        for view in [
            "mv_hotel_market_position",
            "mv_hotel_daily_avg_price",
            "mv_hotel_win_rate",
            "mv_hotel_booking_window",
            "mv_hotel_parity_violations",
        ] {
            sqlx::query(&format!("REFRESH MATERIALIZED VIEW CONCURRENTLY {view}"))
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Raw market-position rows, optionally scoped to one hotel group.
    pub async fn market_position(
        pool: &PgPool,
        hotel_group_id: Option<Uuid>,
    ) -> AppResult<Vec<MarketPositionRow>> {
        let rows = sqlx::query(
            r#"
            SELECT mp.hotel_id, mp.source, mp.room_type, mp.price_thb::float8 as price_thb,
                   mp.checkin_date, mp.scraped_at
            FROM mv_hotel_market_position mp
            WHERE $1::uuid IS NULL OR mp.hotel_id IN (
                SELECT hotel_id FROM hotel_group_members WHERE hotel_group_id = $1
            )
            "#,
        )
        .bind(hotel_group_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| MarketPositionRow {
                hotel_id: r.get("hotel_id"),
                source: r.get("source"),
                room_type: r.get("room_type"),
                price_thb: r.get("price_thb"),
                checkin_date: r.get("checkin_date"),
                scraped_at: r.get("scraped_at"),
            })
            .collect())
    }

    /// REQ-003 F-001 — market overview KPIs, derived from market_position.
    pub async fn overview(pool: &PgPool, hotel_group_id: Option<Uuid>) -> AppResult<MarketOverview> {
        let entries = Self::position_table(pool, hotel_group_id).await?;
        let total_hotels = entries.len() as i64;
        let with_both: Vec<&MarketPositionEntry> =
            entries.iter().filter(|e| e.gother_price.is_some() && e.best_price.is_some()).collect();

        let gother_cheapest_pct = if !with_both.is_empty() {
            100.0 * with_both.iter().filter(|e| e.is_winning).count() as f64 / with_both.len() as f64
        } else {
            0.0
        };
        let avg_gap_thb = if !with_both.is_empty() {
            with_both.iter().filter_map(|e| e.gap_thb).sum::<f64>() / with_both.len() as f64
        } else {
            0.0
        };

        Ok(MarketOverview { total_hotels, gother_cheapest_pct, avg_gap_thb })
    }

    /// REQ-003 F-003 — one row per hotel: Gother price, best OTA price, gap.
    pub async fn position_table(
        pool: &PgPool,
        hotel_group_id: Option<Uuid>,
    ) -> AppResult<Vec<MarketPositionEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT mp.hotel_id, h.name as hotel_name, mp.source, mp.price_thb::float8 as price_thb
            FROM mv_hotel_market_position mp
            JOIN hotels h ON h.id = mp.hotel_id
            WHERE $1::uuid IS NULL OR mp.hotel_id IN (
                SELECT hotel_id FROM hotel_group_members WHERE hotel_group_id = $1
            )
            "#,
        )
        .bind(hotel_group_id)
        .fetch_all(pool)
        .await?;

        let mut by_hotel: HashMap<Uuid, (String, Vec<(String, f64)>)> = HashMap::new();
        for row in &rows {
            let hotel_id: Uuid = row.get("hotel_id");
            let hotel_name: String = row.get("hotel_name");
            let source: String = row.get("source");
            let price: f64 = row.get("price_thb");
            by_hotel.entry(hotel_id).or_insert_with(|| (hotel_name, Vec::new())).1.push((source, price));
        }

        let mut entries: Vec<MarketPositionEntry> = by_hotel
            .into_iter()
            .map(|(hotel_id, (hotel_name, prices))| {
                let gother_price = prices.iter().find(|(s, _)| s == GOTHER).map(|(_, p)| *p);
                let best_ota = prices
                    .iter()
                    .filter(|(s, _)| s != GOTHER)
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                let (best_price, best_source) = match (best_ota, gother_price) {
                    (Some((_, p)), Some(gp)) if gp <= *p => (Some(gp), Some(GOTHER.to_string())),
                    (Some((s, p)), _) => (Some(*p), Some(s.clone())),
                    (None, Some(gp)) => (Some(gp), Some(GOTHER.to_string())),
                    (None, None) => (None, None),
                };

                let gap_thb = match (gother_price, best_price) {
                    (Some(gp), Some(bp)) => Some(gp - bp),
                    _ => None,
                };
                let gap_pct = match (gap_thb, best_price) {
                    (Some(gap), Some(bp)) if bp != 0.0 => Some(gap / bp * 100.0),
                    _ => None,
                };
                let is_winning = best_source.as_deref() == Some(GOTHER);

                MarketPositionEntry {
                    hotel_id,
                    hotel_name,
                    gother_price,
                    best_price,
                    best_source,
                    gap_thb,
                    gap_pct,
                    is_winning,
                }
            })
            .collect();

        entries.sort_by(|a, b| {
            b.gap_pct.unwrap_or(f64::MIN).partial_cmp(&a.gap_pct.unwrap_or(f64::MIN)).unwrap()
        });

        Ok(entries)
    }

    /// REQ-003 F-004 — hotel x provider grid. Built directly from
    /// market_position (no separate mv_hotel_competitor_summary — see
    /// plan notes on why that view was skipped).
    pub async fn heatmap(pool: &PgPool, hotel_group_id: Option<Uuid>) -> AppResult<Vec<HeatmapCell>> {
        let rows = sqlx::query(
            r#"
            SELECT mp.hotel_id, h.name as hotel_name, mp.source, mp.price_thb::float8 as price_thb
            FROM mv_hotel_market_position mp
            JOIN hotels h ON h.id = mp.hotel_id
            WHERE $1::uuid IS NULL OR mp.hotel_id IN (
                SELECT hotel_id FROM hotel_group_members WHERE hotel_group_id = $1
            )
            "#,
        )
        .bind(hotel_group_id)
        .fetch_all(pool)
        .await?;

        let mut gother_by_hotel: HashMap<Uuid, f64> = HashMap::new();
        let mut all: Vec<(Uuid, String, String, f64)> = Vec::new();
        for row in &rows {
            let hotel_id: Uuid = row.get("hotel_id");
            let hotel_name: String = row.get("hotel_name");
            let source: String = row.get("source");
            let price: f64 = row.get("price_thb");
            if source == GOTHER {
                gother_by_hotel.insert(hotel_id, price);
            }
            all.push((hotel_id, hotel_name, source, price));
        }

        Ok(all
            .into_iter()
            .map(|(hotel_id, hotel_name, source, price)| {
                let gap_pct = gother_by_hotel
                    .get(&hotel_id)
                    .filter(|_| source != GOTHER)
                    .map(|gp| (gp - price) / price * 100.0);
                HeatmapCell { hotel_id, hotel_name, source, price_thb: Some(price), gap_pct }
            })
            .collect())
    }

    /// REQ-003 F-005 — win rate per hotel.
    pub async fn win_rate(pool: &PgPool) -> AppResult<Vec<WinRateRow>> {
        let rows = sqlx::query(
            "SELECT hotel_id, days_won, days_total, win_rate_pct::float8 as win_rate_pct FROM mv_hotel_win_rate",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| WinRateRow {
                hotel_id: r.get("hotel_id"),
                days_won: r.get("days_won"),
                days_total: r.get("days_total"),
                win_rate_pct: r.get("win_rate_pct"),
            })
            .collect())
    }

    /// REQ-003 F-014 — price by days-in-advance for one hotel.
    pub async fn booking_window(pool: &PgPool, hotel_id: Uuid) -> AppResult<Vec<BookingWindowRow>> {
        let rows = sqlx::query(
            r#"
            SELECT source, days_in_advance, avg_price_thb::float8 as avg_price_thb,
                   min_price_thb::float8 as min_price_thb, sample_count
            FROM mv_hotel_booking_window
            WHERE hotel_id = $1
            ORDER BY days_in_advance DESC
            "#,
        )
        .bind(hotel_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| BookingWindowRow {
                source: r.get("source"),
                days_in_advance: r.get("days_in_advance"),
                avg_price_thb: r.get("avg_price_thb"),
                min_price_thb: r.get("min_price_thb"),
                sample_count: r.get("sample_count"),
            })
            .collect())
    }

    /// REQ-003 F-013 — hotels where Gother exceeds the best OTA by more
    /// than `threshold` percent (default 5.0).
    pub async fn parity_violations(pool: &PgPool, threshold: f64) -> AppResult<Vec<ParityViolationRow>> {
        let rows = sqlx::query(
            r#"
            SELECT pv.hotel_id, h.name as hotel_name, pv.gother_price::float8 as gother_price,
                   pv.best_ota_price::float8 as best_ota_price, pv.gap_pct::float8 as gap_pct
            FROM mv_hotel_parity_violations pv
            JOIN hotels h ON h.id = pv.hotel_id
            WHERE pv.gap_pct > $1
            ORDER BY pv.gap_pct DESC
            "#,
        )
        .bind(threshold)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| ParityViolationRow {
                hotel_id: r.get("hotel_id"),
                hotel_name: r.get("hotel_name"),
                gother_price: r.get("gother_price"),
                best_ota_price: r.get("best_ota_price"),
                gap_pct: r.get("gap_pct"),
            })
            .collect())
    }
}
