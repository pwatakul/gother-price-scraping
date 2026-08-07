---
title: Sprint 03 — Frontend, Analytics & Data Platform
type: sprint
status: Substantially complete (ahead of schedule)
start: 2026-08-09
end: 2026-08-14
tags: [sprint, competition, frontend, analytics]
related: ["[[REQ-001-v1.3]]", "[[REQ-003-v1.0]]", "[[REQ-005-v1.2]]", "[[wireframes-v1]]"]
---

# Sprint 03 — Frontend, Analytics & Data Platform
_Aug 9 – Aug 14 (6 days)_

> [!NOTE]
> **2026-08-08**: Most of this sprint's scope was pulled forward and completed ahead of its planned start date — see per-task status below. Materialized views landed as migration `016` (not `013` as originally planned; numbering shifted because REQ-001/002 work added migrations 007–012 first). Beyond the originally planned scope, this pass also added: scraper adapter/registry pattern, a global "All Hotels" directory page (REQ-007) with pagination/filtering/export, reorganized sidebar navigation, per-hotel and per-group price-history export, a full raw price-history table on the hotel detail page, and REQ-005 F-002 partition auto-creation. Remaining open items: ⚠️ mismatch badge and evidence panel need a manual demo-flow check (Task 2/3), and the Competitor Heatmap / booking-window chart should be spot-checked visually before Sprint 04.

## Sprint Goal
Wire Sprint 02's backend into the UI (demo screens: report expand/collapse, ⚠️ badge, evidence panel), and build the REQ-003 analytics dashboard on top of the REQ-005 materialized views, backed by real `hotel_price_history` data accumulated since Sprint 02.

---

## Carried over from Sprint 02
| Task | Reason |
|------|--------|
| Migration 013 (materialized views) | Moved here to pair directly with analytics dashboard build |

---

## Planned Tasks

| # | Task | REQ | File(s) | Priority | Status |
|---|------|-----|---------|----------|--------|
| 1 | **Report table: expand/collapse rows** — click row to expand; show all room types per source for that hotel | REQ-001 F-011 | `frontend/src/pages/ReportView.tsx` | High | Done |
| 2 | **Evidence panel** — inside expanded row: table of source / room_type / price / URL / scraped_at / WHO ID per price entry | REQ-001 F-011/F-025 | `frontend/src/components/EvidencePanel.tsx` | High | Done |
| 3 | **⚠️ badge** — show warning icon on price cell when room_type OR meal_plan OR cancellation_policy differs from Gother's entry; tooltip per mismatch type | REQ-001 F-011 | `frontend/src/components/PriceBadge.tsx` | High | Done |
| 4 | **price_diff_percent display** — show Gap THB and Gap % as separate columns in report table; color-code green (cheapest) / red (losing) | REQ-001 F-011 | `frontend/src/pages/ReportView.tsx`, `frontend/src/components/GapPill.tsx` | High | Done |
| 5 | **Scraping method selector in New Job modal** — radio buttons: SerpAPI / ChatGPT / Both; wire to `method` field | REQ-001 F-004 | `frontend/src/components/ScrapeJobForm.tsx` | High | Done |
| 6 | **Excel export: add evidence columns** — update `ExcelWriter` to include source_url, scraped_at, WHO ID, price_diff_percent, gap_thb, gap_pct per row | REQ-001 F-012 | `backend/src/excel/writer.rs` | High | Done |
| 7 | **Excel import: update UI** — import modal shows all columns in preview, supports the ADR-003 hotel-list format | REQ-001 F-002 | `frontend/src/components/ExcelUploader.tsx` | Medium | Done |
| 8 | **Materialized views** — `mv_hotel_market_position`, `mv_hotel_daily_avg_price`, `mv_hotel_win_rate`, `mv_hotel_booking_window`, `mv_hotel_parity_violations` | REQ-005 F-003/F-004 | `migrations/016_create_materialized_views.sql` (landed as 016, not 013 — see sprint note) | High | Done |
| 9 | **Materialized view refresh on schedule completion** — trigger `REFRESH MATERIALIZED VIEW CONCURRENTLY` after each scrape run finishes | REQ-005 F-005 | `src/worker/scheduler.rs` | High | Done — after every job (scheduled or on-demand), stricter than originally planned |
| 10 | **Market Overview Card** — total hotels tracked, % Gother cheapest, avg THB gap | REQ-003 F-001 | `frontend/src/pages/AnalyticsDashboard.tsx` | High | Done |
| 11 | **Price Trend Chart** — line graph: hotel × source × time, filterable by source | REQ-003 F-002 | `frontend/src/pages/AnalyticsDashboard.tsx` (recharts, inline) | High | Done |
| 12 | **Market Position Table** — one row per hotel: Gother price, best OTA price, gap THB/%, win/lose badge | REQ-003 F-003 | `frontend/src/pages/AnalyticsDashboard.tsx` (inline) | High | Done |
| 13 | **Competitor Heatmap** — hotel × OTA grid, color-coded gap | REQ-003 F-004 | `frontend/src/pages/AnalyticsDashboard.tsx` (inline, `HeatmapCell` type) | Medium | Done |
| 14 | **Date range filter** — global 7d/30d/90d/custom filter across dashboard views | REQ-003 F-006 | `frontend/src/pages/AnalyticsDashboard.tsx` | Medium | Done |
| 15 | **Win rate metric** — % of data points where Gother was cheapest, per hotel | REQ-003 F-005 | `mv_hotel_win_rate`, `frontend/src/pages/AnalyticsDashboard.tsx` | Low | Done |

