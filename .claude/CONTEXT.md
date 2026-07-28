---
title: Project Context
type: context
updated: 2026-04-27
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
- Version: v0.1
- Active Sprint: [[sprint-01]]
- Branch: main
- Phase: Requirements & Design (CEO brief fully received — gaps identified below)
- **Submission Deadline: May 15, 2026** (19 days from now)
- Prize target: ฿120,000 (1st place)

> [!WARNING]
> **No implementation until design is signed off.** CEO brief is complete. Critical gaps identified vs. current implementation — see "Gaps vs Competition Brief" section below. Requirements update and system design needed before coding resumes.

> [!NOTE]
> **2026-07-27**: Raw brief data (`docs/raw/Req price scrapping - 17 July 26.xlsx`) parsed into [[REQ-001-v1.2]]. Managed data assets: `docs/data/hotel-list-2200.csv` (the 2200-hotel list) and `docs/data/example-raw-data-schema.md` + `docs/data/example-raw-data-sample.csv` (target scraper output schema). Three open questions from this brief (hotel-list import format, output schema gap, provider-specific scraping) still need design decisions — see REQ-001-v1.2 Open Questions.

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
- Frontend: TypeScript + React 18 + Vite, TanStack Query, Tailwind CSS, shadcn/ui (Radix UI)
- Database: PostgreSQL (v1.0: 6 migrations; v1.1: adds price_history + scheduled_scrape_configs)
- External APIs: SerpAPI (Google Hotels), Gother internal API, OpenAI ChatGPT (Method 1 — bonus)
- Deployment: Docker Compose (postgres + redis + rabbitmq + backend + frontend via nginx)

## What's already implemented (v0.1 — hotel scraping core)
- [x] PostgreSQL schema v1.0 — all 6 migrations
- [x] Rust models, DB repositories, Axum REST API (hotel groups, hotels, scrape jobs, templates)
- [x] RabbitMQ publisher + worker processor (parallel hotel scraping)
- [x] SerpAPI + Gother API scrapers; mock fallback for dev
- [x] Redis price caching
- [x] Price normalizer (room type, meal plan, currency → THB)
- [x] Excel import (hotel lists) + export (price comparison reports)
- [x] Optional Gemini AI normalization
- [x] React frontend — Dashboard, HotelGroupDetail, ReportView pages + all components
- [x] Docker Compose full stack

## What's pending (requires CEO brief + design sign-off first)
- [ ] `price_history` table (partitioned, time-series) — [[REQ-001-v1.1]] F-015
- [ ] Dual-write: scrape worker writes to price_history — [[REQ-002-v1.0]] F-001
- [ ] `scheduled_scrape_configs` + cron worker — [[REQ-002-v1.0]] F-003 / F-005
- [ ] Price history query API — [[REQ-002-v1.0]] F-007 / F-008
- [ ] Market analytics dashboard (trend charts, position table, heatmap) — [[REQ-003-v1.0]]
- [ ] Materialized views for analytics — [[REQ-005-v1.0]] F-003 / F-004
- [ ] Multi-product support (experiences) — [[REQ-004-v1.0]] — Phase 2

## Requirements Index
| Doc | Scope | Status |
|-----|-------|--------|
| [[REQ-001-v1.2]] | Hotel price scraping — core module | Active |
| [[REQ-002-v1.0]] | Price history & automated scraping | Draft |
| [[REQ-003-v1.0]] | Market analytics dashboard | Draft |
| [[REQ-004-v1.0]] | Multi-product support (experiences, flights) | Planned |
| [[REQ-005-v1.0]] | Data platform & scalability | Draft |
| [[REQ-006-v1.0]] | Price forecasting & predictive analytics (Phase 4) | Draft — Planned |

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
- Current Sprint: [[sprint-01]]
- Changelog: [[CHANGELOG]]

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
