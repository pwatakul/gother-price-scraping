-- Migration: Create hotel_group_members table
-- Description: Junction table for many-to-many hotel-group relationship

CREATE TABLE hotel_group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hotel_group_id UUID NOT NULL REFERENCES hotel_groups(id) ON DELETE CASCADE,
    hotel_id UUID NOT NULL REFERENCES hotels(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Prevent duplicate memberships
    UNIQUE(hotel_group_id, hotel_id)
);

-- Indexes for efficient lookups
CREATE INDEX idx_hotel_group_members_group_id ON hotel_group_members(hotel_group_id);
CREATE INDEX idx_hotel_group_members_hotel_id ON hotel_group_members(hotel_id);
