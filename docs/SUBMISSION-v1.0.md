---
title: "Gother Market Intelligence Platform — Challenge Submission"
type: submission
version: "1.0"
date: 2026-08-17
status: Submitted
tags: [submission, competition, overview, architecture]
event: "Gother Challenge 2026"
live_url: "https://34-124-161-138.nip.io"
related: ["[[REQ-001-v1.5]]", "[[REQ-002-v1.3]]", "[[REQ-003-v1.2]]", "[[REQ-008-v1.1]]", "[[REQ-009-v1.0]]", "[[REQ-010-production-deployment]]", "[[ADR-015-gcp-single-vm-deployment]]"]
---

# Gother Market Intelligence Platform
### Gother Challenge 2026 — Submission Document
_Version 1.0 · 17 August 2026_

---

## 1. Executive summary

**What it is.** A market intelligence platform that tracks what competing OTAs charge for the same hotel room, stores that history permanently, and turns it into answers a commercial team can act on: who is cheapest, by how much, on which hotels, and how that is moving.

**Who it is for.** Gother's Product Owner and Business Development team — people who need to know "are we priced competitively on the Bangkok portfolio this week?" without exporting anything into a spreadsheet first.

**Try it now.**

| | |
|---|---|
| **URL** | **https://34-124-161-138.nip.io** |
| **Username** | `admin` |
| **Password** | supplied separately with this submission |

Running live on Google Cloud with HTTPS, authentication, and real scraped data — not a local demo.

### Five things worth noticing

1. **Every comparison is genuinely like-for-like.** Prices are only ever compared within the same hotel *and* the same check-in date. This is not cosmetic: before the fix, the platform was comparing one provider's 30-days-ahead quote against another's 35-days-ahead quote and reporting the difference as a competitive gap. Correcting it moved reported median premiums from 13–19% down to 2–5% — the old numbers overstated the gaps three- to five-fold.

2. **No fabricated data, anywhere.** An earlier build silently substituted a mock scraper when the API key was missing and reported 52/52 successes over 315 invented prices. That path was removed outright. AI-derived estimates are now a separate, visibly-badged tier, and every stored price records which scraper produced it.

3. **Collection is standardised so the data is comparable across time.** Every scheduled run produces the same fixed grid — +1/+3/+7/+14/+30 days ahead, one night, one room, two adults — so a hotel scraped today can be compared with the same hotel scraped last month without normalisation.

4. **We removed three things the brief asked for, and can show why.** Including one worth bonus points. Each removal is backed by a measurement, and each is documented (§6).

5. **The engineering is documented as it was decided.** 15 Architecture Decision Records and 23 versioned requirement files record not just what was built but what was rejected and on what evidence.

**At a glance:** 26 database migrations · 48 API endpoints · 8 application screens · 78 backend tests passing · 10-provider allowlist · 6 materialized analytics views · deployed and live.

---

## 2. What it does

### 2.1 Hotels and groups

| Feature | What it is for |
|---|---|
| **Hotel groups** | Organise the portfolio into comparable sets ("Bangkok City Hotels", "Hua Hin") so analytics can be read per market rather than across the whole estate |
| **Excel / CSV import** | Load a hotel list without typing it. Two formats are accepted: a simple `hotel_name, city, country` sheet, and the **real 2200-row master format** (`HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH`) that the actual hotel list uses |
| **Per-row search parameters** | Each imported row may carry its own check-in, check-out, rooms and adults, falling back to job defaults — so one import can express a whole schedule of different stays |
| **Global hotel directory** | One searchable list of every hotel in the system, filterable by country and city, with pagination and export — for finding a hotel without knowing which group it is in |
| **Hotel editing** | Correct a name, city or country after import, with duplicate detection on the normalised name |

### 2.2 Price collection

