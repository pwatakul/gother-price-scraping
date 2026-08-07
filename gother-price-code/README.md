# 🏨 Gother Market Intelligence Platform

**Gother Challenge 2026 Competition Entry**

A hotel price intelligence platform: scrapes prices from multiple sources (Google Hotels via SerpAPI, ChatGPT, Gother's own API), stores every observation permanently, and surfaces analytics on where Gother wins and loses on price — and how that changes over time.

## 🎯 Features

- **Multi-method price scraping**: SerpAPI (Google Hotels aggregator), ChatGPT (structured JSON prompting), Gother internal API — pluggable scraper adapter/registry, easy to add another source
- **Hotel group management**: organize hotels into groups, Excel import (hotel-list format) / export (comparison report with evidence columns)
- **Global hotel directory ("All Hotels")**: every hotel ever scraped, in one place — country/city filters, search, pagination, export
- **Price history**: every scrape observation kept permanently in a monthly-partitioned table (`hotel_price_history`), with a full raw-history table and trend chart on each hotel's page
- **Scheduled scraping**: cron-style configs run scrape jobs automatically, no manual trigger needed
- **Analytics dashboard**: market overview, price trend chart, market position table, competitor heatmap, win-rate, date-range filter — backed by materialized views refreshed after every job
- **Evidence & apples-to-apples**: each price row carries its source URL, scraped-at timestamp, room type, meal plan, cancellation policy; ⚠️ badge flags mismatches against Gother's terms
- **Price normalization**: currency → THB, room type / meal plan standardization, optional Gemini AI assist
- **Caching**: Redis-based, avoids redundant API calls within the TTL window

## 🏗️ Tech Stack

| Layer | Technology |
|-------|------------|
| **Backend** | Rust + Axum, SQLx (PostgreSQL) |
| **Frontend** | TypeScript (strict) + React 18 + Vite, TanStack Query, recharts |
| **Database** | PostgreSQL — 16 migrations (core schema, monthly-partitioned price history, currency rates, scheduled configs, materialized views) |
| **Cache** | Redis |
| **Queue** | RabbitMQ |
| **Styling** | Tailwind CSS + shadcn/ui (Radix UI) |
| **Scraping** | SerpAPI (Google Hotels), OpenAI ChatGPT, Gother internal API |
| **AI** | Google Gemini (optional, normalization only) |

## 📋 Prerequisites

- [Docker](https://www.docker.com/) & Docker Compose (only requirement to run the full stack)
- Optional, for scraping real data instead of mock prices: a [SerpAPI key](https://serpapi.com/) and/or an [OpenAI key](https://platform.openai.com/)
- Optional, for local (non-Docker) backend/frontend dev: [Rust](https://rustup.rs/) (1.77+), [Node.js](https://nodejs.org/) (20+)

## 🚀 Quick Start (Docker — recommended for judging)

```bash
cd gother-price-code
cp .env.example .env
# Edit .env and add SERPAPI_KEY / OPENAI_API_KEY / GOTHER_API_KEY if you have them.
# Without any keys, the worker falls back to MockScraper (random but realistic prices) —
# the full app works end-to-end with zero configuration.

docker-compose up -d --build
```

Migrations run automatically on backend startup — no manual `sqlx migrate run` needed.

- Frontend: **http://localhost:3000**
- Backend API: **http://localhost:8080**
- Health check: `curl http://localhost:8080/api/health`

To reset to a totally clean state: `docker-compose down -v && docker-compose up -d --build`.

## 🧭 Trying it out (demo flow)

1. Open **http://localhost:3000** — Dashboard shows hotel groups.
2. Create a new group, import a hotel list via Excel (template: `GET /api/templates/hotel-import`).
3. Click **New Price Search** → pick a method (SerpAPI / ChatGPT / Both) → start.
4. Watch job progress (polls every 3s) → open the **Price Comparison Report** on completion.
5. Expand a hotel row to see per-source evidence (URL, scraped-at, room type/meal plan) and any ⚠️ mismatch badges.
6. Open **All Hotels** in the sidebar — every hotel tracked, filterable by country/city, with export.
7. Click into a hotel's detail page for its trend chart and full raw price-history table.
8. Open **Analytics** (under Hotels in the sidebar) for the market overview, trend chart, position table, and competitor heatmap.
9. Export a report (Excel) or price history (CSV/JSON) from the relevant page.

## 🔌 Local (non-Docker) Dev Setup

```bash
# Infrastructure only
docker-compose up -d postgres redis rabbitmq

# Backend
cd backend
cp ../.env.example .env   # set DATABASE_URL/REDIS_URL/RABBITMQ_URL to localhost, add API keys
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
cargo run          # http://localhost:8080

# Frontend (separate terminal)
cd frontend
npm install
npm run dev         # http://localhost:3000, proxies API calls to :8080
```

## 📁 Project Structure

```
gother-price-code/
├── backend/
│   ├── src/
│   │   ├── main.rs           # Entry point — spawns HTTP server, worker, scheduler, partition manager
│   │   ├── config.rs         # Configuration
│   │   ├── error.rs          # AppError → AppResult mapping
│   │   ├── api/               # HTTP handlers + router
│   │   ├── db/                 # Database repositories
│   │   ├── cache/             # Redis operations
│   │   ├── queue/             # RabbitMQ pub/sub
│   │   ├── worker/            # Background job processor, scheduler, partition manager
│   │   ├── scraper/           # SerpAPI / ChatGPT / Gother / mock scrapers + adapter registry
│   │   ├── normalizer/        # Currency/room-type/meal-plan normalization
│   │   ├── excel/             # Excel read/write
│   │   ├── ai/                # Gemini AI integration
│   │   └── models/            # Data structures
│   └── migrations/            # 16 SQL migrations
├── frontend/
│   ├── src/
│   │   ├── api/                # API client
│   │   ├── components/        # React components (incl. layout/Sidebar)
│   │   ├── pages/              # Dashboard, HotelGroupDetail, ReportView, HotelsList, HotelDetail, AnalyticsDashboard
│   │   ├── types/               # TypeScript types
│   │   └── utils/               # Utilities
│   └── public/
└── docker/
    ├── Dockerfile.backend
    ├── Dockerfile.frontend
    └── nginx.conf
```

## 🔌 API Endpoints

### Hotel Groups
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/hotel-groups` | List all groups |
| POST | `/api/hotel-groups` | Create group (with optional Excel) |
| GET | `/api/hotel-groups/:id` | Get group with hotels |
| PUT | `/api/hotel-groups/:id` | Update group |
| DELETE | `/api/hotel-groups/:id` | Delete group |
| POST | `/api/hotel-groups/:id/import` | Import hotels from Excel |
| POST | `/api/hotel-groups/:id/import-master` | Import from the master hotel-list format |
| POST | `/api/hotel-groups/:id/hotels` | Add single hotel |
| DELETE | `/api/hotel-groups/:group_id/hotels/:hotel_id` | Remove hotel |
| GET | `/api/hotel-groups/:id/jobs` | List scrape jobs for this group |

### Hotel Directory ("All Hotels")
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/hotels` | Paginated/filterable list of every hotel tracked |
| GET | `/api/hotels/countries` | Distinct countries for the filter dropdown |
| GET | `/api/hotels/cities` | Distinct cities for the filter dropdown |
| GET | `/api/hotels/export` | Export the directory (csv/json) |
| GET | `/api/hotels/:id` | Hotel detail (profile + trend summary) |
| GET | `/api/hotels/search` | Search existing hotels by name |

### Scrape Jobs
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/scrape-jobs` | Create new scrape job |
| POST | `/api/scrape-jobs/with-overrides` | Create a job with per-hotel param overrides |
| GET | `/api/scrape-jobs/:id` | Get job status + progress |
| DELETE | `/api/scrape-jobs/:id` | Cancel running job |
| GET | `/api/scrape-jobs/:id/results` | Get price comparison results |
| GET | `/api/scrape-jobs/:id/export` | Export results as Excel |

### Price History
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/price-history` | Filtered raw price-history rows (paginated) |
| GET | `/api/price-history/hotel/:id/trend` | Aggregated trend for one hotel |
| GET | `/api/export/price-history` | Export full price history (csv/json), by hotel or group |

### Scheduled Scraping
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/scheduled-scrape-configs` | Create a scheduled scrape config (cron expression) |
| GET | `/api/scheduled-scrape-configs` | List configs |
| PUT | `/api/scheduled-scrape-configs/:id` | Update a config |
| DELETE | `/api/scheduled-scrape-configs/:id` | Delete a config |

### Analytics
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/analytics/overview` | Market overview card metrics |
| GET | `/api/analytics/market-position` | Per-hotel Gother vs. best-OTA position table |
| GET | `/api/analytics/heatmap` | Hotel × source gap heatmap |
| GET | `/api/analytics/win-rate` | % of samples where Gother was cheapest |
| GET | `/api/analytics/parity-violations` | Rate parity violations |
| GET | `/api/analytics/booking-window/:hotel_id` | Booking-window lead-time chart data |
| GET | `/api/analytics/export` | Export analytics data |

### Other
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/templates/hotel-import` | Download Excel import template |
| GET | `/api/health` | Health check (db/redis/rabbitmq status) |

## 🧪 Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend type-check
cd frontend
npm run build
```

## 📊 Data Flow

```
1. User uploads Excel + sets search params (or a scheduled config fires)
   ↓
2. API creates ScrapeJob → PostgreSQL (status: PENDING) → published to RabbitMQ
   ↓
3. Worker consumes job → status: PROCESSING
   ↓
4. For each hotel (WORKER_CONCURRENCY in parallel):
   ├─ Check Redis cache
   ├─ Call the selected scraper(s) via the adapter registry (SerpAPI / ChatGPT / Gother / mock)
   ├─ Normalize data (currency, room type, meal plan)
   └─ Write to scrape_results AND hotel_price_history (dual-write)
   ↓
5. Job completes → status: COMPLETED → materialized views refreshed
   ↓
6. Frontend polls job status; report, All Hotels, hotel detail, and Analytics pages
   all read from the now-current data
```

## 🔧 Configuration

See `.env.example` for the full list. Key ones:

| Variable | Default | Description |
|----------|---------|-------------|
| `WORKER_CONCURRENCY` | 3 | Hotels processed in parallel |
| `WORKER_RETRY_COUNT` | 1 | Retries on failure |
| `PRICE_CACHE_TTL_SECONDS` | 3600 | Redis price cache duration |
| `SERPAPI_KEY` | _(empty)_ | Leave empty to use `MockScraper` instead |
| `OPENAI_API_KEY` | _(empty)_ | Leave empty to skip the ChatGPT scraper |
| `GOTHER_API_KEY` / `GOTHER_API_URL` | _(empty)_ | Leave empty — Gother source returns no rows, doesn't error |

## 🏆 Competition Notes

Built for the **Gother Challenge 2026**. Cloud deployment (Google Cloud Run) is intentionally out of scope for this submission — the focus has been the full functional pipeline: scraping (3 methods), permanent price history, scheduled automation, and analytics, all runnable locally via a single `docker-compose up`.

### Key Differentiators
1. **Pluggable scraper adapters** — adding a new price source is a new `ScraperFactory` impl, not a rewrite
2. **Permanent, partitioned price history** — every observation kept, auto-partitioned monthly with no manual maintenance
3. **Real analytics, not just a report** — market position, trends, heatmap, win rate, all backed by materialized views refreshed after every job
4. **Evidence-first UI** — every price traceable to its source URL and scrape timestamp, mismatches flagged automatically
5. **Zero-config demo path** — no API keys required to see the full app working end-to-end (mock scraper fallback)

## 📝 License

MIT License - Built for Gother Challenge 2026

---

**Good luck! 🎉**
