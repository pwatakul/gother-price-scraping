-- Migration: Create scrape_jobs table
-- Description: Stores scrape job requests with parameters

-- Create enum for job status
CREATE TYPE scrape_job_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'cancelled');

CREATE TABLE scrape_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hotel_group_id UUID NOT NULL REFERENCES hotel_groups(id) ON DELETE CASCADE,
    checkin_date DATE NOT NULL,
    checkout_date DATE NOT NULL,
    rooms INTEGER NOT NULL DEFAULT 1,
    adults INTEGER NOT NULL DEFAULT 2,
    status scrape_job_status NOT NULL DEFAULT 'pending',
    force_refresh BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    
    -- Validation
    CONSTRAINT valid_dates CHECK (checkout_date > checkin_date),
    CONSTRAINT valid_rooms CHECK (rooms > 0),
    CONSTRAINT valid_adults CHECK (adults > 0)
);

-- Indexes
CREATE INDEX idx_scrape_jobs_hotel_group_id ON scrape_jobs(hotel_group_id);
CREATE INDEX idx_scrape_jobs_status ON scrape_jobs(status);
CREATE INDEX idx_scrape_jobs_created_at ON scrape_jobs(created_at DESC);
CREATE INDEX idx_scrape_jobs_group_created ON scrape_jobs(hotel_group_id, created_at DESC);
