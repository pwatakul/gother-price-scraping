-- Seed: 20 real Thailand hotels for the demo
--
-- Replaces the "Demo Hotel N" / "Test Master Hotel" placeholders, which no
-- real price source can resolve — SerpAPI returns nothing for a hotel that
-- doesn't exist, so every scrape against them failed.
--
-- Every hotel below was verified against the live SerpAPI google_hotels
-- engine (2026-08-11, check-in 2026-08-25): each resolves to a real
-- property carrying prices. 13 of the 20 also return Agoda and/or Trip.com
-- rates, which are the only providers ADR-005 keeps; the rest resolve but
-- currently expose no named-provider rate, so they will legitimately show
-- "returned no rates" rather than fabricating a number.
--
-- Run explicitly (this is demo data, not schema — deliberately NOT a
-- migration):
--   docker exec -i hotel_scraper_db psql -U postgres -d hotel_scraper \
--     -v ON_ERROR_STOP=1 < scripts/seed-thailand-demo.sql

BEGIN;

-- 1. Remove the placeholder hotels. ON DELETE CASCADE clears their group
--    memberships, per-job scrape statuses and price history.
DELETE FROM hotels
WHERE name LIKE 'Demo Hotel %'
   OR name LIKE 'Test Master Hotel %';

-- 1b. Remove any hotel this script is about to insert, so re-running it is
--     idempotent. Without this, hotels that already existed under the same
--     name (Peninsula, Siam Kempinski, Anantara Riverside were seeded
--     earlier by hand) come out duplicated.
DELETE FROM hotels WHERE name IN (
    'Mandarin Oriental Bangkok', 'The Peninsula Bangkok',
    'Anantara Riverside Bangkok Resort', 'Siam Kempinski Hotel Bangkok',
    'The Sukhothai Bangkok', 'Banyan Tree Bangkok', 'Shangri-La Bangkok',
    'Conrad Bangkok', 'Amanpuri', 'Rosewood Phuket',
    'Katathani Phuket Beach Resort', 'JW Marriott Phuket Resort & Spa',
    'Four Seasons Resort Chiang Mai', 'Anantara Chiang Mai Resort',
    '137 Pillars House Chiang Mai', 'Four Seasons Resort Koh Samui',
    'Banyan Tree Samui', 'Rayavadee Krabi', 'Dusit Thani Krabi Beach Resort',
    'InterContinental Pattaya Resort'
);

-- 2. Remove every price row produced by the MockScraper fallback. Those
--    rows were written with source/via_method 'serpapi' and are otherwise
--    indistinguishable from real scrapes, so they cannot be filtered later
--    — they have to go before the first real scrape or they permanently
--    contaminate the trend, win-rate and booking-window analytics.
--    (Safe: at seed time no real scrape has run yet.)
TRUNCATE hotel_price_history;

-- 3. Insert the verified hotels and attach them to the demo group.
WITH demo_group AS (
    SELECT id FROM hotel_groups ORDER BY created_at LIMIT 1
),
seeded AS (
    INSERT INTO hotels (name, city, country, normalized_name)
    VALUES
        -- Bangkok
        ('Mandarin Oriental Bangkok',         'Bangkok',    'Thailand', 'mandarin oriental bangkok'),
        ('The Peninsula Bangkok',             'Bangkok',    'Thailand', 'the peninsula bangkok'),
        ('Anantara Riverside Bangkok Resort', 'Bangkok',    'Thailand', 'anantara riverside bangkok resort'),
        ('Siam Kempinski Hotel Bangkok',      'Bangkok',    'Thailand', 'siam kempinski hotel bangkok'),
        ('The Sukhothai Bangkok',             'Bangkok',    'Thailand', 'the sukhothai bangkok'),
        ('Banyan Tree Bangkok',               'Bangkok',    'Thailand', 'banyan tree bangkok'),
        ('Shangri-La Bangkok',                'Bangkok',    'Thailand', 'shangri-la bangkok'),
        ('Conrad Bangkok',                    'Bangkok',    'Thailand', 'conrad bangkok'),
        -- Phuket
        ('Amanpuri',                          'Phuket',     'Thailand', 'amanpuri'),
        ('Rosewood Phuket',                   'Phuket',     'Thailand', 'rosewood phuket'),
        ('Katathani Phuket Beach Resort',     'Phuket',     'Thailand', 'katathani phuket beach resort'),
        ('JW Marriott Phuket Resort & Spa',   'Phuket',     'Thailand', 'jw marriott phuket resort & spa'),
        -- Chiang Mai
        ('Four Seasons Resort Chiang Mai',    'Chiang Mai', 'Thailand', 'four seasons resort chiang mai'),
        ('Anantara Chiang Mai Resort',        'Chiang Mai', 'Thailand', 'anantara chiang mai resort'),
        ('137 Pillars House Chiang Mai',      'Chiang Mai', 'Thailand', '137 pillars house chiang mai'),
        -- Koh Samui
        ('Four Seasons Resort Koh Samui',     'Koh Samui',  'Thailand', 'four seasons resort koh samui'),
        ('Banyan Tree Samui',                 'Koh Samui',  'Thailand', 'banyan tree samui'),
        -- Krabi
        ('Rayavadee Krabi',                   'Krabi',      'Thailand', 'rayavadee krabi'),
        ('Dusit Thani Krabi Beach Resort',    'Krabi',      'Thailand', 'dusit thani krabi beach resort'),
        -- Pattaya
        ('InterContinental Pattaya Resort',   'Pattaya',    'Thailand', 'intercontinental pattaya resort')
    RETURNING id
)
INSERT INTO hotel_group_members (hotel_group_id, hotel_id)
SELECT demo_group.id, seeded.id FROM seeded, demo_group;

-- 4. The group was named for Bangkok only; it now spans six destinations.
UPDATE hotel_groups
SET name = 'Thailand Demo Hotels'
WHERE id = (SELECT id FROM hotel_groups ORDER BY created_at LIMIT 1)
  AND name = 'Bangkok Demo Hotels';

COMMIT;

-- Verify
SELECT city, count(*) AS hotels FROM hotels GROUP BY city ORDER BY city;
SELECT count(*) AS total_hotels FROM hotels;
SELECT count(*) AS remaining_price_rows FROM hotel_price_history;