| Feature | What it is for |
|---|---|
| **On-demand price search** | Run a comparison now, for a chosen group and stay dates |
| **Saved per-group search config** | Store how a group should be searched — method, booking windows, rooms, adults — then re-run it with one click instead of re-entering it |
| **Scheduled collection (cron)** | Automatic recurring runs. Every fire expands to the standard grid: **+1, +3, +7, +14, +30 days ahead**, 1 night, 1 room, 2 adults — identical for every hotel, so the resulting series are directly comparable |
| **Manual "Run now"** | Fire the standard grid immediately without disturbing the cron cadence |
| **Two-tier scraping** | SerpAPI (Google Hotels) is authoritative. Gemini runs **only** where scraping returned nothing, and never overwrites a real price |
| **Per-row provenance** | Every stored price records the scraper that produced it (`via_method`), and AI-derived rows are badged distinctly in the UI |
| **Provider normalisation** | Ten recognised providers: Gother, Agoda, Trip.com, Wink, Booking.com, Expedia, Priceline, Traveloka, Klook, Hotel Direct. Matching is exact, not substring — an earlier substring match filed EaseMyTrip.com and Clicktrip.com as Trip.com |
| **Result caching** | Redis caches scrape results with a TTL so re-running a search does not re-spend API quota |
| **Job progress tracking** | Per-hotel status while a job runs, with failures attributed to the specific hotel and source |

### 2.3 Comparison and evidence

| Feature | What it is for |
|---|---|
| **Price comparison report** | Per hotel, every provider's price side by side for the same stay |
| **Evidence on every row** | The source URL and the exact `scraped_at` timestamp — so any number in the report can be checked against its origin |
| **Blank, never zero** | A provider that returned no price renders as `—`. A missing price is not a price of zero, and the distinction matters when a minimum is taken |
| **Booking-window coverage** | Per hotel, the **cheapest** quote for each of the five standard windows, showing how many providers that minimum was drawn from — a "lowest of one" is flagged, because it is not a comparison |

### 2.4 Analytics

Available both **globally** and **per hotel group** (reached from the group page), because "how are we doing" is usually a question about one market, not the whole estate.

| Feature | What it answers |
|---|---|
| **Provider Benchmark** | How often each provider is cheapest, and its median premium over the cheapest. Stays quoted by only one provider are excluded, and the number of quotes compared is shown so thin coverage is visible next to a high win rate |
| **Market Position** | One row per hotel: Gother's price against the market's cheapest, average and dearest, for that hotel's most recent stay |
| **Competitor Heatmap** | Hotel × provider grid with the cheapest cell highlighted. The Gother column is always present — see §6 |
| **Price trend chart** | Price over time per provider, filtered to a single booking window so the lines are comparable |
| **Booking-window analysis** | How price varies with how far ahead the stay is booked |
| **Parity violations** | Where the same room is priced inconsistently across channels |
| **Search, sort, paginate, export** | On every analytics table, with the state held in the URL so a filtered view can be shared or reloaded. Export writes exactly what is on screen |

### 2.5 Data platform

| Feature | What it is for |
|---|---|
| **Permanent price history** | Every observation is kept. This is the asset — it cannot be re-created, because you cannot scrape the past |
| **Monthly partitioning** | `hotel_price_history` is partitioned by month, with a background task keeping a rolling four-month window of partitions created automatically |
| **6 materialized views** | Analytics read pre-aggregated views refreshed after each job, so dashboards stay fast as history grows |
| **Currency normalisation** | Prices normalised to THB via stored exchange rates |
| **Room-type / meal-plan normalisation** | So "Deluxe King, room only" from two sites can be recognised as the same product |
| **Exports** | Excel for the comparison report; CSV for price history, the hotel directory, and every analytics table |

### 2.6 Access

| Feature | Status |
|---|---|
| **Login** | Argon2id password hashing, signed session token in an httpOnly `SameSite=Strict` cookie, 12-hour sessions |
| **Session restore** | Reloading the page keeps you signed in |
| **Change password** | Self-service, and the seeded default is flagged in the UI until it is changed |
| **Roles** | `admin` / `viewer` stored, carried through the session and displayed — **but not yet enforced.** A viewer can currently do everything an admin can |

### 2.7 Built but not yet producing data

Stated here rather than buried, because a judge will find them:

| Item | Status |
|---|---|
| **Gother price column** | Scraper written and registered; waits on `GOTHER_API_URL` / `GOTHER_API_KEY`. Gother does not appear in Google Hotels, so SerpAPI cannot supply it (§6) |
| **Wink** | In the allowlist, no data source available |
| **WHO ID on Gother rates** | Field wired end to end; unverifiable until Gother's API is connected |
| **Direct-contract flag** | Recognises Wink/HyperGuest; currently a no-op as no scraper emits those rows |
| **Role enforcement** | Roles stored and shown, no endpoint restricted by them |

---

