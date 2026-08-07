-- Migration: Add dimension/evidence columns to scrape_results
-- Description: REQ-001-v1.2 F-023/F-024 (device, login-state), F-025
-- (WHO ID for Gother rates), F-004 (los_variants -> los_nights per result),
-- and via_method to distinguish which scraper produced a row when
-- method='both'. Defaults preserve existing rows.

ALTER TABLE scrape_results
    ADD COLUMN los_nights INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN device device_type NOT NULL DEFAULT 'desktop',
    ADD COLUMN login_state login_state_type NOT NULL DEFAULT 'public',
    ADD COLUMN who_id VARCHAR(100),
    ADD COLUMN via_method scrape_method NOT NULL DEFAULT 'serpapi';
