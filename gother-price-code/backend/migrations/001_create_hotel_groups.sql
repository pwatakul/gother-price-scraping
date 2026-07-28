-- Migration: Create hotel_groups table
-- Description: Stores hotel group collections

CREATE TABLE hotel_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for listing groups by creation date
CREATE INDEX idx_hotel_groups_created_at ON hotel_groups(created_at DESC);

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_hotel_groups_updated_at
    BEFORE UPDATE ON hotel_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
