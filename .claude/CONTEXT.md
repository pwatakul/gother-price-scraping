---
title: Project Context
type: context
updated: 2026-08-17
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
- Version: **v1.0 — submitted** (Gother Challenge 2026, 17 Aug 2026)
- **Live: https://34-124-161-138.nip.io** — GCP, HTTPS, authenticated. See [[REQ-010-production-deployment]]
- Submission document: [[SUBMISSION-v1.0]] — read this first for a full picture of what exists
- Phase: Implementation complete for Phase 1 (hotels). Deployed and verified end to end
- 26 migrations · 48 API endpoints · 8 screens · 78 backend tests passing

### Delivered since the 2026-08-08 note below
| Change | Reference |
|---|---|
| ChatGPT scraper **removed**; AI demoted to a marked fallback with per-row `via_method` provenance | [[ADR-007-remove-chatgpt-scraper]], [[ADR-011-serpapi-primary-gemini-fallback]], [[REQ-001-v1.5]] |
| Mock-scraper silent fallback **removed** — a missing key now fails loudly | [[ADR-008-no-silent-mock-fallback]] |
| Device and member/login-state dimensions **dropped** (measured: zero price difference over 69 sources) | [[ADR-010-drop-device-and-member-dimensions]], [[REQ-008-v1.1]] |
| Provider allowlist widened to 10, exact matching | [[ADR-009-widen-provider-allowlist]] |
| Analytics rebased so comparisons are within one (hotel, check-in date) | [[ADR-013-booking-window-comparison-basis]] |
| Per-group saved search config + per-group analytics | [[ADR-012-group-search-config]], [[REQ-003-v1.2]] |
| Login / sessions / role field | [[ADR-014-cookie-session-authentication]], [[REQ-009-v1.0]] |
| **GCP deployment — no longer suspended.** Single VM, Docker, auto-HTTPS | [[ADR-015-gcp-single-vm-deployment]], [[REQ-010-production-deployment]] |

## Gaps vs Competition Brief (must fix before demo)

| # | Gap | Current State | Required by Brief | REQ |
|---|-----|--------------|-------------------|-----|
| 1 | **Excel per-row search params** | checkin/checkout/rooms/adults set at job level (same for all hotels) | Each Excel ROW can have its own checkin_date, checkout_date, rooms, adults | [[REQ-001-v1.2]] |
| 2 | ~~**Method 1: ChatGPT scraper**~~ | **Closed — deliberately not shipped** | Bonus points offered, but AI-generated prices were measured as fabricated (3 OTAs at an identical ฿6,551 vs a true ฿6,773). AI retained only as a badged fallback | [[ADR-007-remove-chatgpt-scraper]] |
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
| Method 1: ChatGPT + Gother API | ⛔ Removed by decision | Not a gap — measured as fabricating prices. See [[ADR-007-remove-chatgpt-scraper]] and §6 of [[SUBMISSION-v1.0]] |

## Tech Stack
- Backend: Rust + Axum, SQLx (PostgreSQL), Lapin (RabbitMQ), Redis
- Frontend: TypeScript + React 18 + Vite, TanStack Query, Tailwind CSS, shadcn/ui (Radix UI), recharts, react-router-dom
- Database: PostgreSQL 16 (**26 migrations**: core schema, price_history + partitioning, currency rates, scheduled configs, materialized views, group search config, users)
- External APIs: SerpAPI (Google Hotels — authoritative), Gemini (marked fallback + room-type normalization), Gother internal API (awaiting credentials). **ChatGPT removed** — [[ADR-007-remove-chatgpt-scraper]]
- Scraper dispatch: pluggable adapter/registry pattern — `ScraperFactory` trait + `default_registry()` (`backend/src/scraper/registry.rs`), stored in `AppState.scraper_registry`
- Deployment: Docker Compose locally; **live on GCP** (single e2-small VM, Caddy auto-HTTPS, Cloud Build → Artifact Registry) — [[ADR-015-gcp-single-vm-deployment]]

