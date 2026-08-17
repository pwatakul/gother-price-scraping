-- Migration: Make booking window an explicit comparison dimension
-- Description: ADR-013 — price depends on how far ahead you book, so any
-- comparison that ignores booking window is not apples-to-apples.
--
-- mv_hotel_daily_avg_price grouped by (hotel_id, source, day) only, so a
-- +30-day quote and a +35-day quote for the same hotel collapsed into one
-- point. Worse, providers rarely cover the same windows: on real data
-- Klook had only +30 and Agoda only +35, so the trend chart plotted one
-- against the other as if they were the same product.
--
-- mv_hotel_price_by_stay is new: the latest price per
-- (hotel, source, check-in date). That triple is the unit a fair
-- comparison needs — same hotel, same stay, different provider.
--
-- Both views keep a unique index covering every grouping column, which
-- REFRESH MATERIALIZED VIEW CONCURRENTLY requires (see migration 016).

DROP MATERIALIZED VIEW mv_hotel_daily_avg_price;

CREATE MATERIALIZED VIEW mv_hotel_daily_avg_price AS
SELECT
    hotel_id,
    source,
    DATE_TRUNC('day', scraped_at) AS day,
    (checkin_date - DATE(scraped_at)) AS days_in_advance,
    AVG(price_thb) AS avg_price_thb,
    MIN(price_thb) AS min_price_thb,
    MAX(price_thb) AS max_price_thb,
    COUNT(*) AS sample_count
FROM hotel_price_history
GROUP BY hotel_id, source, DATE_TRUNC('day', scraped_at),
         (checkin_date - DATE(scraped_at));

CREATE UNIQUE INDEX idx_mv_daily_avg_price_hotel_source_day
    ON mv_hotel_daily_avg_price (hotel_id, source, day, days_in_advance);

-- Latest quote per hotel + provider + stay date. Unlike
-- mv_hotel_market_position (latest per hotel+source, whatever date that
-- happened to be), this keeps the check-in date so providers are only
-- ever compared on the same stay.
CREATE MATERIALIZED VIEW mv_hotel_price_by_stay AS
SELECT DISTINCT ON (hotel_id, source, checkin_date)
    hotel_id,
    source,
    checkin_date,
    (checkin_date - DATE(scraped_at)) AS days_in_advance,
    price_thb,
    room_type,
    scraped_at
FROM hotel_price_history
ORDER BY hotel_id, source, checkin_date, scraped_at DESC;

CREATE UNIQUE INDEX idx_mv_price_by_stay_hotel_source_checkin
    ON mv_hotel_price_by_stay (hotel_id, source, checkin_date);

CREATE INDEX idx_mv_price_by_stay_window
    ON mv_hotel_price_by_stay (hotel_id, days_in_advance);
