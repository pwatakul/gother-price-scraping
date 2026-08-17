-- Migration: Drop dead configuration columns from scheduled_scrape_configs
-- Description: REQ-008 / ADR-006 — the booking-window × device grid and its
-- stay parameters are system constants in worker/scheduler.rs, not per-config
-- input, so the scheduler stopped reading these four columns. Leaving them in
-- place meant the API still required `lookahead_days` and the UI still
-- collected values that were silently ignored. These columns hold
-- configuration, not observations, so nothing analysable is lost; existing
-- configs keep running on cron_expression, method and is_active.

ALTER TABLE scheduled_scrape_configs
    DROP COLUMN lookahead_days,
    DROP COLUMN los_variants,
    DROP COLUMN rooms,
    DROP COLUMN adults;