## 3. Getting started

### 3.1 Use the live system (fastest)

Go to **https://34-124-161-138.nip.io** and sign in as `admin`.

1. **Dashboard** — two hotel groups are seeded: *Thailand Demo Hotels* (20 real, verified Thai hotels) and *Hua Hin*.
2. **Open a group** → the hotel list, with each hotel's latest price and source.
3. **Click a hotel** → its detail page: price trend chart, booking-window coverage showing the cheapest quote per window, and the full raw price history, filterable by provider.
4. **Back to the group → Analytics** → Provider Benchmark, Market Position and Competitor Heatmap for that group alone. Try the search box and column sorting; note the URL updates so the view can be shared.
5. **Export** any analytics table — the CSV matches what is on screen, filters and sort order included.
6. **Run a search** — on the group page, *Price Search* runs the saved configuration. Job progress updates per hotel; results land in the report and flow into analytics automatically.

> **A one-minute path:** log in → *Thailand Demo Hotels* → **Analytics** → sort the Competitor Heatmap by cheapest. That single screen shows the comparison basis, the winner highlight, and the deliberately-empty Gother column.

### 3.2 Run it locally

Prerequisites: Docker and Docker Compose.

```bash
git clone <repo>
cd gother-price-code
cp .env.example .env

# Two values are required:
#   SERPAPI_KEY  — the live price source
#   JWT_SECRET   — openssl rand -base64 48   (the backend refuses to start without it)

docker compose up -d
```

Then open **http://localhost:3000**. Migrations run automatically at startup and an `admin` user is created on first boot — set `ADMIN_PASSWORD` in `.env` beforehand, or the default `admin1234!` is used and logged as a warning.

Backend tests: `cd backend && cargo test` (78 tests).

---

## 4. Architecture — top view

### 4.1 System layers

```
                          ┌────────────────────────┐
                          │        Browser         │
                          │   React 18 + Vite SPA  │
                          └───────────┬────────────┘
                                      │ HTTPS
                          ┌───────────▼────────────┐
                          │         Caddy          │  TLS termination
                          │  auto Let's Encrypt    │  ports 80/443 — the
                          └───────────┬────────────┘  only ones published
                                      │
                          ┌───────────▼────────────┐
                          │         nginx          │  serves the SPA,
                          │   static + /api proxy  │  proxies /api
                          └───────────┬────────────┘
                                      │
        ┌─────────────────────────────▼─────────────────────────────┐
        │                  Rust backend (Axum)                      │
        │                    ONE process                            │
        │  ┌──────────┐ ┌──────────┐ ┌───────────┐ ┌─────────────┐  │
        │  │   HTTP   │ │  Queue   │ │   Cron    │ │  Partition  │  │
        │  │   API    │ │ consumer │ │ scheduler │ │   manager   │  │
        │  │ 48 routes│ │          │ │           │ │             │  │
        │  └──────────┘ └──────────┘ └───────────┘ └─────────────┘  │
        │        auth middleware guards everything but              │
        │              /health and /auth/login                      │
        └───┬──────────────┬───────────────┬────────────────┬───────┘
            │              │               │                │
    ┌───────▼──────┐ ┌─────▼─────┐  ┌──────▼──────┐  ┌──────▼───────┐
    │  PostgreSQL  │ │   Redis   │  │  RabbitMQ   │  │  SerpAPI /   │
    │  16          │ │  cache    │  │   queue     │  │  Gemini      │
    │              │ │  (TTL)    │  │             │  │  (external)  │
    │ partitioned  │ └───────────┘  └─────────────┘  └──────────────┘
    │ history +    │
    │ 6 mat. views │
    └──────────────┘
```

**One process, four responsibilities** — and that shapes the whole deployment. The cron scheduler checks whether a job is due and records that it ran *afterwards*, with no lock. Two concurrent backend instances would both see "due" and both fire the full grid, doubling API spend against a limited quota. **Exactly one backend instance may run**, which is why this is a single VM rather than an autoscaling service.

### 4.2 One price search, end to end

