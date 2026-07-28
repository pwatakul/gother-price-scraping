-- Migration: Create scrape_results table
-- Description: Stores price data from each OTA source

CREATE TABLE scrape_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scrape_job_id UUID NOT NULL REFERENCES scrape_jobs(id) ON DELETE CASCADE,
    hotel_id UUID NOT NULL REFERENCES hotels(id) ON DELETE CASCADE,
    source VARCHAR(50) NOT NULL,  -- e.g., 'agoda', 'booking', 'gother', 'official'
    room_type VARCHAR(255) NOT NULL,
    price_thb DECIMAL(12, 2) NOT NULL,
    original_price DECIMAL(12, 2),
    currency VARCHAR(10),
    meal_plan VARCHAR(100),
    cancellation VARCHAR(255),
    source_url TEXT,
    scraped_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_scrape_results_job_id ON scrape_results(scrape_job_id);
CREATE INDEX idx_scrape_results_hotel_id ON scrape_results(hotel_id);
CREATE INDEX idx_scrape_results_job_hotel ON scrape_results(scrape_job_id, hotel_id);
CREATE INDEX idx_scrape_results_source ON scrape_results(source);
