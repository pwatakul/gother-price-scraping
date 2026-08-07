---
title: Sprint 02 — Core Backend (Scraping + History)
type: sprint
status: Active — REQ-001 tasks (1–10) done 2026-08-04; REQ-002 tasks (11–13) not started
start: 2026-08-02
end: 2026-08-08
tags: [sprint, competition, backend]
related: ["[[REQ-001-v1.3]]", "[[REQ-002-v1.0]]", "[[REQ-005-v1.0]]"]
---

# Sprint 02 — Core Backend (Scraping + History)
_Aug 2 – Aug 8 (7 days)_

## Sprint Goal
Implement everything REQ-001-v1.3 and REQ-002 need in the backend: Excel import fix, ChatGPT scraper (Method 1), evidence + price_diff_percent in API responses, provider-specific scraping per the Sprint 01 ADR, and dual-write to `hotel_price_history`.

## Context
Depends on Sprint 01 being signed off: ADR-003/004/005/006 decided, migrations 007–012 applied, REQ-001-v1.3 published. Do not start Task 7 or 11 below until ADR-005 (provider-specific scraping) is resolved — the scope of both depends directly on that decision.

---

## Carried over from Sprint 01
| Task | Reason |
|------|--------|
| Migration 013 (materialized views) | Medium priority — moved to Sprint 03, pairs naturally with analytics dashboard work |

---

## Planned Tasks

| # | Task | REQ | File(s) | Priority | Status |
|---|------|-----|---------|----------|--------|
| 1 | **Fix Excel import** — add checkin_date, checkout_date, rooms, adults, currency columns; implement JobDefaults fallback; support the HID/SLUG format per ADR-003 | REQ-001 F-002 | `src/excel/reader.rs` | High | Done |
| 2 | **Add `method` to scrape_job model + API** — update `ScrapeJob` struct, `CreateScrapeJobRequest`, DB insert, and `ScrapeJobMessage` to carry `method` field | REQ-001 F-004 | `src/models/`, `src/api/`, `src/queue/` | High | Done |
| 3 | **Add `price_diff_percent` to response** — compute `((gother_price - best_price) / best_price) * 100` in results handler; add to `HotelPriceComparison` struct | REQ-001 F-011 | `src/api/responses.rs`, `src/api/handlers/` | High | Done |
| 4 | **Add evidence to API response** — expose `source_url` + `scraped_at` inside each `PriceEntry` in the results response | REQ-001 F-011 | `src/api/responses.rs`, `src/api/handlers/` | High | Done |
| 5 | **ChatGPT scraper (Method 1)** — implement `ChatGptScraper` using OpenAI API; prompt requests strict `ChatGptHotelPriceJson` schema; merge results with SerpAPI results | REQ-001 F-020 | `src/scraper/chatgpt.rs` (new file) | High | Done |
| 6 | **WHO ID on Gother rates** — thread WHO ID through `GotherScraper` result and into the price entry/response | REQ-001 F-025 | `src/scraper/gother.rs`, `src/models/` | High | Done |
| 7 | **Provider-specific scraping (F-022)** — implement per ADR-005's decision: either named-provider extraction from SerpAPI results, or dedicated Agoda/Trip/Wink scrapers | REQ-001 F-022 | `src/scraper/` | High | Done |
| 8 | **Direct-contract rate handling (Wink/HyperGuest)** — include and flag direct-contract rates as comparable in the results set | REQ-001 F-026 | `src/scraper/`, `src/api/responses.rs` | High | Done |
| 9 | **Blank-not-zero rendering** — ensure missing rate/provider combos serialize as `null`/absent, not `0` | REQ-001 F-027 | `src/api/responses.rs` | Medium | Done |
| 10 | **Device + login-state dimensions** — add `device` (Desktop/Mobile Web) and `login_state` (Public/Member) to scrape job config and carry through to results | REQ-001 F-023/F-024 | `src/models/`, `src/scraper/` | Medium | Done |
| 11 | **Dual-write to `hotel_price_history`** — after each hotel scrape succeeds, insert rows into `hotel_price_history` (lookup/create `currency_exchange_rates` entry first), mapped per ADR-004's output schema decision | REQ-002 F-001 | `src/worker/` | High | Todo |
| 12 | **Scheduled scrape config CRUD + cron worker** — API for `scheduled_scrape_configs`, plus the scheduler implementation chosen in ADR-006, targeting weekly cadence | REQ-002 F-003/F-005, REQ-001 F-028 | `src/api/`, `src/worker/scheduler.rs` (new) | High | Todo |
| 13 | **Price history query API** — endpoints to read back `hotel_price_history` filtered by hotel/source/date range, for Sprint 03's analytics dashboard to consume | REQ-002 F-007/F-008 | `src/api/handlers/history.rs` (new) | High | Todo |

---

## Definition of Done (per task)
- Code compiles (`cargo build` passes)
- `cargo test` passes (or new test added for new behaviour)
- Manual smoke test: run Docker Compose, hit the endpoint, verify expected output

---

## Blockers / Notes
- **Gother API endpoint + auth** — not yet supplied. Use existing `GotherScraper` as-is. Do not block on this.
- **SerpAPI rate limits** — not confirmed. Keep `WORKER_CONCURRENCY=3` as safe default.
- **OpenAI API key** — needed for Task 5. Set `OPENAI_API_KEY` in `.env`; `ChatGptScraper` falls back to `MockScraper` if missing (same pattern as existing API key guard).
- Task 7 (provider-specific scraping) is the highest-risk item in this sprint — if ADR-005 concluded dedicated per-OTA scrapers are required, this task alone could consume most of the sprint. Reassess scope on day 1 of this sprint against the ADR-005 outcome and re-cut the remaining tasks if needed.
- 2200-hotel / weekly-cadence full coverage (F-021, F-028) is a production target, not a demo requirement — a working pipeline proven on a smaller seed set is sufficient for Sprint 02; full-scale coverage is validated in Sprint 04.

---

## Carries to Sprint 03
- Frontend: evidence expand panel, ⚠️ badge, price_diff_percent display, method selector
- Excel export: add evidence + price_diff_percent columns
- Analytics dashboard (REQ-003) + materialized views (REQ-005 Migration 013)

## Retrospective
_Fill at end of sprint._
### What went well
-
### What didn't
-
### Improve next sprint
-
