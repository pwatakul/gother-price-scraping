-- Migration: Add master hotel-list import fields
-- Description: REQ-001-v1.2 F-021 — real hotel-list-2200.csv is HID-keyed
-- (HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country), which
-- does not fit the existing hotel_name/city/country-only import path.
-- These columns are nullable so hotels created via the existing 3-column
-- import are unaffected.

ALTER TABLE hotels
    ADD COLUMN hid BIGINT,
    ADD COLUMN slug VARCHAR(255),
    ADD COLUMN update_url TEXT,
    ADD COLUMN supplier_type VARCHAR(50);

CREATE UNIQUE INDEX idx_hotels_hid ON hotels(hid) WHERE hid IS NOT NULL;
