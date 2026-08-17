-- Migration: Rebuild mv_hotel_booking_window with the device dimension
-- Description: REQ-008 F-006 — the booking-window view (migration 016)
-- buckets prices by days-in-advance but had no device in its grouping key,
-- so the mandatory mobile/desktop pair collapsed into a single average.
-- A materialized view's column list can't be altered in place, so it is
-- dropped and recreated; it repopulates from hotel_price_history on the
-- next refresh, so no history is lost. The unique index is required for
-- the existing REFRESH MATERIALIZED VIEW CONCURRENTLY path.

DROP MATERIALIZED VIEW mv_hotel_booking_window;

CREATE MATERIALIZED VIEW mv_hotel_booking_window AS
SELECT
    hotel_id,
    source,
    device,
    (checkin_date - DATE(scraped_at)) AS days_in_advance,
    AVG(price_thb) AS avg_price_thb,
    MIN(price_thb) AS min_price_thb,
    COUNT(*) AS sample_count
FROM hotel_price_history
GROUP BY hotel_id, source, device, (checkin_date - DATE(scraped_at));

CREATE UNIQUE INDEX idx_mv_booking_window_hotel_source_device_days
    ON mv_hotel_booking_window (hotel_id, source, device, days_in_advance);
