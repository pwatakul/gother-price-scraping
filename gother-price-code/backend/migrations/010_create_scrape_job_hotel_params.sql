-- Migration: Create scrape_job_hotel_params table
-- Description: REQ-001-v1.2 F-002 — optional per-hotel search-parameter
-- overrides for a scrape job (checkin/checkout/rooms/adults/currency).
-- Absence of a row for a given (scrape_job_id, hotel_id) means "use the
-- job-level defaults" (JobDefaults fallback).

CREATE TABLE scrape_job_hotel_params (
    scrape_job_id UUID NOT NULL REFERENCES scrape_jobs(id) ON DELETE CASCADE,
    hotel_id UUID NOT NULL REFERENCES hotels(id) ON DELETE CASCADE,
    checkin_date DATE,
    checkout_date DATE,
    rooms INTEGER,
    adults INTEGER,
    currency VARCHAR(10),

    PRIMARY KEY (scrape_job_id, hotel_id),
    CONSTRAINT valid_override_dates CHECK (
        checkout_date IS NULL OR checkin_date IS NULL OR checkout_date > checkin_date
    ),
    CONSTRAINT valid_override_rooms CHECK (rooms IS NULL OR rooms > 0),
    CONSTRAINT valid_override_adults CHECK (adults IS NULL OR adults > 0)
);