## What's already implemented
- [x] PostgreSQL schema — all 16 migrations (core, price history + monthly partitioning, currency rates, scheduled scrape configs, materialized views)
- [x] Rust models, DB repositories, Axum REST API (hotel groups, hotels, scrape jobs, templates, price history, scheduled scrape configs, analytics, hotel directory)
- [x] RabbitMQ publisher + worker processor (parallel hotel scraping)
- [x] SerpAPI (authoritative) + Gemini (badged fallback) + Gother API (awaiting credentials) scrapers; pluggable adapter/registry. Mock is **opt-in only** (`ENABLE_MOCK_SCRAPER`)
- [x] Redis price caching
- [x] Price normalizer (room type, meal plan, currency → THB) + optional Gemini AI normalization
- [x] Excel import (hotel lists, ADR-003 format) + export (price comparison reports, per-hotel history, per-group history)
- [x] Dual-write: scrape worker writes to `hotel_price_history` — [[REQ-002-v1.1]] F-001
- [x] `scheduled_scrape_configs` + cron worker (`worker/scheduler.rs`) — [[REQ-002-v1.1]] F-003 / F-005
- [x] Price history query + export API — [[REQ-002-v1.1]] F-007 / F-008, F-006
- [x] `hotel_price_history` partition auto-creation (daily idempotent loop, no `pg_partman`) — [[REQ-005-v1.2]] F-002
- [x] Materialized views for analytics (6 refreshed: market position, price-by-stay, daily avg price, win rate, booking window, parity violations) — [[REQ-005-v1.2]] F-003 / F-004, refreshed after every scrape job
- [x] Market analytics dashboard — overview card, trend chart, position table, heatmap, date-range filter — [[REQ-003-v1.0]], `frontend/src/pages/AnalyticsDashboard.tsx`
- [x] Global "All Hotels" directory page — country/city filters, search, URL-synced pagination, export — [[REQ-007-hotel-directory]], `frontend/src/pages/HotelsList.tsx`
- [x] Hotel detail page — trend chart + full paginated/filterable raw price-history table — `frontend/src/pages/HotelDetail.tsx`
- [x] React frontend — Dashboard, HotelGroupDetail, ReportView, HotelsList, HotelDetail, AnalyticsDashboard pages; reorganized sidebar (collapsible Hotels section: New Price Search / All Hotels / collapsible Analytics submenu)
- [x] Docker Compose full stack
- [ ] Multi-product support (experiences) — [[REQ-004-v1.0]] — Phase 2, descoped
- [x] GCP deployment — single VM + Docker + auto-HTTPS, live — [[ADR-015-gcp-single-vm-deployment]]

## Requirements Index
| Doc | Scope | Status |
|-----|-------|--------|
| [[REQ-001-v1.5]] | Hotel price scraping — core module | Active — current version |
| [[REQ-002-v1.3]] | Price history & automated scraping | Active — current version |
| [[REQ-003-v1.2]] | Market analytics dashboard | Active — current version, incl. per-group analytics |
| [[REQ-004-v1.0]] | Multi-product support (experiences, flights) | Descoped — blocked on REQ-001/002 stability per its own doc |
| [[REQ-005-v1.2]] | Data platform & scalability | Active — implemented 2026-08-08, incl. partition automation (F-002) |
| [[REQ-006-v1.0]] | Price forecasting & predictive analytics (Phase 4) | Descoped — requires 6mo of history data that can't exist by Aug 17 |
| [[REQ-007-hotel-directory]] | Global hotel directory / "All Hotels" page | Active |
| [[REQ-008-v1.1]] | Standardized booking-window tracking | Active |
| [[REQ-009-v1.0]] | Login authentication + role field | Active — roles stored, not enforced |
| [[REQ-010-production-deployment]] | Production deployment (GCP) | Active — live |

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
cd gother-price-code
cp .env.example .env
#   SERPAPI_KEY  — required, the live price source (no mock fallback)
#   JWT_SECRET   — required, openssl rand -base64 48 (backend refuses to boot without it)
#   ADMIN_PASSWORD — optional; unset seeds admin/admin1234! and logs a warning
docker compose up -d          # migrations run automatically at startup
# → frontend http://localhost:3000   ·   backend http://localhost:8080

cd backend && cargo test      # 78 tests
```

Deploying to GCP: `./deploy.sh` from `gother-price-code/` — see [[REQ-010-production-deployment]].

## Notes for new contributors

> [!WARNING]
> **A missing `SERPAPI_KEY` now FAILS the scrape — it does not fall back to mock data.**
> The old silent fallback reported 52/52 successes over 315 fabricated prices that were
> indistinguishable from real ones. It was removed in [[ADR-008-no-silent-mock-fallback]].
> The mock scraper still exists but is opt-in only, via `ENABLE_MOCK_SCRAPER=true`, and
> logs a warning on every startup when enabled.

- Code root: `gother-price-code/` (backend + frontend + docker-compose.yml)
- Backend: port 8080 | Frontend: port 3000 (Vite proxies API)
- Scrape concurrency: `WORKER_CONCURRENCY` env var (default 3)
- Price cache TTL: `PRICE_CACHE_TTL_SECONDS` env var (default 3600s)
- Hotel name normalization strips "hotel"/"resort", lowercases — used for cross-OTA deduplication
