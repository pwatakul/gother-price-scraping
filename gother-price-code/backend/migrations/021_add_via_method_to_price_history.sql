-- Migration: Record which scraper actually produced each price row
-- Description: ADR-011 — under method=both, SerpAPI is authoritative and
-- Gemini runs only to fill total blanks. Gemini is an AI estimate, not a
-- scrape, and has been observed quoting three different OTAs at one
-- identical (wrong) price. Without provenance those rows are
-- indistinguishable from real scrapes in every analytics view, which is
-- what forced truncating the earlier mock data wholesale instead of
-- filtering it.
--
-- VARCHAR rather than the scrape_method enum on purpose: this records the
-- actual *producer* ('serpapi'/'gemini'/'gother'/'mock'), and the enum has
-- no 'gother' variant. Note scrape_results.via_method stores the job's
-- requested method instead — a pre-existing imprecision not copied here.
--
-- The default correctly backfills existing rows: every price written
-- before this migration came from SerpAPI.

ALTER TABLE hotel_price_history
    ADD COLUMN via_method VARCHAR(20) NOT NULL DEFAULT 'serpapi';

CREATE INDEX idx_hph_via_method ON hotel_price_history (via_method);