```
User clicks "Price Search"
   │
   ▼
POST /api/scrape-jobs ─────────► job row created (status: pending)
   │                                      │
   │                                      ▼
   │                            published to RabbitMQ
   │                                      │
   ▼                                      ▼
UI polls job status            Worker consumes the message
   │                                      │
   │                                      ▼
   │                            Redis cache hit? ──yes──► reuse, no API call
   │                                      │ no
   │                                      ▼
   │                            SerpAPI (Google Hotels)
   │                                      │
   │                            rows returned? ──no──► Gemini fallback
   │                                      │            (badged as an estimate)
   │                                      ▼
   │                            normalise: provider name, currency → THB,
   │                                       room type, meal plan
   │                                      │
   │                                      ▼
   │                            write to hotel_price_history
   │                            (with via_method provenance)
   │                                      │
   │                                      ▼
   │                            REFRESH MATERIALIZED VIEWS
   │                                      │
   ▼                                      ▼
Job completes ◄──────────────── analytics reflect the new data
```

### 4.3 Components and why each is there

| Component | Role | Why this one |
|---|---|---|
| **Rust + Axum** | API and workers | Type safety and low memory — the whole stack fits a 2 GB VM |
| **PostgreSQL 16** | Price history, partitioned; materialized views | Native partitioning and materialized views cover the analytics workload without a separate warehouse |
| **RabbitMQ** | Job queue | Decouples the request from the scrape, so a 20-hotel job doesn't block an HTTP response |
| **Redis** | Result cache (TTL) | Stops repeat searches re-spending API quota. Pure cache — losing it costs nothing but re-scrapes |
| **React 18 + TypeScript + Vite** | SPA | Strict mode; TanStack Query for server state; Recharts for charts |
| **nginx** | Static serving + `/api` proxy | Makes the whole app same-origin, so the session cookie needs no CORS machinery |
| **Caddy** | TLS termination | Automatic Let's Encrypt certificates with no DNS to configure |
| **SerpAPI** | Live price source | The only source returning real, date-specific rates across OTAs |
| **Gemini** | Marked fallback | Fills gaps only where scraping found nothing; never overwrites a scraped price |

### 4.4 Deployment

One `e2-small` GCE VM in `asia-southeast1` (Singapore, nearest to Bangkok) running all six containers, at roughly **$20/month**.

- Images built by **Cloud Build** → **Artifact Registry** (development machines are arm64, the VM is amd64)
- Secrets in **Secret Manager**, never in the repository
- **Only ports 80 and 443 are reachable.** Postgres, Redis, RabbitMQ and the backend publish no host ports; SSH is via IAP tunnel only, and the default network's public SSH/RDP rules were deleted
- One-command redeploy (`./deploy.sh`) that refuses to report success unless an unauthenticated request still returns 401

---

## 5. Key engineering decisions

Each of these is recorded in full, with alternatives and evidence, in `docs/decisions/`.

**No silent fallback to fabricated data** *(ADR-008)*. With no API key configured, the system used to fall back to a mock scraper and report success. It logged 52/52 hotels scraped over 315 rows that were entirely invented, and those rows were indistinguishable from real ones in every view — they had to be deleted wholesale rather than filtered. The fallback was removed: a missing credential now fails loudly.

**The comparison unit is (hotel, check-in date)** *(ADR-013)*. Hotel prices depend heavily on booking lead time. The trend chart and benchmark were mixing +30-day and +35-day quotes into one comparison. Everything now compares within a single stay, and stays quoted by fewer than two providers are excluded from win-rate figures because a sole quote is trivially "cheapest".

**SerpAPI primary, Gemini a marked fallback** *(ADR-011)*. Measured on a real hotel, Gemini quoted Agoda, Trip.com and Booking.com at an identical ฿6,551 when the true Trip.com rate was ฿6,773. AI estimates are useful for filling a hole; they are not measurements, so they are tiered below scraping, badged in the UI, and recorded per row.

**Exact provider matching** *(ADR-009)*. Substring matching filed EaseMyTrip.com and Clicktrip.com as Trip.com. The allowlist was widened to ten real providers and matching made exact.

**Cookie sessions, and no fallback signing key** *(ADR-014)*. The session token lives in an httpOnly cookie — unreadable by injected JavaScript, and with `SameSite=Strict` no CSRF token is needed. The backend refuses to start without `JWT_SECRET`: a default key would let anyone with the source forge a session, making the login screen decorative.

