---
title: Project Context
type: context
updated: 2026-08-08
tags: [context, project-status]
---

# Project: Gother Market Intelligence Platform
_Formerly: Price Scraper_

## What this project does
A **market intelligence platform** for Gother's Product Owner and Business Development team. The platform scrapes hotel (and future: experience, flight) prices from competing OTAs, stores the history permanently, and surfaces actionable analytics — showing where Gother wins and loses on price, and how that changes over time.

Built as a competition entry for **Gother Challenge 2026** and designed to evolve into a production business tool.

## Platform Vision
```
Phase 1 (current) → Hotel price comparison + history + analytics
Phase 2 (planned)  → Experience / Activity price comparison
Phase 3 (future)   → Flight price comparison
```
All phases share the same job queue, price history store, and analytics dashboard.

## Current Status
- Version: v0.3 (REQ-001 core, REQ-002 price history/scheduling, REQ-003 analytics dashboard, REQ-005 data platform incl. partition automation, REQ-007 hotel directory — all implemented)
- Active Sprint: [[sprint-03]] (frontend/analytics work substantially complete ahead of schedule — see note below); [[sprint-04]] (demo polish) up next
- Branch: main
- Phase: Implementation — functionality largely complete; Part B (Cloud Run deployment) suspended indefinitely by user directive to focus on functionality first
- **Submission Deadline: Aug 17, 2026**
- Prize target: ฿120,000 (1st place)

> [!NOTE]
> **2026-08-08**: Google Cloud Run deployment (Part B) suspended indefinitely per user directive — focus is on functionality. In this pass: REQ-002 (price history + scheduled scraping) and REQ-003 (analytics dashboard) fully implemented; scraper dispatch refactored into a pluggable adapter/registry pattern (`ScraperFactory` trait + `default_registry()` in `backend/src/scraper/registry.rs`); new global "All Hotels" page (REQ-007, `backend/src/api/handlers/hotel_directory.rs` + `frontend/src/pages/HotelsList.tsx`) with country/city filtering, search, numbered pagination (URL-synced), and export; sidebar reorganized (collapsible "Hotels" section: New Price Search / All Hotels / collapsible Analytics submenu; Import/Export tab removed); data export added for both per-hotel and per-group (across all jobs) price history; hotel detail page now shows the full paginated, filterable raw price-history table, not just the aggregated trend chart. Finally, [[REQ-005-v1.2]] closed the last flagged data-platform gap: `hotel_price_history` partition auto-creation is now automated via a daily background loop (`backend/src/worker/partition_manager.rs`), application-level and idempotent — no `pg_partman` dependency added, consistent with the standing decision against that extension. Verified live via docker: partitions ensured on startup, idempotent on restart (`pg_inherits` unchanged, no errors).
>
> **2026-08-04**: REQ-001's core scraping/API/Excel/evidence scope (F-002, F-004, F-011, F-020, F-021–F-027) implemented — see [[REQ-001-v1.3]], [[ADR-001-scraper-choice]], [[ADR-003-hotel-list-import-format]], [[ADR-005-provider-specific-scraping]]. Migrations 007–010 added. Wink still has no real data source (stubbed blank), device/login-state dimensions are recorded but not verified to change actual scrape behavior, Gother WHO ID field not confirmed to exist upstream — see REQ-001-v1.3's Open Risks section (still open as of 2026-08-08).
>
> **2026-07-30**: Deadline moved to Aug 17, 2026 (from May 15, 2026). Descoped, not schedulable by Aug 17: [[REQ-004-v1.0]] (multi-product/experiences) — its own doc gates on hotels/price history being stable, only true after Sprint 02. [[REQ-006-v1.0]] (forecasting) — requires 6 months of accumulated `hotel_price_history`, which cannot exist by Aug 17 regardless of engineering effort. Both remain descoped as of 2026-08-08.
>
> **2026-07-27** (prior note): Raw brief data (`docs/raw/Req price scrapping - 17 July 26.xlsx`) parsed into [[REQ-001-v1.2]]. Managed data assets: `docs/data/hotel-list-2200.csv` (the 2200-hotel list) and `docs/data/example-raw-data-schema.md` + `docs/data/example-raw-data-sample.csv` (target scraper output schema).

