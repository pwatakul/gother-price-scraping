-- Migration: Create analytics materialized views
-- Description: REQ-003/REQ-005 — pre-aggregated views backing the
-- analytics dashboard, refreshed via materialized_view_repo::refresh_all
-- after every scrape job completes. Each view has a unique index so
-- REFRESH MATERIALIZED VIEW CONCURRENTLY (non-blocking) works.
-- mv_hotel_competitor_summary is intentionally omitted — no SQL
-- definition exists for it anywhere in the design docs, and the
-- competitor heatmap is built directly from mv_hotel_market_position.

-- Latest price per hotel per source (dashboard market position, heatmap)
CREATE MATERIALIZED VIEW mv_hotel_market_position AS
SELECT DISTINCT ON (hotel_id, source)
    hotel_id, source, room_type, price_thb, checkin_date, scraped_at
FROM hotel_price_history
ORDER BY hotel_id, source, scraped_at DESC;

CREATE UNIQUE INDEX idx_mv_market_position_hotel_source ON mv_hotel_market_position (hotel_id, source);

-- Daily average/min/max price per hotel per source (trend charts)
CREATE MATERIALIZED VIEW mv_hotel_daily_avg_price AS
SELECT
    hotel_id,
    source,
    DATE_TRUNC('day', scraped_at) AS day,
    AVG(price_thb) AS avg_price_thb,
    MIN(price_thb) AS min_price_thb,
    MAX(price_thb) AS max_price_thb,
    COUNT(*) AS sample_count
FROM hotel_price_history
GROUP BY hotel_id, source, DATE_TRUNC('day', scraped_at);

CREATE UNIQUE INDEX idx_mv_daily_avg_price_hotel_source_day ON mv_hotel_daily_avg_price (hotel_id, source, day);

-- Win rate: % of days Gother's price was the best (or tied) vs. all sources
CREATE MATERIALIZED VIEW mv_hotel_win_rate AS
WITH daily_best AS (
    SELECT hotel_id, DATE_TRUNC('day', scraped_at) AS day, MIN(price_thb) AS best_price
    FROM hotel_price_history
    GROUP BY hotel_id, DATE_TRUNC('day', scraped_at)
),
gother_daily AS (
    SELECT hotel_id, DATE_TRUNC('day', scraped_at) AS day, MIN(price_thb) AS gother_price
    FROM hotel_price_history
    WHERE source = 'gother'
    GROUP BY hotel_id, DATE_TRUNC('day', scraped_at)
)
SELECT
    g.hotel_id,
    COUNT(*) FILTER (WHERE g.gother_price <= b.best_price) AS days_won,
    COUNT(*) AS days_total,
    ROUND(100.0 * COUNT(*) FILTER (WHERE g.gother_price <= b.best_price) / NULLIF(COUNT(*), 0), 1) AS win_rate_pct
FROM gother_daily g
JOIN daily_best b ON g.hotel_id = b.hotel_id AND g.day = b.day
GROUP BY g.hotel_id;

CREATE UNIQUE INDEX idx_mv_win_rate_hotel ON mv_hotel_win_rate (hotel_id);

-- Price by days-in-advance-of-checkin, per hotel per source (booking window chart)
CREATE MATERIALIZED VIEW mv_hotel_booking_window AS
SELECT
    hotel_id,
    source,
    (checkin_date - DATE(scraped_at)) AS days_in_advance,
    AVG(price_thb) AS avg_price_thb,
    MIN(price_thb) AS min_price_thb,
    COUNT(*) AS sample_count
FROM hotel_price_history
GROUP BY hotel_id, source, (checkin_date - DATE(scraped_at));

CREATE UNIQUE INDEX idx_mv_booking_window_hotel_source_days ON mv_hotel_booking_window (hotel_id, source, days_in_advance);

-- Hotels where Gother's latest price exceeds the best OTA price (rate parity violations)
CREATE MATERIALIZED VIEW mv_hotel_parity_violations AS
WITH latest AS (
    SELECT DISTINCT ON (hotel_id, source) hotel_id, source, price_thb, scraped_at
    FROM hotel_price_history
    ORDER BY hotel_id, source, scraped_at DESC
),
gother AS (
    SELECT hotel_id, price_thb AS gother_price FROM latest WHERE source = 'gother'
),
best_ota AS (
    SELECT hotel_id, MIN(price_thb) AS best_ota_price FROM latest WHERE source != 'gother' GROUP BY hotel_id
)
SELECT
    g.hotel_id,
    g.gother_price,
    b.best_ota_price,
    ROUND(100.0 * (g.gother_price - b.best_ota_price) / NULLIF(b.best_ota_price, 0), 1) AS gap_pct
FROM gother g
JOIN best_ota b ON g.hotel_id = b.hotel_id
WHERE g.gother_price > b.best_ota_price;

CREATE UNIQUE INDEX idx_mv_parity_violations_hotel ON mv_hotel_parity_violations (hotel_id);
