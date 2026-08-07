---
title: Changelog
type: changelog
tags: [changelog]
---

# Changelog
All notable changes to this project will be documented here.
Format: `[version] - YYYY-MM-DD`

---

## [Unreleased]
### Added
- Price history (`hotel_price_history`, partitioned by month) + dual-write from scrape worker — [[REQ-002-v1.1]]
- Scheduled scraping via cron-style configs + background scheduler — [[REQ-002-v1.1]]
- Price history query + export API (csv/json) — [[REQ-002-v1.1]]
- Materialized views (market position, daily avg price, win rate, booking window, parity violations) + refresh-after-every-job — [[REQ-005-v1.2]]
- `hotel_price_history` partition auto-creation — daily idempotent loop, no `pg_partman` dependency — [[REQ-005-v1.2]] F-002
- Market analytics dashboard (overview card, trend chart, market position table, competitor heatmap, date-range filter) — [[REQ-003-v1.0]]
- Global "All Hotels" directory page — country/city filters, search, URL-synced pagination, export — [[REQ-007-hotel-directory]]
- Per-hotel and per-group (across all jobs) price-history export
- Full paginated/filterable raw price-history table on hotel detail page
- ChatGPT scraper (Method 1, bonus) alongside SerpAPI
- Master-hotel-list import format (HID/UPDATE URL/SLUG) exposed in the UI via a format toggle on the group import dialog — previously API-only
- Scheduled-scrape-config management UI (list/create/delete) on the hotel group detail page — previously API-only

### Changed
- Scraper dispatch refactored to a pluggable adapter/registry pattern (`ScraperFactory` trait + `default_registry()`)
- Sidebar navigation reorganized: collapsible "Hotels" section (New Price Search / All Hotels / collapsible Analytics submenu); Import/Export tab removed

### Fixed
-

---

## [0.1.0] - 2026-04-22
### Added
- Initial project setup
- Full backend: Rust/Axum REST API, PostgreSQL schema (6 migrations), RabbitMQ worker, Redis caching
- SerpAPI and Gother API scrapers with mock fallback
- Excel import/export (hotel list template + price comparison report)
- Frontend: React + Vite + Tailwind, Dashboard, HotelGroupDetail, ReportView pages
- Docker Compose setup for full stack

---

> [!NOTE]
> Versioning: `MAJOR.MINOR.PATCH` — MAJOR: breaking change; MINOR: new feature; PATCH: bug fix.
> Commit prefixes: `feat:` `fix:` `req:` `doc:` `refactor:`
