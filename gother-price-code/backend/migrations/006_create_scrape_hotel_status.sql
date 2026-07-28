-- Migration: Create scrape_hotel_status table
-- Description: Tracks success/failure status per hotel per job

-- Create enum for hotel scrape status
CREATE TYPE hotel_scrape_status AS ENUM ('pending', 'processing', 'success', 'failed');

CREATE TABLE scrape_hotel_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scrape_job_id UUID NOT NULL REFERENCES scrape_jobs(id) ON DELETE CASCADE,
    hotel_id UUID NOT NULL REFERENCES hotels(id) ON DELETE CASCADE,
    status hotel_scrape_status NOT NULL DEFAULT 'pending',
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One status per hotel per job
    UNIQUE(scrape_job_id, hotel_id)
);

-- Indexes
CREATE INDEX idx_scrape_hotel_status_job_id ON scrape_hotel_status(scrape_job_id);
CREATE INDEX idx_scrape_hotel_status_status ON scrape_hotel_status(status);

-- Trigger for updated_at
CREATE TRIGGER update_scrape_hotel_status_updated_at
    BEFORE UPDATE ON scrape_hotel_status
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
