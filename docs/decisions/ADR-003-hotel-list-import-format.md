---
title: "ADR-003: Hotel-list import — two separate imports, HID as primary key"
type: decision
date: 2026-08-04
status: Accepted
tags: [adr, excel, import, hotels]
---

# ADR-003: Hotel-list import — two separate imports, HID as primary key

## Context
The real CEO-brief hotel list (`docs/data/hotel-list-2200.csv`) uses columns `No, HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH`. This does not match the existing F-002 spec (`hotel_name, city, country, checkin_date, checkout_date, rooms, adults, currency`), and the master list has no search-parameter columns at all (checkin/checkout/rooms/adults are per-job, not per-hotel-in-a-list).

## Decision
Two separate imports, not one combined format:
1. **Master hotel-list import** (`POST /hotel-groups/:id/import-master`, `ExcelReader::read_master_hotel_list`) — HID-keyed, matches the real CSV shape. Populates `hotels` (new `hid`/`slug`/`update_url`/`supplier_type` columns, migration 007) via find-or-create-by-HID.
2. **Per-job search-parameter overrides** (`POST /scrape-jobs/with-overrides`, `ExcelReader::read_job_hotel_overrides`) — optional sheet keyed by `hid` (name-only rows are not resolved — ambiguous), carrying checkin_date/checkout_date/rooms/adults/currency. Blank cells fall back to job-level defaults (`JobDefaults`, `merge_job_params`) — this is where F-002's original per-row override requirement actually lives.

The existing plain `hotel_name/city/country` import (`POST /hotel-groups/:id/import`, `ExcelReader::read_hotels`) is left untouched for manual/ad-hoc group creation.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| Two separate imports (chosen) | Each format stays simple; existing 3-column import is a zero-risk no-op change | Two endpoints/readers to maintain |
| Single combined format (add search-param columns to the master list) | One file for everything | Master list is a fixed 2200-row asset from the CEO brief — extending its schema means diverging from the source-of-truth file |
| Extend the 3-column format with an optional HID column | Minimal change | Doesn't solve the real gap: the master list's columns (SLUG, Supplier-or-Direct, UPDATE URL) still have nowhere to go |

## Consequences
### Positive
- The master hotel-list import can ingest the real 2200-hotel CSV unmodified.
- JobDefaults fallback (per-hotel search-param overrides) is decoupled from hotel identity, matching how the brief actually structures the data.

### Negative / Trade-offs
- Two import paths mean two things to keep in sync if the schema changes again.
- Override rows without a `hid` are silently dropped rather than resolved by name — acceptable given HID is the reliable key, but worth surfacing in the UI (not yet done).

## Related
- REQ: [[REQ-001-v1.3]]
- Data: `docs/data/hotel-list-2200.csv`
