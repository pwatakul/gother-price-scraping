# ===========================================
# Hotel Price Scraper - Makefile
# ===========================================

.PHONY: help setup dev dev-backend dev-frontend build test clean docker-up docker-down migrate

# Default target
help:
	@echo "Hotel Price Scraper - Available Commands"
	@echo ""
	@echo "Setup:"
	@echo "  make setup          - Initial setup (copy .env, install deps)"
	@echo "  make migrate        - Run database migrations"
	@echo ""
	@echo "Development:"
	@echo "  make dev            - Start all services (docker + backend + frontend)"
	@echo "  make dev-backend    - Start backend only"
	@echo "  make dev-frontend   - Start frontend only"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-up      - Start infrastructure (postgres, redis, rabbitmq)"
	@echo "  make docker-down    - Stop all docker services"
	@echo "  make docker-all     - Build and run entire stack"
	@echo ""
	@echo "Build & Test:"
	@echo "  make build          - Build backend and frontend"
	@echo "  make test           - Run all tests"
	@echo "  make clean          - Clean build artifacts"

# Setup
setup:
	@echo "Setting up project..."
	@test -f .env || cp .env.example .env
	@echo "Installing backend dependencies..."
	cd backend && cargo fetch
	@echo "Installing frontend dependencies..."
	cd frontend && npm install
	@echo "Setup complete! Edit .env with your API keys, then run 'make dev'"

# Database migrations
migrate:
	@echo "Running database migrations..."
	cd backend && sqlx migrate run

# Development
dev: docker-up
	@echo "Starting development servers..."
	@make -j2 dev-backend dev-frontend

dev-backend:
	@echo "Starting backend..."
	cd backend && cargo run

dev-frontend:
	@echo "Starting frontend..."
	cd frontend && npm run dev

# Docker
docker-up:
	@echo "Starting infrastructure..."
	docker-compose up -d postgres redis rabbitmq
	@echo "Waiting for services to be ready..."
	sleep 5

docker-down:
	@echo "Stopping all docker services..."
	docker-compose down

docker-all:
	@echo "Building and starting entire stack..."
	docker-compose up -d --build

# Build
build:
	@echo "Building backend..."
	cd backend && cargo build --release
	@echo "Building frontend..."
	cd frontend && npm run build

# Test
test:
	@echo "Running backend tests..."
	cd backend && cargo test
	@echo "Running frontend tests..."
	cd frontend && npm test || true

# Clean
clean:
	@echo "Cleaning build artifacts..."
	cd backend && cargo clean
	cd frontend && rm -rf node_modules dist
	@echo "Clean complete!"

# Quick start for competition demo
demo: setup docker-up migrate
	@echo ""
	@echo "🎉 Setup complete!"
	@echo ""
	@echo "To start the application:"
	@echo "  Terminal 1: cd backend && cargo run"
	@echo "  Terminal 2: cd frontend && npm run dev"
	@echo ""
	@echo "Then open http://localhost:3000"
