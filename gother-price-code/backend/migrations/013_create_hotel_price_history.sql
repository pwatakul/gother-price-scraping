-- Migration: Create hotel_price_history table (partitioned)
-- Description: REQ-002/REQ-005/ADR-002 — long-term, queryable price
-- history, separate from job-scoped scrape_results. Partitioned by
-- scraped_at (month) per REQ-005's scale/partitioning requirements.
-- Partition key must be part of the primary key for a partitioned table,
-- hence PK (id, scraped_at) rather than PK (id) alone.

CREATE TABLE hotel_price_history (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    hotel_id UUID NOT NULL REFERENCES hotels(id) ON DELETE CASCADE,
    source VARCHAR(50) NOT NULL,
    room_type VARCHAR(255) NOT NULL,
    price_thb DECIMAL(12, 2) NOT NULL,
    original_price DECIMAL(12, 2),
    currency VARCHAR(10),
    exchange_rate_id UUID NOT NULL REFERENCES currency_exchange_rates(id),
    meal_plan VARCHAR(100),
    cancellation VARCHAR(255),
    source_url TEXT,
    checkin_date DATE NOT NULL,
    checkout_date DATE NOT NULL,
    rooms SMALLINT NOT NULL,
    adults SMALLINT NOT NULL,
    scrape_job_id UUID REFERENCES scrape_jobs(id) ON DELETE SET NULL,
    scraped_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (id, scraped_at)
) PARTITION BY RANGE (scraped_at);

-- Indexes (inherited by all partitions)
CREATE INDEX idx_hph_hotel_scraped ON hotel_price_history (hotel_id, scraped_at DESC);
CREATE INDEX idx_hph_source_scraped ON hotel_price_history (source, scraped_at DESC);
CREATE INDEX idx_hph_hotel_checkin ON hotel_price_history (hotel_id, checkin_date);
CREATE INDEX idx_hph_hotel_src_chkin ON hotel_price_history (hotel_id, source, checkin_date, scraped_at DESC);
