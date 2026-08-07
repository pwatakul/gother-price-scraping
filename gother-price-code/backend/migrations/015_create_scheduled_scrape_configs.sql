-- Migration: Create scheduled_scrape_configs table
-- Description: REQ-002 F-003/F-004 — automated/scheduled scraping
-- configuration per hotel group. `method` reuses the existing
-- scrape_method enum (serpapi/chatgpt/gemini/both) so scheduled runs are
-- explicit about which scraper(s) to use, defaulting to serpapi.

CREATE TABLE scheduled_scrape_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hotel_group_id UUID NOT NULL REFERENCES hotel_groups(id) ON DELETE CASCADE,
    name VARCHAR(100),
    cron_expression VARCHAR(50) NOT NULL,
    lookahead_days INTEGER[] NOT NULL,
    los_variants INTEGER[] NOT NULL DEFAULT ARRAY[1],
    method scrape_method NOT NULL DEFAULT 'serpapi',
    rooms SMALLINT NOT NULL DEFAULT 1,
    adults SMALLINT NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_rooms CHECK (rooms > 0),
    CONSTRAINT valid_adults CHECK (adults > 0)
);

CREATE INDEX idx_ssc_hotel_group_id ON scheduled_scrape_configs (hotel_group_id);
CREATE INDEX idx_ssc_active ON scheduled_scrape_configs (is_active) WHERE is_active = TRUE;

CREATE TRIGGER update_scheduled_scrape_configs_updated_at
    BEFORE UPDATE ON scheduled_scrape_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
