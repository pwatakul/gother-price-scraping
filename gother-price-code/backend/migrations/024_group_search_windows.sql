-- Migration: Saved search takes several booking windows, one night each
-- Description: The config had its two dimensions the wrong way round —
-- one check-in date at several stay lengths, when what's wanted is
-- several check-in dates at one night each.
--
-- A single manual window also produced series nobody could compare: real
-- runs created +30 and +35, which sit apart from the scheduler's
-- +1/+3/+7/+14/+30 (see ADR-013). Selecting from the standard set means a
-- manual run reinforces the same windows the scheduler produces.
--
-- Existing values are preserved as one-element arrays — nobody's setting
-- is silently widened.

-- The CHECK and DEFAULT are both typed against the scalar, so they have to
-- go before the type can change.
ALTER TABLE hotel_groups DROP CONSTRAINT valid_search_days_ahead;
ALTER TABLE hotel_groups ALTER COLUMN search_days_ahead DROP DEFAULT;

ALTER TABLE hotel_groups
    ALTER COLUMN search_days_ahead TYPE INTEGER[]
    USING ARRAY[search_days_ahead];

ALTER TABLE hotel_groups
    ALTER COLUMN search_days_ahead SET DEFAULT ARRAY[7],
    ADD CONSTRAINT valid_search_days_ahead
        CHECK (array_length(search_days_ahead, 1) >= 1);

-- Length of stay is now a constant (1 night), matching the scheduler's
-- STANDARD_LOS_NIGHTS. A stored setting that can never vary is the
-- dead-surface pattern already removed in migrations 019 and 022.
ALTER TABLE hotel_groups DROP COLUMN search_los_variants;
