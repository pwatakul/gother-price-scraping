# 🏨 Hotel Price Scraper

**Gother Challenge 2026 Competition Entry**

A powerful hotel price comparison tool that scrapes prices from multiple OTAs (Agoda, Booking.com, Trip.com, etc.) and compares them against Gother's prices.

## 🎯 Features

- **Multi-OTA Price Scraping**: Fetch prices from Google Hotels (via SerpAPI), individual OTAs, and Gother
- **Hotel Group Management**: Organize hotels into groups for batch processing
- **Excel Import/Export**: Upload hotel lists via Excel, download comparison reports
- **Real-time Progress Tracking**: Monitor scraping progress with live updates
- **Price Normalization**: Convert all currencies to THB, standardize room types and meal plans
- **Caching**: Redis-based caching to avoid redundant API calls
- **AI Enhancement**: Optional Gemini AI integration for data normalization

## 🏗️ Tech Stack

| Layer | Technology |
|-------|------------|
| **Backend** | Rust + Axum |
| **Frontend** | TypeScript + React + Vite |
| **Database** | PostgreSQL |
| **Cache** | Redis |
| **Queue** | RabbitMQ |
| **Styling** | Tailwind CSS + shadcn/ui |
| **Scraping** | SerpAPI (Google Hotels) |
| **AI** | Google Gemini (optional) |

## 📋 Prerequisites

- [Rust](https://rustup.rs/) (1.77+)
- [Node.js](https://nodejs.org/) (20+)
- [Docker](https://www.docker.com/) & Docker Compose
- [SerpAPI Key](https://serpapi.com/) (for Google Hotels)

## 🚀 Quick Start

### 1. Clone & Setup Environment

```bash
# Navigate to project
cd hotel-price-scraper

# Copy environment file
cp .env.example .env

# Edit .env and add your API keys
# - SERPAPI_KEY (required)
# - GOTHER_API_KEY (required)
# - GEMINI_API_KEY (optional)
```

### 2. Start Infrastructure

```bash
# Start PostgreSQL, Redis, and RabbitMQ
docker-compose up -d postgres redis rabbitmq

# Wait for services to be ready (~10 seconds)
sleep 10
```

### 3. Setup Backend

```bash
cd backend

# Install sqlx-cli for migrations
cargo install sqlx-cli --no-default-features --features postgres

# Run database migrations
sqlx migrate run

# Build and run backend
cargo run
```

The backend will start at `http://localhost:8080`

### 4. Setup Frontend

```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

The frontend will start at `http://localhost:3000`

### 5. Access the Application

Open your browser and navigate to: **http://localhost:3000**

## 📁 Project Structure

```
hotel-price-scraper/
├── backend/
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   ├── lib.rs            # Library exports
│   │   ├── config.rs         # Configuration
│   │   ├── error.rs          # Error handling
│   │   ├── api/              # HTTP handlers
│   │   ├── db/               # Database repositories
│   │   ├── cache/            # Redis operations
│   │   ├── queue/            # RabbitMQ pub/sub
│   │   ├── worker/           # Background job processor
│   │   ├── scraper/          # OTA scrapers
│   │   ├── normalizer/       # Data normalization
│   │   ├── excel/            # Excel read/write
│   │   ├── ai/               # Gemini AI integration
│   │   └── models/           # Data structures
│   └── migrations/           # SQL migrations
├── frontend/
│   ├── src/
│   │   ├── api/              # API client
│   │   ├── components/       # React components
│   │   ├── pages/            # Page components
│   │   ├── types/            # TypeScript types
│   │   └── utils/            # Utilities
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
| POST | `/api/hotel-groups/:id/hotels` | Add single hotel |
| DELETE | `/api/hotel-groups/:id/hotels/:hotelId` | Remove hotel |

### Scrape Jobs
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/scrape-jobs` | Create new scrape job |
| GET | `/api/scrape-jobs/:id` | Get job status + progress |
| DELETE | `/api/scrape-jobs/:id` | Cancel running job |
| GET | `/api/scrape-jobs/:id/results` | Get price comparison results |
| GET | `/api/scrape-jobs/:id/export` | Export results as Excel |

### Other
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/hotels/search` | Search existing hotels |
| GET | `/api/templates/hotel-import` | Download Excel template |
| GET | `/api/health` | Health check |

## 🧪 Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend tests (if configured)
cd frontend
npm test
```

## 🐳 Docker Deployment

### Build Images

```bash
# Build backend
docker build -f docker/Dockerfile.backend -t hotel-scraper-backend ./backend

# Build frontend
docker build -f docker/Dockerfile.frontend -t hotel-scraper-frontend ./frontend
```

### Run Full Stack

```bash
docker-compose up -d
```

## 📊 Data Flow

```
1. User uploads Excel + sets search params
   ↓
2. API creates ScrapeJob → PostgreSQL (status: PENDING)
   ↓
3. Job published to RabbitMQ
   ↓
4. Worker consumes job → status: PROCESSING
   ↓
5. For each hotel (3 parallel):
   ├─ Check Redis cache
   ├─ Call SerpAPI / Gother API
   ├─ Normalize data
   └─ Save to PostgreSQL + Redis
   ↓
6. Job complete → status: COMPLETED
   ↓
7. Frontend polls → displays results
```

## 🔧 Configuration

### Worker Settings
| Variable | Default | Description |
|----------|---------|-------------|
| `WORKER_CONCURRENCY` | 3 | Hotels processed in parallel |
| `WORKER_RETRY_COUNT` | 1 | Retries on failure |
| `PRICE_CACHE_TTL_SECONDS` | 3600 | Cache duration (1 hour) |

### Rate Limiting
- SerpAPI: Respects plan limits
- Gother API: As configured
- Redis-based rate limiting per API

## 🏆 Competition Notes

This project was built for the **Gother Challenge 2026** with the following goals:
- Win 1st place (฿120,000 prize)
- Learn Rust and TypeScript
- Build a production-ready scraping system

### Key Differentiators
1. **Modern Tech Stack**: Rust for performance, React for UX
2. **Modular Architecture**: Easy to extend with new OTAs
3. **Real-time Updates**: WebSocket-ready progress tracking
4. **AI Integration**: Gemini for smart data normalization
5. **Production Ready**: Docker, caching, error handling

## 📝 License

MIT License - Built for Gother Challenge 2026

---

**Good luck! 🎉**
