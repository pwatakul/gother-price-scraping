---
title: Data Model
type: design
version: "1.0"
updated: 2026-04-22
tags: [design, database, schema]
---

# Data Model

## Overview
Six PostgreSQL tables support the hotel price comparison workflow. `hotel_groups` and `hotels` are the core reference data; `hotel_group_members` is a many-to-many junction. `scrape_jobs` stores each user-initiated price fetch request; `scrape_hotel_status` tracks per-hotel progress within a job; `scrape_results` stores the raw price records returned by each OTA source.

All primary keys are UUID v4. Timestamps are `TIMESTAMPTZ`. `updated_at` columns are auto-maintained by a `BEFORE UPDATE` trigger.

## Entities

### hotel_groups
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | UUID | Yes | Primary key (`gen_random_uuid()`) |
| name | VARCHAR(255) | Yes | Display name for the group |
| description | TEXT | No | Optional notes |
| created_at | TIMESTAMPTZ | Yes | Auto-set to NOW() |
| updated_at | TIMESTAMPTZ | Yes | Auto-updated by trigger |

### hotels
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | UUID | Yes | Primary key |
| name | VARCHAR(255) | Yes | Hotel display name (as imported) |
| city | VARCHAR(100) | Yes | City name |
| country | VARCHAR(100) | Yes | Country name |
| normalized_name | VARCHAR(255) | Yes | Lowercase, stripped ("hotel"/"resort" removed) — used for cross-OTA matching |
| created_at | TIMESTAMPTZ | Yes | Auto-set |
| updated_at | TIMESTAMPTZ | Yes | Auto-updated |

### hotel_group_members
Junction table linking hotels to groups (many-to-many).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | UUID | Yes | Primary key |
| hotel_group_id | UUID | Yes | FK → hotel_groups(id) CASCADE DELETE |
| hotel_id | UUID | Yes | FK → hotels(id) CASCADE DELETE |
| created_at | TIMESTAMPTZ | Yes | Auto-set |

> [!NOTE]
> Unique constraint on `(hotel_group_id, hotel_id)` prevents duplicate memberships.

### scrape_jobs
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | UUID | Yes | Primary key |
| hotel_group_id | UUID | Yes | FK → hotel_groups(id) CASCADE DELETE |
| checkin_date | DATE | Yes | Check-in date |
| checkout_date | DATE | Yes | Check-out date (must be > checkin_date) |
| rooms | INTEGER | Yes | Number of rooms (must be > 0, default 1) |
| adults | INTEGER | Yes | Number of adults (must be > 0, default 2) |
| status | scrape_job_status | Yes | Enum: `pending` / `processing` / `completed` / `failed` / `cancelled` |
| force_refresh | BOOLEAN | Yes | Skip Redis cache when true (default false) |
| created_at | TIMESTAMPTZ | Yes | Auto-set |
| completed_at | TIMESTAMPTZ | No | Set when job reaches completed/failed/cancelled |

### scrape_hotel_status
Tracks the per-hotel scraping status within a job. One row per (job, hotel) pair.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | UUID | Yes | Primary key |
| scrape_job_id | UUID | Yes | FK → scrape_jobs(id) CASCADE DELETE |
| hotel_id | UUID | Yes | FK → hotels(id) CASCADE DELETE |
| status | hotel_scrape_status | Yes | Enum: `pending` / `processing` / `success` / `failed` |
| retry_count | INTEGER | Yes | Number of retries attempted (default 0) |
| error_message | TEXT | No | Last error string if status = failed |
| created_at | TIMESTAMPTZ | Yes | Auto-set |
| updated_at | TIMESTAMPTZ | Yes | Auto-updated |

> [!NOTE]
> Unique constraint on `(scrape_job_id, hotel_id)`. This table drives the progress counters exposed by the API.

### scrape_results
Raw price records returned by OTA scrapers for a given job + hotel.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | UUID | Yes | Primary key |
| scrape_job_id | UUID | Yes | FK → scrape_jobs(id) CASCADE DELETE |
| hotel_id | UUID | Yes | FK → hotels(id) CASCADE DELETE |
| source | VARCHAR(50) | Yes | OTA identifier: `agoda`, `booking`, `gother`, `trip.com`, `official`, etc. |
| room_type | VARCHAR(255) | Yes | Normalized room type label |
| price_thb | DECIMAL(12,2) | Yes | Price in Thai Baht |
| original_price | DECIMAL(12,2) | No | Price in original currency |
| currency | VARCHAR(10) | No | Original currency code (e.g. USD, THB) |
| meal_plan | VARCHAR(100) | No | Normalized meal plan label (e.g. "Breakfast Included") |
| cancellation | VARCHAR(255) | No | Cancellation policy string |
| source_url | TEXT | No | Deep link to the OTA listing |
| scraped_at | TIMESTAMPTZ | Yes | Auto-set to NOW() |

## Relationships
- `hotel_groups` has many `hotels` (through `hotel_group_members`)
- `hotels` belongs to many `hotel_groups` (through `hotel_group_members`)
- `hotel_groups` has many `scrape_jobs`
- `scrape_jobs` has many `scrape_hotel_status` (one per hotel in the group)
- `scrape_jobs` has many `scrape_results` (one per OTA source per hotel)
- `hotels` has many `scrape_hotel_status`
- `hotels` has many `scrape_results`

## Entity Relationship Diagram
![[erd-v1.png]]

## Notes

> [!NOTE]
> `normalized_name` is generated by `Hotel::normalize_name()` in Rust: lowercase, trim, strip "hotel" and "resort", collapse double spaces. Used by `find_or_create` to avoid duplicates during Excel import.

- `scrape_results` is append-only — multiple rows per `(job, hotel, source)` are possible if a source returns multiple room types
- All CASCADE DELETE ensures deleting a `hotel_group` or `hotel` cleans up all child records
- `scrape_hotel_status.status` enum flow: `pending → processing → success | failed`