## Gaps vs Competition Brief (must fix before demo)

| # | Gap | Current State | Required by Brief | REQ |
|---|-----|--------------|-------------------|-----|
| 1 | **Excel per-row search params** | checkin/checkout/rooms/adults set at job level (same for all hotels) | Each Excel ROW can have its own checkin_date, checkout_date, rooms, adults | [[REQ-001-v1.2]] |
| 2 | **Method 1: ChatGPT scraper** | Not implemented | Bonus points for implementing ChatGPT prompt method alongside SerpAPI | new REQ |
| 3 | **Evidence column in results** | source_url stored but not prominently displayed | URL + scraped_at timestamp must be visible in UI table and Excel export | [[REQ-001-v1.2]] |
| 4 | **Apple-to-apple comparison** | All results shown, no filtering by matching room type | Judges expect same room type compared across sources | [[REQ-001-v1.2]] |
| 5 | **TripAdvisor API** | Not integrated | Example prompt uses TripAdvisor as a source — investigate if accessible | new REQ |
| 6 | **Collection volume/scope** | No target volume defined | 2200 hotels (1200 domestic / 1000 international), per-segment booking-window lead times | [[REQ-001-v1.2]] |
| 7 | **Named per-provider scraping** | Generic "SerpAPI (Google Hotels aggregator)" | Named sources: Gother, Agoda, Trip, Wink (Wink domestic-only) | [[REQ-001-v1.2]] — open question, needs ADR |
| 8 | **Device / login-state / IP dimensions** | Not tracked | Desktop + Mobile Web; Public + Member; Thai IP | [[REQ-001-v1.2]] |
| 9 | **WHO ID on Gother rates** | Not implemented | WHO ID must be displayed for every Gother-sourced rate | [[REQ-001-v1.2]] |
| 10 | **Direct-contract rate comparability** | Not addressed | Wink/HyperGuest direct-contract rates must be included and comparable | [[REQ-001-v1.2]] |
| 11 | **Hotel-list import format** | F-002 expects `hotel_name, city, country, checkin_date,...` | Real hotel list uses `HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH` (see `docs/data/hotel-list-2200.csv`) | [[REQ-001-v1.2]] — open question |
| 12 | **Scraped-data output schema** | `hotel_price_history` has one price + one original price field | Brief's target output has separate tax-exclusive/inclusive price pairs, `task_id`, `Scrapping Round`, structured `Notes` codes (see `docs/data/example-raw-data-schema.md`) | [[REQ-001-v1.2]] — open question, needs ADR |

## Methods Status (competition)
| Method | Status | Notes |
|--------|--------|-------|
| Method 2: SerpAPI + Gother API | ✅ Implemented | Core scraper already built |
| Method 1: ChatGPT + Gother API | ❌ Not implemented | Bonus points — implement after Method 2 is solid |

## Tech Stack
- Backend: Rust + Axum, SQLx (PostgreSQL), Lapin (RabbitMQ), Redis
- Frontend: TypeScript + React 18 + Vite, TanStack Query, Tailwind CSS, shadcn/ui (Radix UI), recharts, react-router-dom
- Database: PostgreSQL (16 migrations: core schema, price_history + partitioning, currency_exchange_rates, scheduled_scrape_configs, materialized views)
- External APIs: SerpAPI (Google Hotels), Gother internal API, OpenAI ChatGPT (Method 1 — bonus), optional Gemini normalization
- Scraper dispatch: pluggable adapter/registry pattern — `ScraperFactory` trait + `default_registry()` (`backend/src/scraper/registry.rs`), stored in `AppState.scraper_registry`
- Deployment: Docker Compose (postgres + redis + rabbitmq + backend + frontend via nginx) — Cloud Run deployment (Part B) suspended indefinitely, functionality-first

