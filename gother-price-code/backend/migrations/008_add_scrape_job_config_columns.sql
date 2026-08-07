-- Migration: Add scraping method / dimension columns to scrape_jobs
-- Description: REQ-001-v1.2 F-020 (ChatGPT method), F-023/F-024 (device,
-- login-state dimensions), F-004 (los_variants). Defaults preserve current
-- behavior for existing rows and for callers that don't set these fields.

CREATE TYPE scrape_method AS ENUM ('serpapi', 'chatgpt', 'both');
CREATE TYPE device_type AS ENUM ('desktop', 'mobile_web');
CREATE TYPE login_state_type AS ENUM ('public', 'member');

ALTER TABLE scrape_jobs
    ADD COLUMN method scrape_method NOT NULL DEFAULT 'serpapi',
    ADD COLUMN los_variants INTEGER[] NOT NULL DEFAULT ARRAY[1],
    ADD COLUMN device device_type NOT NULL DEFAULT 'desktop',
    ADD COLUMN login_state login_state_type NOT NULL DEFAULT 'public';
