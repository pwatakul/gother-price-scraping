-- Migration: Make the "at least one booking window" check actually bite
-- Description: Migration 024 used
--   CHECK (array_length(search_days_ahead, 1) >= 1)
-- which does not reject an empty array. `array_length('{}', 1)` returns
-- NULL, not 0, and a CHECK constraint that evaluates to NULL *passes* —
-- so `search_days_ahead = '{}'` was accepted, leaving a group whose saved
-- search would queue zero jobs.
--
-- `cardinality()` returns 0 for an empty array, so the comparison is a
-- real boolean and the constraint rejects it.

-- Repair any group that slipped through before fixing the rule, so the
-- constraint can be added without failing validation.
UPDATE hotel_groups
SET search_days_ahead = ARRAY[7]
WHERE cardinality(search_days_ahead) = 0;

ALTER TABLE hotel_groups DROP CONSTRAINT valid_search_days_ahead;

ALTER TABLE hotel_groups
    ADD CONSTRAINT valid_search_days_ahead
        CHECK (cardinality(search_days_ahead) >= 1);