### Added beyond original sprint scope (pulled forward, 2026-08-08)
| Task | REQ | File(s) | Status |
|------|-----|---------|--------|
| Scraper adapter/registry pattern | — | `backend/src/scraper/registry.rs`, `traits.rs` | Done |
| Global "All Hotels" directory — filters, search, pagination, export | REQ-007 | `backend/src/api/handlers/hotel_directory.rs`, `frontend/src/pages/HotelsList.tsx`, `frontend/src/components/Pagination.tsx` | Done |
| Sidebar reorg — collapsible Hotels section, Import/Export tab removed | — | `frontend/src/components/layout/Sidebar.tsx` | Done |
| Per-hotel + per-group price-history export | REQ-002 F-006 | `backend/src/api/handlers/price_history.rs` | Done |
| Full raw price-history table on hotel detail page | REQ-002 | `frontend/src/pages/HotelDetail.tsx` | Done |
| `hotel_price_history` partition auto-creation (daily, idempotent) | REQ-005 F-002 | `backend/src/worker/partition_manager.rs` | Done |

---

## Key Screens to Match (wireframes-v1)
- **Screen 5: Price Comparison Report** — key demo screen: expand/collapse, evidence, ⚠️ badge, Gap THB/%, green/red coding
- **Screen 3: New Job Modal** — method selector (SerpAPI / ChatGPT / Both)
- **New: Analytics Dashboard** — not in original wireframes-v1; build to REQ-003 acceptance criteria directly since no wireframe exists — flag to user if a wireframe pass is wanted before this sprint starts

---

## Definition of Done (per task)
- Feature visible and working in browser at `localhost:3000`
- Golden path tested manually: no console errors, correct data displayed
- TypeScript strict mode: `npm run build` passes with no type errors

---

## Blockers / Notes
- Backend Sprint 02 must be complete before frontend/analytics work begins in earnest — Tasks 10–15 need real rows in `hotel_price_history`, which only exist once Sprint 02's dual-write and scheduler are live. If Sprint 02 slips, seed synthetic history data rather than blocking this entire sprint.
- No wireframe exists yet for the analytics dashboard (unlike the 7 screens in wireframes-v1) — Task 10–14 are built directly from REQ-003's acceptance criteria. Flag this gap if the demo needs a polished, judge-facing analytics screen rather than a functional one.
- If ChatGPT scraper (Sprint 02 Task 5) is not stable, demo with `method=serpapi` — don't let it block UI work.

---

## Carries to Sprint 04
- Final demo data preparation (full or representative hotel-list scale)
- Submission documentation
- Rate parity alerts (REQ-003 F-013) and booking window chart (REQ-003 F-014) if not reached — Low priority, cut first under time pressure

## Retrospective
_Fill at end of sprint._
### What went well
-
### What didn't
-
### Improve next sprint
-
