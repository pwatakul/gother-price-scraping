---
title: Sprint 01 — Design Decisions & Migrations
type: sprint
status: Active
start: 2026-07-30
end: 2026-08-01
tags: [sprint, competition, design, backend]
related: ["[[REQ-001-v1.2]]", "[[REQ-002-v1.0]]", "[[REQ-005-v1.0]]", "[[ADR-001-scraper-choice]]", "[[ADR-002-price-history-schema]]"]
---

# Sprint 01 — Design Decisions & Migrations
_Jul 30 – Aug 1 (3 days)_

## Sprint Goal
Close the 3 open design questions blocking [[REQ-001-v1.2]] before any related schema or scraper code is written, then land the schema migrations that depend on those decisions. This sprint exists because CONTEXT.md's implementation gate ("no implementation until design sign-off") is still active and the CEO brief data (`docs/raw/Req price scrapping - 17 July 26.xlsx`, parsed 2026-07-27) introduced real gaps that were never resolved.

## Context
Code lives at: `gother-price-code/`
- Backend: `gother-price-code/backend/`
- Migrations: `gother-price-code/backend/migrations/` (currently 001–006 only — verified 2026-07-30)

> [!WARNING]
> **Deadline: Aug 17, 2026.** This sprint is 3 days — do not let ADR discussion sprawl. Each open question needs a decision, not a perfect answer.

---

## Planned Tasks

| # | Task | REQ | Output | Priority | Status |
|---|------|-----|--------|----------|--------|
| 1 | **Decide hotel-list import format** — brief's real list (`HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH`) vs. F-002 spec (`hotel_name, city, country, checkin_date,...`). Decide: extend F-002 to accept both, or redesign import around HID/SLUG as primary key. | REQ-001 F-002 | `docs/decisions/ADR-003-hotel-list-import-format.md` | High | Todo |
| 2 | **Decide output schema handling** — brief's example output (`task_id`, `Scrapping Round`, split tax-exclusive/inclusive price pairs, structured `Notes` codes) vs. current `hotel_price_history`. Decide: add columns, or export-time transform. | REQ-001/002 | `docs/decisions/ADR-004-output-schema-mapping.md` | High | Todo |
| 3 | **Decide provider-specific scraping approach (F-022)** — can SerpAPI's Google Hotels aggregation reliably attribute results to Agoda/Trip/Wink specifically, or are dedicated per-OTA scrapers required? Time-box investigation to half a day; decide based on what's actually testable, not theoretical. | REQ-001 F-022 | `docs/decisions/ADR-005-provider-specific-scraping.md` | High | Todo |
| 4 | **Decide cron/scheduling approach** — in-process scheduler (tokio-cron-scheduler) vs. external cron hitting an API trigger, for weekly `scheduled_scrape_configs` runs. Flagged as needed in CONTEXT.md's "Design work still needed." | REQ-002 F-003/F-005 | `docs/decisions/ADR-006-cron-approach.md` | Medium | Todo |
| 5 | **Migration 007** — create `product_type` enum | REQ-001 | `migrations/007_add_product_type_enum.sql` | High | Todo |
| 6 | **Migration 008** — add `method` + `product_type` columns to `scrape_jobs` | REQ-001 | `migrations/008_add_method_product_type_to_scrape_jobs.sql` | High | Todo |
| 7 | **Migration 009** — create `currency_exchange_rates` table | REQ-002 | `migrations/009_create_currency_exchange_rates.sql` | High | Todo |
| 8 | **Migration 010** — create partitioned `hotel_price_history` table + indexes, incorporating the ADR-004 output-schema decision (task_id, Scrapping Round, tax-exclusive/inclusive pairs if in-schema) | REQ-002/005 | `migrations/010_create_hotel_price_history.sql` | High | Todo |
| 9 | **Migration 011** — create initial monthly partitions (Aug–Dec 2026) | REQ-002/005 | `migrations/011_create_hotel_price_history_partitions.sql` | High | Todo |
| 10 | **Migration 012** — create `scheduled_scrape_configs` table, shaped per ADR-006 | REQ-002 | `migrations/012_create_scheduled_scrape_configs.sql` | High | Todo |
| 11 | **Update REQ-001 to v1.3** — close the 3 open questions in [[REQ-001-v1.2]], record decisions in the Change Log, do not overwrite v1.2 | REQ-001 | `docs/requirements/REQ-001-v1.3.md` | High | Todo |

---

## Definition of Done (per task)
- ADRs: decision recorded, alternatives considered, consequence noted (use `docs/decisions/ADR-000-template.md`)
- Migrations: `sqlx migrate run` succeeds against a clean database; `cargo build` passes
- REQ-001-v1.3 created (not overwriting v1.2), Open Questions section shows all 3 closed

---

## Blockers / Notes
- Task 3 (provider attribution) is the highest-risk open question — if SerpAPI can't reliably attribute to named providers, Sprint 02's F-022 work balloons into building per-OTA scrapers from scratch. Surface this risk immediately if discovered, don't wait for sprint end.
- Do not start Sprint 02 backend work until Tasks 1–4 (ADRs) are signed off — this is the explicit design gate from CONTEXT.md.

---

## Carries to Sprint 02
- All scraper/API implementation work (nothing implemented this sprint beyond migrations)

## Retrospective
_Fill at end of sprint._
### What went well
-
### What didn't
-
### Improve next sprint
-
