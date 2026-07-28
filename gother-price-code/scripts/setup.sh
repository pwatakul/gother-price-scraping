#!/bin/bash
# ===========================================
# Hotel Price Scraper - Development Setup
# ===========================================

set -e

echo "🏨 Hotel Price Scraper - Development Setup"
echo "=========================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo ""
echo "📋 Checking prerequisites..."

check_command() {
    if command -v $1 &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} $1 found"
        return 0
    else
        echo -e "  ${RED}✗${NC} $1 not found"
        return 1
    fi
}

check_command "docker" || { echo "Please install Docker"; exit 1; }
check_command "docker-compose" || check_command "docker compose" || { echo "Please install Docker Compose"; exit 1; }
check_command "cargo" || { echo "Please install Rust: https://rustup.rs/"; exit 1; }
check_command "node" || { echo "Please install Node.js 20+"; exit 1; }
check_command "npm" || { echo "Please install npm"; exit 1; }

# Setup environment
echo ""
echo "🔧 Setting up environment..."

if [ ! -f .env ]; then
    cp .env.example .env
    echo -e "  ${GREEN}✓${NC} Created .env from .env.example"
    echo -e "  ${YELLOW}!${NC} Edit .env to add your API keys (optional for testing with mock data)"
else
    echo -e "  ${GREEN}✓${NC} .env already exists"
fi

# Start Docker services
echo ""
echo "🐳 Starting Docker services..."
docker-compose up -d postgres redis rabbitmq

echo "  Waiting for services to be ready..."
sleep 5

# Check services
echo ""
echo "🔍 Checking services..."

if docker-compose ps | grep -q "postgres.*Up"; then
    echo -e "  ${GREEN}✓${NC} PostgreSQL is running"
else
    echo -e "  ${RED}✗${NC} PostgreSQL failed to start"
fi

if docker-compose ps | grep -q "redis.*Up"; then
    echo -e "  ${GREEN}✓${NC} Redis is running"
else
    echo -e "  ${RED}✗${NC} Redis failed to start"
fi

if docker-compose ps | grep -q "rabbitmq.*Up"; then
    echo -e "  ${GREEN}✓${NC} RabbitMQ is running"
else
    echo -e "  ${RED}✗${NC} RabbitMQ failed to start"
fi

# Install backend dependencies
echo ""
echo "📦 Setting up backend..."
cd backend

if ! command -v sqlx &> /dev/null; then
    echo "  Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

echo "  Running migrations..."
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/hotel_scraper"
sqlx migrate run 2>/dev/null || echo "  (migrations may already be applied)"

cd ..

# Install frontend dependencies
echo ""
echo "📦 Setting up frontend..."
cd frontend
npm install --silent
cd ..

# Done!
echo ""
echo "=========================================="
echo -e "${GREEN}✅ Setup complete!${NC}"
echo ""
echo "To start the application:"
echo ""
echo "  Terminal 1 (Backend):"
echo "    cd backend && cargo run"
echo ""
echo "  Terminal 2 (Frontend):"
echo "    cd frontend && npm run dev"
echo ""
echo "Then open: http://localhost:3000"
echo ""
echo "📝 Note: Without API keys, the app will use mock data for testing."
echo "   Edit .env to add your SERPAPI_KEY for real hotel prices."
echo ""
