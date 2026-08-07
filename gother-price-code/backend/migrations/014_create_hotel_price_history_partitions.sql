-- Migration: Create initial hotel_price_history partitions
-- Description: Creates the current month + next 3 months of monthly
-- partitions, computed from the date this migration actually runs (not
-- hardcoded) so it stays correct regardless of when it's applied.
--
-- NOTE: no pg_partman/automation is wired up (out of scope for this pass,
-- see REQ-005 "Future Considerations"). New partitions must be created
-- manually (a new migration, same pattern as this one) before each
-- future month starts, or inserts for that month will fail.

DO $$
DECLARE
    partition_start DATE;
    partition_end DATE;
    partition_name TEXT;
    i INTEGER;
BEGIN
    FOR i IN 0..3 LOOP
        partition_start := date_trunc('month', CURRENT_DATE) + (i || ' months')::INTERVAL;
        partition_end := partition_start + INTERVAL '1 month';
        partition_name := 'hotel_price_history_' || to_char(partition_start, 'YYYY_MM');

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF hotel_price_history FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );
    END LOOP;
END $$;
