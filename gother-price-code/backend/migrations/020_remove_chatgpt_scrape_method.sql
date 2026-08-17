-- Migration: Remove 'chatgpt' from the scrape_method enum
-- Description: The ChatGPT scraper is removed — price sources are SerpAPI
-- (live Google Hotels data) and Gemini. Postgres has no DROP VALUE for
-- enums, so the type is recreated without 'chatgpt' and every dependent
-- column is re-pointed at it. Safe to do destructively here: no rows
-- reference 'chatgpt' (verified 0 scrape_jobs.method and 0
-- scrape_results.via_method before writing this migration).
--
-- Defaults must be dropped before the type swap and restored after, since
-- a column default is typed against the old enum.

ALTER TABLE scrape_jobs ALTER COLUMN method DROP DEFAULT;
ALTER TABLE scrape_results ALTER COLUMN via_method DROP DEFAULT;
ALTER TABLE scheduled_scrape_configs ALTER COLUMN method DROP DEFAULT;

ALTER TYPE scrape_method RENAME TO scrape_method_old;

CREATE TYPE scrape_method AS ENUM ('serpapi', 'gemini', 'both');

ALTER TABLE scrape_jobs
    ALTER COLUMN method TYPE scrape_method USING method::text::scrape_method;
ALTER TABLE scrape_results
    ALTER COLUMN via_method TYPE scrape_method USING via_method::text::scrape_method;
ALTER TABLE scheduled_scrape_configs
    ALTER COLUMN method TYPE scrape_method USING method::text::scrape_method;

ALTER TABLE scrape_jobs ALTER COLUMN method SET DEFAULT 'serpapi';
ALTER TABLE scrape_results ALTER COLUMN via_method SET DEFAULT 'serpapi';
ALTER TABLE scheduled_scrape_configs ALTER COLUMN method SET DEFAULT 'serpapi';

DROP TYPE scrape_method_old;