**Single-VM deployment** *(ADR-015)*. Chosen over Cloud Run because the scheduler must be a singleton and RabbitMQ has no managed GCP equivalent — Cloud Run would have needed pinning to exactly one instance *and* a Pub/Sub rewrite, at four times the cost. Redis stayed a container rather than Memorystore: it is a pure cache with three call sites, and Memorystore's smallest tier costs more than the rest of the deployment combined.

---

## 6. Where we departed from the brief

Three things the brief asked for are not in the product. Each was built or attempted, measured, and then removed on the evidence. We would rather show the measurement than quietly ship a feature that displays a number we know to be meaningless.

### ChatGPT scraping method — removed *(ADR-007)*
**The brief offered bonus points for it.** We implemented an AI price method and tested it against known-true rates. It fabricated: three OTAs quoted at an identical ฿6,551 when the real Trip.com rate was ฿6,773. An AI answering from training data cannot know tonight's rate, and a fabricated price is worse than a missing one because it is indistinguishable from a measurement.

**What we built instead:** AI retained as an explicitly-marked fallback tier that runs only where scraping returned nothing, never overwrites a scraped price, and stamps `via_method` on every row so provenance is visible in the UI and queryable in the database.

### Mobile vs desktop cross-tracking — removed *(ADR-010)*
The brief made this mandatory. SerpAPI's Google Hotels engine documents no `device` parameter and silently ignores one. We tested anyway: three hotels, **69 common sources, zero price differences**. The feature was writing two identical rows for every observation, doubling API cost against a limited quota, and the UI comparison could only ever display zero — presenting a measured finding where none existed.

**What we built instead:** the device column is retained on every row (it accurately records the collection condition, and a future direct-OTA scraper could genuinely vary by device), but the axis is never varied. The freed budget went into the five-window booking grid, which does produce real signal.

### Member / logged-in rates — removed
No API parameter exposes them, and we hold no credentials for the target sites. Member pricing is **unobservable**, not merely unimplemented. The login-state column is retained and fixed at `public`, so every row states the condition it was captured under.

### Gother's own price — absent, and deliberately visible
Not an oversight. We checked: **36 distinct sources across 5 hotels, Gother never among them** — Gother does not appear in Google Hotels, so SerpAPI cannot return it.

We were asked whether to fill the gap with Gemini estimates and declined. A fabricated own-price sitting in a row of real competitor prices is worse than a blank, because the entire purpose of the row is the comparison, and the "cheapest" highlight would start awarding wins to a guess.

**What we built instead:** the Gother column is *pinned first and always rendered*, with an explanatory note when empty — the gap is asserted rather than hidden. The Gother API scraper is written, registered and tested; it populates the column the moment `GOTHER_API_URL` and `GOTHER_API_KEY` are supplied, with no code change.

---

## 7. Limitations and roadmap

Stated plainly, because a platform whose value is data honesty should be honest about itself.

### Known limitations

| Limitation | Detail |
|---|---|
| **No automated backups** | A deliberate cost trade on the deployment. A lost disk loses all price history permanently — it cannot be re-scraped. Reversible in about ten minutes with a scheduled dump to Cloud Storage |
| **Roles are not enforced** | Stored, carried and displayed; no endpoint restricted by them. A `viewer` can currently do everything an `admin` can |
| **No session revocation** | A stolen cookie stays valid up to 12 hours; changing the password does not invalidate outstanding sessions |
| **Single point of failure** | One VM. A zone outage means rebuild and restore |
| **Six bad price rows** | Four Seasons Koh Samui rows scraped at ฿52–88 (a parse artefact) remain in the data and distort that hotel's minimum. Identified, not yet deleted |
| **Data volume is demo-scale** | 23 hotels, 601 price rows, 8 providers. The import path handles the full 2200-hotel list; the seeded set is deliberately small because SerpAPI quota is roughly 250 searches/month and one full scheduled run over 20 hotels costs 100 |
| **Gother and Wink have no source** | See §6 |

### Roadmap

**Immediate** — enforce roles on write endpoints; connect the Gother API; automated backups; delete the six bad rows.

**Phase 2 — Experiences** *(REQ-004, descoped for this submission)*. The queue, history store and analytics are product-agnostic by design; adding a product type is a scraper plus a dimension, not a new platform.

**Phase 3 — Flights.** Same shape.

**Forecasting** *(REQ-006, descoped)*. Blocked on data, not engineering: meaningful price forecasting needs roughly six months of accumulated history, which cannot exist yet regardless of effort. The collection standard in §2.2 exists precisely so that history will be usable when it arrives.

