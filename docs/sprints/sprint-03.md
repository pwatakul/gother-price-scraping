---
title: Sprint 03 — Demo Polish & Submission
type: sprint
status: Planning
start: 2026-05-12
end: 2026-05-15
tags: [sprint, competition, demo, submission]
related: ["[[REQ-001-v1.1]]", "[[wireframes-v1]]"]
---

# Sprint 03 — Demo Polish & Submission
_May 12 – May 15 (4 days — HARD DEADLINE)_

## Sprint Goal
Competition-ready: stable demo environment, realistic seed data, clean Docker Compose startup, submission package prepared.

---

## Planned Tasks

| # | Task | Priority | Status |
|---|------|----------|--------|
| 1 | **Seed demo data** — import real Bangkok hotel list via Excel (20–30 hotels), run a full scrape job with both methods, verify results | High | Todo |
| 2 | **Demo script dry-run** — walk through the full competition demo flow end-to-end: group → import → job → progress → report → export; time it; fix anything broken | High | Todo |
| 3 | **Docker Compose clean start** — `docker-compose down -v && docker-compose up`, run all migrations from scratch, seed data, verify full flow works on a fresh machine | High | Todo |
| 4 | **Environment file** — ensure `.env.example` has all required keys documented: `SERPAPI_KEY`, `OPENAI_API_KEY`, `GOTHER_API_KEY`, `DATABASE_URL`, `REDIS_URL`, `RABBITMQ_URL` | High | Todo |
| 5 | **README update** — clear setup instructions for judges: prerequisites, how to run, how to trigger a demo scrape, what to look for in the report | High | Todo |
| 6 | **Error handling review** — make sure no panics or unhandled errors appear in the demo flow; verify error states show gracefully in UI | High | Todo |
| 7 | **Performance check** — ensure job progress polling (3s interval) works smoothly; report table loads without lag for 20+ hotels | Medium | Todo |
| 8 | **Final submission package** — zip / repo clean-up, tag the release commit, confirm submission format per competition rules | High | Todo |

---

## Competition Demo Flow (must work perfectly)
```
1. Open app → Dashboard shows hotel groups
2. Create new group "Bangkok City Hotels"
3. Import Excel with 20 hotels (hotel_name, city, country, checkin_date, checkout_date, rooms, adults)
4. Click "New Price Search" → select Both methods → Start Search
5. Watch Job Progress screen: per-hotel status updates every 3s
6. Job completes → View Price Comparison Report
7. Expand a hotel row → see evidence (URL, scraped_at), ⚠️ badge if mismatched
8. Export Excel → open file, verify all columns present
```

---

## Blockers / Notes
- If Gother API credentials still not supplied: run demo with SerpAPI + ChatGPT only (GotherScraper returns empty)
- If ChatGPT scraper is unstable: demo with `method=serpapi` — focus on the report UX not the method count
- **Do not add new features in this sprint** — polish and stability only

## Post-Competition Backlog (Phase 2+)
After submission, resume with:
- [ ] Scheduled scraping (cron worker) — REQ-002
- [ ] Analytics dashboard (trend charts, heatmap, win rate) — REQ-003
- [ ] Rate parity alerts — REQ-003 F-013
- [ ] Booking window chart — REQ-003 F-014
- [ ] LOS variants scraping — REQ-001 F-004 extended
- [ ] Experience product (Klook) — REQ-004
- [ ] Price forecasting — REQ-006

## Retrospective
_Fill at end of sprint._
### What went well
-
### What didn't
-
### Improve for Phase 2
-
