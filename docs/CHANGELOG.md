---
title: Changelog
type: changelog
tags: [changelog]
---

# Changelog
All notable changes to this project will be documented here.
Format: `[version] - YYYY-MM-DD`

---

## [Unreleased]
### Added
-

### Changed
-

### Fixed
-

---

## [0.1.0] - 2026-04-22
### Added
- Initial project setup
- Full backend: Rust/Axum REST API, PostgreSQL schema (6 migrations), RabbitMQ worker, Redis caching
- SerpAPI and Gother API scrapers with mock fallback
- Excel import/export (hotel list template + price comparison report)
- Frontend: React + Vite + Tailwind, Dashboard, HotelGroupDetail, ReportView pages
- Docker Compose setup for full stack

---

> [!NOTE]
> Versioning: `MAJOR.MINOR.PATCH` — MAJOR: breaking change; MINOR: new feature; PATCH: bug fix.
> Commit prefixes: `feat:` `fix:` `req:` `doc:` `refactor:`
