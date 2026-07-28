-- ===========================================
-- Hotel Price Scraper - Sample Data
-- ===========================================
-- Run this after migrations to add sample data for testing

-- Sample Hotel Group
INSERT INTO hotel_groups (id, name, description)
VALUES 
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'Bangkok Luxury Hotels', 'Top luxury hotels in Bangkok for Q2 2026 comparison'),
  ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'Phuket Beach Resorts', 'Beach resorts in Phuket area')
ON CONFLICT DO NOTHING;

-- Sample Hotels
INSERT INTO hotels (id, name, city, country, normalized_name)
VALUES
  ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'Dusit Thani Bangkok', 'Bangkok', 'Thailand', 'dusit thani bangkok'),
  ('c1eebc99-9c0b-4ef8-bb6d-6bb9bd380a12', 'Mandarin Oriental Bangkok', 'Bangkok', 'Thailand', 'mandarin oriental bangkok'),
  ('c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a13', 'The Peninsula Bangkok', 'Bangkok', 'Thailand', 'peninsula bangkok'),
  ('c3eebc99-9c0b-4ef8-bb6d-6bb9bd380a14', 'Shangri-La Hotel Bangkok', 'Bangkok', 'Thailand', 'shangri-la bangkok'),
  ('c4eebc99-9c0b-4ef8-bb6d-6bb9bd380a15', 'Four Seasons Hotel Bangkok', 'Bangkok', 'Thailand', 'four seasons bangkok'),
  ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a21', 'Amanpuri Phuket', 'Phuket', 'Thailand', 'amanpuri phuket'),
  ('d1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'Banyan Tree Phuket', 'Phuket', 'Thailand', 'banyan tree phuket'),
  ('d2eebc99-9c0b-4ef8-bb6d-6bb9bd380a23', 'Trisara Phuket', 'Phuket', 'Thailand', 'trisara phuket')
ON CONFLICT DO NOTHING;

-- Add hotels to groups
INSERT INTO hotel_group_members (hotel_group_id, hotel_id)
VALUES
  -- Bangkok Luxury Hotels group
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'),
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'c1eebc99-9c0b-4ef8-bb6d-6bb9bd380a12'),
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a13'),
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'c3eebc99-9c0b-4ef8-bb6d-6bb9bd380a14'),
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'c4eebc99-9c0b-4ef8-bb6d-6bb9bd380a15'),
  -- Phuket Beach Resorts group
  ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'd0eebc99-9c0b-4ef8-bb6d-6bb9bd380a21'),
  ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'd1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22'),
  ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'd2eebc99-9c0b-4ef8-bb6d-6bb9bd380a23')
ON CONFLICT DO NOTHING;

-- Confirmation
SELECT 'Sample data loaded successfully!' as status;
SELECT COUNT(*) as hotel_groups_count FROM hotel_groups;
SELECT COUNT(*) as hotels_count FROM hotels;
SELECT COUNT(*) as group_members_count FROM hotel_group_members;
