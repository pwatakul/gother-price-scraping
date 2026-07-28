---
title: Sprint 01 — Competition Critical Backend
type: sprint
status: Active
start: 2026-04-28
end: 2026-05-04
tags: [sprint, competition, backend]
related: ["[[REQ-001-v1.1]]", "[[REQ-002-v1.0]]", "[[data-model-v1.1]]"]
---

# Sprint 01 — Competition Critical Backend
_Apr 28 – May 4 (7 days)_

## Sprint Goal
Ship all backend changes required for the competition demo: schema migrations, Excel fix, ChatGPT scraper (Method 1), evidence in API response, and dual-write to `hotel_price_history`.

## Context
Code lives at: `gother-price-code/`
- Backend: `gother-price-code/backend/`
- Frontend: `gother-price-code/frontend/`
- Migrations: `gother-price-code/backend/migrations/` (currently 001–006)

> [!WARNING]
> **Competition deadline: May 15.** Do not gold-plate. Minimum viable implementation for each task.

---

## Planned Tasks

| # | Task | REQ | File(s) | Priority | Status |
|---|------|-----|---------|----------|--------|
| 1 | **Migration 007** — create `product_type` enum | REQ-001 | `migrations/007_add_product_type_enum.sql` | High | Todo |
| 2 | **Migration 008** — add `method` + `product_type` columns to `scrape_jobs` | REQ-001 | `migrations/008_add_method_product_type_to_scrape_jobs.sql` | High | Todo |
| 3 | **Migration 009** — create `currency_exchange_rates` table | REQ-002 | `migrations/009_create_currency_exchange_rates.sql` | High | Todo |
| 4 | **Migration 010** — create partitioned `hotel_price_history` table + indexes | REQ-002 | `migrations/010_create_hotel_price_history.sql` | High | Todo |
| 5 | **Migration 011** — create initial monthly partitions (Apr–Aug 2026) | REQ-002 | `migrations/011_create_hotel_price_history_partitions.sql` | High | Todo |
| 6 | **Migration 012** — create `scheduled_scrape_configs` table | REQ-002 | `migrations/012_create_scheduled_scrape_configs.sql` | High | Todo |
| 7 | **Fix Excel import** — add checkin_date, checkout_date, rooms, adults, currency columns; implement JobDefaults fallback in `ExcelReader` | REQ-001 F-002 | `src/excel/reader.rs` | High | Todo |
| 8 | **Add `method` to scrape_job model + API** — update `ScrapeJob` struct, `CreateScrapeJobRequest`, DB insert, and `ScrapeJobMessage` to carry `method` field | REQ-001 F-004 | `src/models/`, `src/api/`, `src/queue/` | High | Todo |
| 9 | **Add `price_diff_percent` to response** — compute `((gother_price - best_price) / best_price) * 100` in results handler; add to `HotelPriceComparison` struct | REQ-001 F-011 | `src/api/responses.rs`, `src/api/handlers/` | High | Todo |
| 10 | **Add evidence to API response** — expose `source_url` + `scraped_at` inside each `PriceEntry` in the results response (already stored in DB, just not returned) | REQ-001 F-011 | `src/api/responses.rs`, `src/api/handlers/` | High | Todo |
| 11 | **ChatGPT scraper (Method 1)** — implement `ChatGptScraper` using OpenAI API; prompt requests strict `ChatGptHotelPriceJson` schema; merge results with SerpAPI results | REQ-001 F-020 | `src/scraper/chatgpt.rs` (new file) | High | Todo |
| 12 | **Dual-write to `hotel_price_history`** — after each hotel scrape succeeds, insert rows into `hotel_price_history` (lookup/create `currency_exchange_rates` entry first) | REQ-002 F-001 | `src/worker/` | High | Todo |
| 13 | **Migration 013** — create materialized views (`mv_hotel_market_position`, `mv_hotel_daily_avg_price`, `mv_hotel_win_rate`) | REQ-005 | `migrations/013_create_materialized_views.sql` | Medium | Todo |

---

## Definition of Done (per task)
- Code compiles (`cargo build` passes)
- `cargo test` passes (or new test added for new behaviour)
- Manual smoke test: run Docker Compose, hit the endpoint, verify expected output

---

## Blockers / Notes
- **Gother API endpoint + auth** — not yet supplied. Use existing `GotherScraper` as-is. Do not block on this.
- **SerpAPI rate limits** — not confirmed. Keep `WORKER_CONCURRENCY=3` as safe default.
- **OpenAI API key** — needed for Task 11. Set `OPENAI_API_KEY` in `.env`.
- **`OPENAI_API_KEY` missing** → `ChatGptScraper` should fall back to `MockScraper` (same pattern as existing API key guard).
- Migration 013 (materialized views) is Medium priority — skip if time runs short; analytics dashboard can be built after competition.

---

## Carries to Sprint 02
- Frontend: evidence expand panel, ⚠️ badge, price_diff_percent display
- Excel export: add evidence + price_diff_percent columns
- End-to-end demo flow testing

## Retrospective
_Fill at end of sprint._
### What went well
-
### What didn't
-
### Improve next sprint
-
