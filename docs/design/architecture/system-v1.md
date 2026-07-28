---
title: System Architecture v1
type: design
version: "1.0"
updated: 2026-04-27
status: Draft
tags: [design, architecture, system]
related: ["[[REQ-001-v1.1]]", "[[REQ-002-v1.0]]", "[[REQ-003-v1.0]]", "[[data-model-v1.1]]"]
---

# System Architecture v1

## Component Overview

```mermaid
graph TB
    subgraph Client["🖥️ Client Layer"]
        UI["React Frontend\n(Vite + TypeScript)\nlocalhost:3000"]
    end

    subgraph Gateway["🔀 API Gateway"]
        NGINX["Nginx\nReverse Proxy"]
    end

    subgraph Backend["⚙️ Backend (Rust + Axum) :8080"]
        API["REST API\nRouter"]
        ExcelR["Excel Reader\n(calamine)"]
        ExcelW["Excel Writer\n(rust_xlsxwriter)"]
        Norm["Normalizer\nroom_type · meal_plan\ncurrency → THB"]
        Pub["Queue Publisher\n(lapin)"]
    end

    subgraph Infra["🗄️ Infrastructure"]
        PG[("PostgreSQL\nMain DB")]
        Redis[("Redis\nPrice Cache")]
        RMQ[("RabbitMQ\nJob Queue")]
    end

    subgraph Worker["🔧 Background Worker (Rust)"]
        Con["Queue Consumer\n(lapin)"]
        Proc["Job Processor\n(parallel batches)"]
        Cron["Cron Scheduler\n(scheduled_scrape_configs)"]
    end

    subgraph Scrapers["🌐 Scraper Modules"]
        Serp["SerpApiScraper\nGoogle Hotels\n(Method 2 ✅)"]
        Gother["GotherScraper\nGother Hotel API\n(required ✅)"]
        GPT["ChatGptScraper\nOpenAI API\n(Method 1 🎯 bonus)"]
        Mock["MockScraper\n(dev/test only)"]
    end

    subgraph External["☁️ External APIs"]
        SerpAPI["SerpAPI\nserpapi.com"]
        GotherAPI["Gother Hotel\nPrice Search API"]
        OpenAI["OpenAI\nChatGPT API"]
    end

    UI -->|HTTP/JSON| NGINX
    NGINX -->|proxy| API
    API --> ExcelR
    API --> ExcelW
    API --> Norm
    API --> Pub
    API <-->|read/write| PG
    API <-->|cache| Redis
    Pub -->|publish job| RMQ

    Con -->|consume job| RMQ
    Con --> Proc
    Cron -->|publish scheduled job| RMQ
    Cron -->|read configs| PG

    Proc --> Serp
    Proc --> Gother
    Proc --> GPT
    Proc --> Mock
    Proc -->|write results + price_history| PG
    Proc -->|cache prices| Redis

    Serp --> SerpAPI
    Gother --> GotherAPI
    GPT --> OpenAI
```

---

## Data Flow — Competition Demo Path

```mermaid
sequenceDiagram
    actor User
    participant UI as React Frontend
    participant API as Backend API
    participant RMQ as RabbitMQ
    participant Worker as Job Processor
    participant SerpAPI as SerpAPI
    participant GotherAPI as Gother API
    participant DB as PostgreSQL
    participant Cache as Redis

    User->>UI: Upload Excel\n(hotel_name, city, checkin,\ncheckout, rooms, adults)
    UI->>API: POST /hotel-groups/:id/import\n(multipart Excel)
    API->>DB: parse rows → upsert hotels
    API-->>UI: { imported_count }

    User->>UI: Create Scrape Job\n(select group, method)
    UI->>API: POST /scrape-jobs
    API->>DB: INSERT scrape_job (pending)
    API->>DB: INSERT scrape_hotel_status per hotel
    API->>RMQ: publish ScrapeJobMessage
    API-->>UI: { job_id, status: pending }

    loop Poll every 3s
        UI->>API: GET /scrape-jobs/:id
        API-->>UI: { progress: { total, completed, failed } }
    end

    RMQ->>Worker: consume ScrapeJobMessage
    Worker->>DB: UPDATE job status → processing

    loop For each hotel (parallel batch of 3)
        Worker->>Cache: check price cache
        alt Cache hit
            Cache-->>Worker: cached prices
        else Cache miss
            Worker->>SerpAPI: search hotel prices
            SerpAPI-->>Worker: OTA prices (Agoda, Booking, Trip...)
            Worker->>GotherAPI: search hotel prices
            GotherAPI-->>Worker: Gother prices
        end
        Worker->>Worker: normalize\n(room_type, meal_plan, currency→THB)
        Worker->>DB: INSERT scrape_results
        Worker->>DB: INSERT price_history
        Worker->>Cache: SET price cache (1hr TTL)
        Worker->>DB: UPDATE scrape_hotel_status → success/failed
    end

    Worker->>DB: UPDATE job status → completed

    User->>UI: View results
    UI->>API: GET /scrape-jobs/:id/results
    API->>DB: query scrape_results + hotel info
    API-->>UI: ScrapeResultsResponse\n(per hotel: all sources, best price,\ngother price, diff, URL, scraped_at)
    UI-->>User: Price Comparison Table

    User->>UI: Export Excel
    UI->>API: GET /scrape-jobs/:id/export
    API-->>UI: .xlsx download
    UI-->>User: hotel-price-report.xlsx
```

