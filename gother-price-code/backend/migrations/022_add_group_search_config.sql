-- Migration: Per-group saved price-search config
-- Description: ADR-012 — a group carries its own search settings so a
-- price search can be edited once and re-run with one click, instead of
-- being re-entered every time.
--
-- Dates are stored as a days-ahead offset, not calendar dates: a saved
-- config holding literal dates goes stale silently (a search saved today
-- starts querying a past date next month), whereas "7 days ahead" stays
-- valid forever. This also matches how the scheduler's booking windows
-- already work.
--
-- Defaults reproduce the previous form defaults exactly, so existing
-- groups keep behaving the same way without anyone editing them.

ALTER TABLE hotel_groups
    ADD COLUMN search_method       scrape_method NOT NULL DEFAULT 'serpapi',
    ADD COLUMN search_days_ahead   INTEGER       NOT NULL DEFAULT 7,
    ADD COLUMN search_los_variants INTEGER[]     NOT NULL DEFAULT ARRAY[1],
    ADD COLUMN search_rooms        SMALLINT      NOT NULL DEFAULT 1,
    ADD COLUMN search_adults       SMALLINT      NOT NULL DEFAULT 2,
    ADD CONSTRAINT valid_search_days_ahead CHECK (search_days_ahead >= 0),
    ADD CONSTRAINT valid_search_rooms      CHECK (search_rooms > 0),
    ADD CONSTRAINT valid_search_adults     CHECK (search_adults > 0);

-- The group is now the single source of truth for which scraper to use.
-- Leaving `method` on the schedule would be a second place to set the same
-- thing, silently diverging from the group — the dead-config problem
-- migration 019 already had to correct once.
ALTER TABLE scheduled_scrape_configs DROP COLUMN method;
