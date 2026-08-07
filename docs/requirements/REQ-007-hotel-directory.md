---
title: "REQ-007: Global Hotel Directory"
type: requirement
version: "1.0"
date: 2026-08-09
status: Active
tags: [requirement, hotels, directory, pagination]
related: ["[[REQ-001-v1.3]]", "[[REQ-002-v1.1]]"]
---

# REQ-007: Global Hotel Directory
_Version: 1.0_

## Raw Requirement (plain language)
The user needs a single page that lists every hotel tracked anywhere in the system — not scoped to one hotel group — so they can see, search, filter, and track any hotel regardless of which group(s) it belongs to. This was requested directly (not from the original CEO brief) alongside REQ-002/REQ-003 work. First implemented 2026-08-08 without a formal requirement doc; this document formalizes it and adds a proper pagination spec, requested as a follow-up on 2026-08-09.

## Goal
Give the user one place to answer "what hotels do we track, and what's their latest price?" without needing to know which group a hotel lives in first.

## Acceptance Criteria
- [x] List every hotel in the system, one row per hotel, regardless of group membership
- [x] Filter by country and city (dropdowns populated from real distinct values, not hardcoded)
- [x] Free-text search by hotel name
- [x] Each row shows: name, city, country, which group(s) it belongs to, last known price + source + scraped time
- [x] Click through to a per-hotel detail page showing identity (HID/slug/supplier type) and a price trend chart
- [x] Export the current filtered list as CSV
- [ ] **Pagination is complete and robust** — see F-007 below, this is the gap being closed now

## Functional Requirements
| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| F-001 | `GET /hotels` — paginated, filtered listing (country/city/search/limit/offset), returns `{ hotels, total }` | High | Done |
| F-002 | `GET /hotels/:id` — hotel detail: identity + group memberships + price trend | High | Done |
| F-003 | `GET /hotels/countries`, `GET /hotels/cities?country=` — distinct-value endpoints for filter dropdowns | High | Done |
| F-004 | `GET /hotels/export` — CSV export honoring the current filters (not just the current page) | High | Done |
| F-005 | Frontend list page with search + country/city filters | High | Done |
| F-006 | Per-hotel detail page with 90-day price trend chart | High | Done |
| F-007 | **Full pagination**: page-size selector, numbered page controls (not just Previous/Next), current filters+page reflected in the URL (survives refresh/back-button/share-link) | High | **This pass** |

### F-007 detail — pagination
The original implementation (2026-08-08) had page state held only in React component state: a fixed page size of 25, and Previous/Next buttons with a "Page X of Y" label. Gaps being closed:
1. **Page-size selector** — 25/50/100 per page, user-selectable, resets to page 1 on change.
2. **Numbered page buttons** — direct jump to any nearby page, not just one step at a time; condensed with ellipsis when there are many pages (e.g. `1 2 3 … 42`).
3. **URL-synced state** — `country`, `city`, `q`, `page`, `pageSize` all reflected as query params (`/hotels?country=Thailand&page=3&pageSize=50`). Refreshing, using the browser back button, or sharing the link preserves the exact view. This also fixes a latent UX bug in the original version: changing a filter didn't reset to page 1, so a filter change could land on an out-of-range page showing no results.
4. Backend already supports arbitrary `limit`/`offset` and returns `total` — no backend change needed for F-007, this is frontend-only.

## Out of Scope
- Sorting by column (name/price/last-scraped) — not requested, not built
- Bulk actions (multi-select, bulk delete/export) — not requested
- Saved filter presets

## Dependencies
- `HotelDirectoryRepo::list` (`backend/src/db/repositories/hotel_directory_repo.rs`) — already paginates via `limit`/`offset` and returns a total count; no change needed.
- `react-router-dom`'s `useSearchParams` for URL state sync (already a dependency, used elsewhere for route params but not yet for query-string state).

## Change Log
| Version | Date | Change | Reason |
|---------|------|--------|--------|
| 1.0 | 2026-08-09 | Initial formal doc, retroactively covering the 2026-08-08 implementation, with F-007 (full pagination) as the active gap | User asked for the directory's pagination to be properly specified and completed |