---

## 8. Appendix

### 8.1 API surface — 48 endpoints

| Area | Endpoints |
|---|---|
| **Auth** | `POST /auth/login` · `POST /auth/logout` · `GET /auth/me` · `POST /auth/change-password` |
| **Hotel groups** | `GET|POST /hotel-groups` · `GET|PUT|DELETE /hotel-groups/:id` · `POST /hotel-groups/:id/hotels` · `DELETE /hotel-groups/:group_id/hotels/:hotel_id` · `POST /hotel-groups/:id/import` · `POST /hotel-groups/:id/import-master` · `GET /hotel-groups/:id/jobs` · `PUT /hotel-groups/:id/search-config` · `POST /hotel-groups/:id/search-runs` |
| **Hotels** | `GET /hotels` · `GET /hotels/search` · `GET /hotels/countries` · `GET /hotels/cities` · `GET /hotels/export` · `GET|PUT /hotels/:id` |
| **Scrape jobs** | `POST /scrape-jobs` · `POST /scrape-jobs/with-overrides` · `GET|DELETE /scrape-jobs/:id` · `GET /scrape-jobs/:id/results` · `GET /scrape-jobs/:id/export` |
| **Price history** | `GET /price-history` · `GET /price-history/hotel/:id/trend` · `GET /price-history/hotel/:id/trend/windows` · `GET /export/price-history` |
| **Scheduling** | `GET|POST /scheduled-scrape-configs` · `PUT|DELETE /scheduled-scrape-configs/:id` · `POST /scheduled-scrape-configs/:id/run` |
| **Analytics** | `GET /analytics/overview` · `/market-position` · `/heatmap` · `/win-rate` · `/provider-benchmark` · `/parity-violations` · `/booking-window/:hotel_id` · `/export` |
| **Other** | `GET /health` · `GET /templates/hotel-import` |

All routes require a session cookie except `/health` and `/auth/login`. The guard wraps the entire router, so a route added later is protected by default.

### 8.2 Data model

**Core:** `hotel_groups` · `hotels` · `hotel_group_members` · `scrape_jobs` · `scrape_results` · `scrape_hotel_status` · `scrape_job_hotel_params` · `users`

**History:** `hotel_price_history` — monthly range partitions, one row per (hotel, source, stay, observation), carrying `price_thb`, `checkin_date`, `days_in_advance`, `source_url`, `scraped_at`, `via_method`, `device`, `login_state`

**Supporting:** `scheduled_scrape_configs` · `currency_exchange_rates`

**Materialized views (6, refreshed after each job):** `mv_hotel_market_position` · `mv_hotel_price_by_stay` · `mv_hotel_daily_avg_price` · `mv_hotel_win_rate` · `mv_hotel_booking_window` · `mv_hotel_parity_violations`

### 8.3 Documentation index

| To understand… | Read |
|---|---|
| Core scraping, Excel import, evidence | `REQ-001-v1.5` |
| Price history and automated scraping | `REQ-002-v1.3` |
| Analytics dashboard | `REQ-003-v1.2` |
| Data platform and partitioning | `REQ-005-v1.2` |
| Hotel directory | `REQ-007` |
| The standard booking-window grid | `REQ-008-v1.1` |
| Authentication | `REQ-009-v1.0` |
| Deployment and operations | `REQ-010` |
| Why decisions were made | `docs/decisions/ADR-001` … `ADR-015` |

### 8.4 Verification evidence

- **78 backend tests pass**; frontend type-checks clean under TypeScript strict mode
- **All 26 migrations replay cleanly** into an empty database
- **Deployment verified end to end:** valid Let's Encrypt certificate; ports 5432/6379/5672/15672/8080/22 all closed from the internet; `/api/hotels` returns 401 without a session; login sets `HttpOnly; SameSite=Strict; Secure`; VM reboot restored service in 45 seconds reusing the certificate
- **Live scrape through the public URL:** job completed, price history 594 → 601 rows, exercising queue, worker, external API and view refresh together
- **Data integrity claims spot-checked against raw SQL** — the heatmap's highlighted cheapest cell for Shangri-La Bangkok matches `mv_hotel_price_by_stay` exactly, and no hotel row mixes check-in dates

---

## Change Log

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-17 | Initial submission document for Gother Challenge 2026 |
