-- Migration: Add device dimension to hotel_price_history
-- Description: REQ-008 F-005 — `device` already existed on scrape_jobs and
-- scrape_results (migrations 008/009) but was never carried into
-- hotel_price_history, so the dimension was invisible to the trend API,
-- the materialized views and the hotel detail page. Adding it here makes
-- mobile-vs-desktop cross-tracking analysable. Existing rows default to
-- 'desktop', which is accurate: the scheduler only ever produced desktop
-- jobs before REQ-008. ALTER on the partitioned parent propagates to all
-- existing and future partitions.

ALTER TABLE hotel_price_history
    ADD COLUMN device device_type NOT NULL DEFAULT 'desktop';

CREATE INDEX idx_hph_hotel_device_chkin
    ON hotel_price_history (hotel_id, device, checkin_date, scraped_at DESC);
