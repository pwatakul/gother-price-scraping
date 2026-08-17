---
title: Sprint 04 — Demo Polish & Submission
type: sprint
status: In Progress
start: 2026-08-15
end: 2026-08-17
tags: [sprint, competition, demo, submission]
related: ["[[REQ-001-v1.3]]", "[[REQ-002-v1.0]]", "[[REQ-003-v1.0]]", "[[wireframes-v1]]"]
---

# Sprint 04 — Demo Polish & Submission
_Aug 15 – Aug 17 (3 days — HARD DEADLINE)_

## Sprint Goal
Competition-ready: full/representative hotel-list coverage proven, stable demo environment, clean Docker Compose startup, submission package prepared.

---

## Planned Tasks

| # | Task | Priority | Status |
|---|------|----------|--------|
| 1 | **Scale validation** — run a scrape job against a representative slice of hotels; confirm dual-write, materialized view refresh, and analytics dashboard hold up at that volume | High | Done — 50-hotel job via API completed in 9s at `WORKER_CONCURRENCY=3` (mock scraper — real SerpAPI/ChatGPT timing not yet measured, no keys configured); analytics/hotels/price-history endpoints all responded in <50ms at that volume |
| 2 | **Seed demo data** — import real Bangkok hotel list, run a full scrape job, verify results feed both the report and the analytics dashboard | High | Done — seeded a "Bangkok Demo Hotels" group (50 hotels), ran a job, confirmed results/price-history/analytics/hotel-directory all reflect it correctly |
| 3 | **Demo script dry-run** — walk the full flow end-to-end: group → import → job → progress → report → analytics dashboard → export; time it; fix anything broken | High | Done at the API level (group create → add hotels → job → poll → results → price-history → analytics → exports, all verified via curl). **Not yet walked in the browser** — UI/console-error check still outstanding, see below |
| 4 | **Docker Compose clean start** — `docker-compose down -v && docker-compose up`, run all migrations from scratch, seed data, verify full flow works on a fresh machine | High | Done — clean `down -v`/`up -d --build`, all 16 migrations applied fresh, all services healthy, `/api/health` and frontend both confirmed reachable |
| 5 | **Environment file** — ensure `.env.example` has all required keys documented: `SERPAPI_KEY`, `OPENAI_API_KEY`, `GOTHER_API_KEY`, `DATABASE_URL`, `REDIS_URL`, `RABBITMQ_URL` | High | Done — already complete, all keys present and documented |
| 6 | **README update** — clear setup instructions for judges: prerequisites, how to run, how to trigger a demo scrape, what to look for in the report and dashboard | High | Done — full rewrite: current feature set, zero-config quick start, demo flow, complete API reference, updated data-flow diagram |
| 7 | **Error handling review** — no panics or unhandled errors in the demo flow; error states show gracefully in UI | High | Done (backend) — no `panic!`/non-test `.expect()`; all non-test `.unwrap()` calls are on static `Response::builder()` bodies (infallible, no user input). **Frontend error-state UI not re-reviewed this pass** |
| 8 | **Performance check** — job progress polling (3s interval) smooth; report table and analytics dashboard load without lag at demo scale | Medium | Done — see Task 1 timings; frontend polling cadence not separately re-verified in-browser this pass |
| 9 | **Final submission package** — repo clean-up, tag the release commit, confirm submission format per competition rules | High | Partial — repo committed as a single snapshot (`e33ccb0`, 115 files, no secrets). **Not yet tagged** — hold until the browser walkthrough confirms the UI looks right |

> [!NOTE]
> **2026-08-08**: Tasks 1, 2, 4, 5, 6, 7 fully done; Tasks 3 and 8 verified at the API/backend level but still need a real browser walkthrough (visual check of report table, ⚠️ badges, evidence panel, analytics charts rendering, no console errors) before calling the demo itself ready. Task 9 blocked on that. Demo seed data left in place in the dev database: hotel group "Bangkok Demo Hotels" (50 hotels + 2 master-import test hotels, 2 completed jobs) — safe to reuse for the browser walkthrough or delete before final submission.
>
> **2026-08-08 (later same day)**: While verifying the frontend↔backend wiring for the dry-run above, found two backend features with **zero frontend UI**: master-hotel-list import (`/import-master` — the real HID/UPDATE-URL/SLUG format the 2200-hotel list actually uses) and scheduled-scrape-config management (`/scheduled-scrape-configs`, REQ-002 F-003/F-004) — both only reachable via direct API calls. This contradicts the demo flow above, which calls for "Import Excel with 20 hotels (ADR-003 format)" — without the master-format toggle, only the plain hotel_name/city/country format was actually importable from the UI. Flagged to the user, who chose to add minimal UI for both rather than leave the gap (overriding this sprint's "no new features" rule for this specific case, since it's closing a demo-blocking gap, not scope creep):
> - Import dialog (`HotelGroupDetail.tsx`) now has a format toggle — Simple vs. Master hotel list — wired to `importHotels`/`importMasterHotels` respectively.
> - New "Scheduled Scrapes" card on the group detail page — list, create (cron expression, lookahead days, method), delete — backed by a new `frontend/src/api/scheduledScrapeConfigs.ts` client.
> - Verified: `tsc`/`vite build` clean, frontend container rebuilt, both features round-tripped against the live API (create/list/delete schedule; master-format `.xlsx` import) with real requests matching what the UI now sends.

---

## Competition Demo Flow (must work perfectly)
```
1. Open app → Dashboard shows hotel groups
2. Create new group "Bangkok City Hotels"
3. Import Excel with 20 hotels (ADR-003 format)
4. Click "New Price Search" → select Both methods → Start Search
5. Watch Job Progress screen: per-hotel status updates every 3s
6. Job completes → View Price Comparison Report
7. Expand a hotel row → see evidence (URL, scraped_at, WHO ID), ⚠️ badge if mismatched
8. Open Analytics Dashboard → market overview, trend chart, position table, heatmap
9. Export Excel → open file, verify all columns present
```

---

## Blockers / Notes
- If Gother API credentials still not supplied: run demo with SerpAPI + ChatGPT only (GotherScraper returns empty)
- If ChatGPT scraper is unstable: demo with `method=serpapi` — focus on the report/analytics UX, not the method count
- If Sprint 01–03 work slipped and full 2200-hotel-scale validation (Task 1) isn't realistic in 3 days, descope to "proven at representative scale (100–300 hotels), architecture supports full scale" — say so explicitly in the demo and README rather than claiming untested scale
- **Do not add new features in this sprint** — polish and stability only

## Post-Competition Backlog (Phase 2+)
After submission, resume with:
- [ ] Rate parity alerts — REQ-003 F-013 (if not reached in Sprint 03)
- [ ] Booking window chart — REQ-003 F-014 (if not reached in Sprint 03)
- [ ] LOS variants scraping — REQ-001 F-004 extended
- [ ] Experience product (Klook) — REQ-004 (descoped for this deadline; its own doc requires REQ-001/002 to be stable first, which is only true after this sprint)
- [ ] Price forecasting — REQ-006 (descoped for this deadline; requires 6 months of accumulated `hotel_price_history`, not possible by Aug 17)

## Retrospective
_Fill at end of sprint._
### What went well
-
### What didn't
-
### Improve for Phase 2
-
