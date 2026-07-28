---
title: Sprint 02 — Frontend + Integration
type: sprint
status: Planning
start: 2026-05-05
end: 2026-05-11
tags: [sprint, competition, frontend]
related: ["[[REQ-001-v1.1]]", "[[REQ-003-v1.0]]", "[[wireframes-v1]]"]
---

# Sprint 02 — Frontend + Integration
_May 5 – May 11 (7 days)_

## Sprint Goal
Wire all Sprint 01 backend changes into the UI; implement the competition demo screens (report expand/collapse, ⚠️ badge, evidence panel); validate the full scrape → report → export flow end-to-end.

---

## Carried over from Sprint 01
| Task | Reason |
|------|--------|
| Migration 013 (materialized views) | Medium priority — carry if not done in Sprint 01 |

---

## Planned Tasks

| # | Task | REQ | File(s) | Priority | Status |
|---|------|-----|---------|----------|--------|
| 1 | **Report table: expand/collapse rows** — click row to expand; show all room types per source for that hotel | REQ-001 F-011 | `frontend/src/pages/ReportView.tsx` | High | Todo |
| 2 | **Evidence panel** — inside expanded row: table of source / room_type / price / URL / scraped_at per price entry | REQ-001 F-011 | `frontend/src/components/EvidencePanel.tsx` (new) | High | Todo |
| 3 | **⚠️ badge** — show warning icon on price cell when room_type OR meal_plan OR cancellation_policy differs from Gother's entry; tooltip per mismatch type | REQ-001 F-011 | `frontend/src/components/PriceBadge.tsx` (new) | High | Todo |
| 4 | **price_diff_percent display** — show Gap THB and Gap % as separate columns in report table; color-code green (cheapest) / red (losing) | REQ-001 F-011 | `frontend/src/pages/ReportView.tsx` | High | Todo |
| 5 | **Scraping method selector in New Job modal** — radio buttons: SerpAPI / ChatGPT / Both; wire to `method` field in `CreateScrapeJobRequest` | REQ-001 F-004 | `frontend/src/components/NewJobModal.tsx` | High | Todo |
| 6 | **Excel export: add evidence columns** — update `ExcelWriter` to include source_url, scraped_at, price_diff_percent, gap_thb, gap_pct per row | REQ-001 F-012 | `backend/src/excel/writer.rs` | High | Todo |
| 7 | **Excel import: update UI** — update import modal to show all columns in preview (checkin_date, checkout_date, rooms, adults, currency) | REQ-001 F-002 | `frontend/src/components/ImportModal.tsx` | Medium | Todo |
| 8 | **End-to-end demo flow test** — full flow: create group → import Excel → start job (Both methods) → watch progress → view report → export Excel | All | Manual | High | Todo |
| 9 | **Docker Compose full-stack test** — `docker-compose up`, run migrations, run demo flow from fresh state | All | `docker-compose.yml` | High | Todo |
| 10 | **Basic analytics KPI cards** (if time) — market overview: total hotels, Gother win rate %, avg price gap — read from `mv_hotel_market_position` | REQ-003 F-001 | `frontend/src/pages/AnalyticsDashboard.tsx` (new) | Low | Todo |

---

## Key Screens to Match (wireframes-v1)
- **Screen 5: Price Comparison Report** — this is the KEY SCREEN judges will see
  - One row per hotel, cheapest per source column
  - ▶ expand to show all room types + evidence
  - ⚠️ badge on mismatched room type / meal plan / cancellation
  - Gap THB and Gap % columns
  - Green = cheapest / Red = losing
- **Screen 3: New Job Modal** — method selector (SerpAPI / ChatGPT / Both)
- **Screen 4: Job Progress** — already built; verify it still works after backend changes

---

## Definition of Done (per task)
- Feature visible and working in browser at `localhost:3000`
- Golden path tested manually: no console errors, correct data displayed
- TypeScript strict mode: `npm run build` passes with no type errors

---

## Blockers / Notes
- Backend Sprint 01 must be complete before frontend work begins in earnest
- `EvidencePanel` and `PriceBadge` should be new components; avoid bloating `ReportView`
- If ChatGPT scraper is not stable, demo with `method=serpapi` — don't let it block the UI work

---

## Carries to Sprint 03
- Final demo data preparation
- Submission documentation
- Any unfinished low-priority items

## Retrospective
_Fill at end of sprint._
### What went well
-
### What didn't
-
### Improve next sprint
-