---

## Scraper Strategy

```mermaid
flowchart LR
    Job["Scrape Job\nCreated"]
    Check{"API Keys\nConfigured?"}
    Method{"Scraping\nMethod"}
    
    Mock["MockScraper\nRandom prices\n(dev only)"]
    
    M2["Method 2\nSerpAPI → Google Hotels\n+ Gother API"]
    M1["Method 1 🎯\nChatGPT Prompt\n+ Gother API"]
    Both["Both Methods\n(bonus points ✅)"]
    
    Merge["Merge + Deduplicate\nby source name"]
    Normalize["Normalize\nroom_type · meal_plan\ncurrency → THB"]
    Save["Save to\nscrape_results\n+ price_history"]

    Job --> Check
    Check -->|No keys| Mock
    Check -->|Keys set| Method
    Method -->|method=serpapi| M2
    Method -->|method=chatgpt| M1
    Method -->|method=both| Both
    M2 --> Merge
    M1 --> Merge
    Both --> Merge
    Mock --> Normalize
    Merge --> Normalize
    Normalize --> Save
```

---

## Deployment Architecture

```mermaid
graph LR
    subgraph Docker Compose
        FE["frontend\nnginx:3000"]
        BE["backend\nrust:8080"]
        PG[("postgres\n:5432")]
        RD[("redis\n:6379")]
        RMQ[("rabbitmq\n:5672\n:15672 mgmt")]
    end

    Browser["Browser"] --> FE
    FE -->|proxy /api| BE
    BE --> PG
    BE --> RD
    BE --> RMQ
    BE -.->|worker process| RMQ
```

---

## Key Design Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| Backend language | Rust + Axum | Performance, safety, competition differentiator |
| Async job processing | RabbitMQ | Decouples slow scraping from HTTP response |
| Price cache | Redis | Avoid redundant API calls within 1-hour window |
| Scraping: OTA | SerpAPI (Google Hotels) | Aggregates Agoda, Booking, Trip.com in one call |
| Scraping: Gother | Gother internal API | Required by competition |
| Scraping: Bonus | ChatGPT (OpenAI) | Method 1 — bonus points |
| Data persistence | PostgreSQL | Single store for all data; sufficient for competition scale |
| History table | Partitioned by month | Scale: 1M+ rows/year without performance loss |
| Concurrency | 3 hotels parallel | Balance speed vs. API rate limits |

---

## Decisions

| Decision | Answer |
|----------|--------|
| `ScrapingMethod` stored in DB | ✅ Yes — `scrape_jobs.method` column. Persisted so results always show which method was used |
| ChatGPT response format | ✅ Prompt requests strict JSON (`ChatGptHotelPriceJson` schema). No free-text parsing |
| Method 1 + 2 results display | ✅ Merged into one comparison table, deduplicated by source name |

## Open Questions
> [!WARNING]
> Still unresolved — must answer before implementation.

- [ ] What is the Gother Hotel Room Price Search API endpoint and auth method? → **Pending** — internal API docs to be supplied separately before GotherScraper implementation.
- [x] Does ChatGPT (Method 1) call TripAdvisor internally, or do we call TripAdvisor separately? → Resolved: **ChatGPT calls TripAdvisor internally** as part of its own lookup. We do not integrate TripAdvisor API directly.
- [x] Should we integrate TripAdvisor API directly as an OTA source? → Resolved: **No** — out of scope for Phase 1.
- [ ] Rate limits on SerpAPI plan — how many requests/day? → **Pending** — need to check account dashboard before setting `WORKER_CONCURRENCY` and scheduled run frequency.

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-04-27 | Initial draft — competition submission architecture |
