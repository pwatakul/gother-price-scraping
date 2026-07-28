-- Migration: Create hotels table
-- Description: Stores individual hotel records

CREATE TABLE hotels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    city VARCHAR(100) NOT NULL,
    country VARCHAR(100) NOT NULL,
    normalized_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for searching hotels
CREATE INDEX idx_hotels_normalized_name ON hotels(normalized_name);
CREATE INDEX idx_hotels_city_country ON hotels(city, country);

-- Trigger for updated_at
CREATE TRIGGER update_hotels_updated_at
    BEFORE UPDATE ON hotels
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