## What's already implemented
- [x] PostgreSQL schema — all 16 migrations (core, price history + monthly partitioning, currency rates, scheduled scrape configs, materialized views)
- [x] Rust models, DB repositories, Axum REST API (hotel groups, hotels, scrape jobs, templates, price history, scheduled scrape configs, analytics, hotel directory)
- [x] RabbitMQ publisher + worker processor (parallel hotel scraping)
- [x] SerpAPI + Gother API + ChatGPT scrapers; pluggable adapter/registry; mock fallback for dev
- [x] Redis price caching
- [x] Price normalizer (room type, meal plan, currency → THB) + optional Gemini AI normalization
- [x] Excel import (hotel lists, ADR-003 format) + export (price comparison reports, per-hotel history, per-group history)
- [x] Dual-write: scrape worker writes to `hotel_price_history` — [[REQ-002-v1.1]] F-001
- [x] `scheduled_scrape_configs` + cron worker (`worker/scheduler.rs`) — [[REQ-002-v1.1]] F-003 / F-005
- [x] Price history query + export API — [[REQ-002-v1.1]] F-007 / F-008, F-006
- [x] `hotel_price_history` partition auto-creation (daily idempotent loop, no `pg_partman`) — [[REQ-005-v1.2]] F-002
- [x] Materialized views for analytics (5 views: market position, daily avg price, win rate, booking window, parity violations) — [[REQ-005-v1.2]] F-003 / F-004, refreshed after every scrape job
- [x] Market analytics dashboard — overview card, trend chart, position table, heatmap, date-range filter — [[REQ-003-v1.0]], `frontend/src/pages/AnalyticsDashboard.tsx`
- [x] Global "All Hotels" directory page — country/city filters, search, URL-synced pagination, export — [[REQ-007-hotel-directory]], `frontend/src/pages/HotelsList.tsx`
- [x] Hotel detail page — trend chart + full paginated/filterable raw price-history table — `frontend/src/pages/HotelDetail.tsx`
- [x] React frontend — Dashboard, HotelGroupDetail, ReportView, HotelsList, HotelDetail, AnalyticsDashboard pages; reorganized sidebar (collapsible Hotels section: New Price Search / All Hotels / collapsible Analytics submenu)
- [x] Docker Compose full stack
- [ ] Multi-product support (experiences) — [[REQ-004-v1.0]] — Phase 2, descoped
- [ ] Cloud Run deployment (Part B) — suspended indefinitely per user directive

## Requirements Index
| Doc | Scope | Status |
|-----|-------|--------|
| [[REQ-001-v1.3]] | Hotel price scraping — core module | Active — core scraping/API/Excel/evidence implemented 2026-08-04 |
| [[REQ-002-v1.1]] | Price history & automated scraping | Active — implemented 2026-08-08 |
| [[REQ-003-v1.0]] | Market analytics dashboard | Active — implemented 2026-08-08 |
| [[REQ-004-v1.0]] | Multi-product support (experiences, flights) | Descoped — blocked on REQ-001/002 stability per its own doc |
| [[REQ-005-v1.2]] | Data platform & scalability | Active — implemented 2026-08-08, incl. partition automation (F-002) |
| [[REQ-006-v1.0]] | Price forecasting & predictive analytics (Phase 4) | Descoped — requires 6mo of history data that can't exist by Aug 17 |
| [[REQ-007-hotel-directory]] | Global hotel directory / "All Hotels" page | Active — implemented 2026-08-08 |

## Design Index
| Doc | Scope | Status |
|-----|-------|--------|
| [[data-model]] | Database schema v1.0 (implemented) | Done |
| [[data-model-v1.1]] | Database schema v1.1 (hotel_price_history + currency_exchange_rates + scheduled scraping) | Done |
| [[api-design]] | REST API v1.0 | Done |
| [[system-v1]] | System architecture — components, data flow, scraper strategy, deployment | Done |
| [[class-diagram-v1]] | Domain model — all entities, scrapers, Excel, normalizer, frontend types | Done |
| [[wireframes-v1]] | UI wireframes — all 7 screens, row interaction model, color coding | Done |
| [[ADR-001-scraper-choice]] | Why SerpAPI was chosen | Draft |
| [[ADR-002-price-history-schema]] | Why product-specific history tables (not polymorphic) | Done |

## Design work still needed (before implementation)
- [x] System architecture diagram (components, data flow) — [[system-v1]]
- [x] Class diagram (domain model) — [[class-diagram-v1]]
- [x] UI wireframes (all 7 screens) — [[wireframes-v1]]
- [x] ADR for product-specific price history — [[ADR-002-price-history-schema]]
- [ ] API design v1.1 (new analytics + history endpoints)
- [ ] ADR for: cron approach, analytics query layer

## Important decisions made
- SerpAPI for OTA aggregation (Google Hotels engine) — see [[ADR-001-scraper-choice]]
- RabbitMQ for async job queue
- Redis for short-term price cache (1 hour TTL)
- PostgreSQL as sole data store (no separate data warehouse for current scale)
- Product-specific price history tables (`hotel_price_history`, not a single polymorphic table) — see [[ADR-002-price-history-schema]]
- `currency_exchange_rates` table with `exchange_rate_id` FK on all history tables — auditable, recalculable
- `hotel_price_history` partitioned by month on `scraped_at` (not a flat table)
- All prices in THB as the canonical currency
- `scrape_jobs.method` column persisted in DB so results page always knows which scraper produced the data
- ChatGPT scraper prompts for strict JSON (`ChatGptHotelPriceJson` schema) — no free-text parsing
- Excel import: blank date/rooms/adults cells fall back to job-level `JobDefaults`
- Report table: one row per hotel, cheapest price per OTA column, expandable for all room types + evidence

## Key docs index
- Requirements: docs/requirements/
- Data Model: [[data-model-v1.1]]
- API Design: [[api-design]]
- Decisions: docs/decisions/
- Current Sprint: [[sprint-04]] (demo polish & submission — in progress, started ahead of schedule); [[sprint-03]] substantially complete
- Changelog: [[CHANGELOG]]

> [!NOTE]
> **2026-08-08 (Sprint 04 started early)**: Docker Compose clean-start, env file, README, error-handling review, seed demo data (50-hotel group via API), and a performance check (50-hotel job completed in 9s at `WORKER_CONCURRENCY=3`, all analytics/directory endpoints <50ms) are all done — see [[sprint-04]] for details.
>
> **2026-08-08 (later)**: Dry-run surfaced two backend features with no frontend UI — master-hotel-list import (`/import-master`) and scheduled-scrape-config management. Both now have minimal UI: a format toggle on the group import dialog, and a "Scheduled Scrapes" card on the group detail page (`frontend/src/api/scheduledScrapeConfigs.ts` + `HotelGroupDetail.tsx`). `tsc`/`vite build` clean; both round-tripped against the live API. No browser tool is available in this environment, so the visual walkthrough (report table, badges, evidence panel, analytics charts rendering, no console errors) still needs a human pass before Sprint 04 Task 9 (final submission package) can start.

## How to run the project
```bash
# Start infrastructure
cd gother-price-code && docker-compose up -d postgres redis rabbitmq

# Backend (from gother-price-code/backend/)
cp .env.example .env   # set SERPAPI_KEY, OPENAI_API_KEY, GOTHER_API_KEY, etc.
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
cargo run

# Frontend (from gother-price-code/frontend/)
npm install && npm run dev

# Tests
cd gother-price-code/backend && cargo test
```

## Notes for new contributors

> [!NOTE]
> If `SERPAPI_KEY` is empty, the worker uses `MockScraper` — returns random prices. Safe for dev without real keys.

- Code root: `gother-price-code/` (backend + frontend + docker-compose.yml)
- Backend: port 8080 | Frontend: port 3000 (Vite proxies API)
- Scrape concurrency: `WORKER_CONCURRENCY` env var (default 3)
- Price cache TTL: `PRICE_CACHE_TTL_SECONDS` env var (default 3600s)
- Hotel name normalization strips "hotel"/"resort", lowercases — used for cross-OTA deduplication
