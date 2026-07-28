---
title: "ADR-002: Product-Specific Price History Tables"
type: decision
date: 2026-04-27
status: Accepted
tags: [adr, database, architecture, price-history, scalability]
related: ["[[data-model-v1.1]]", "[[REQ-002-v1.0]]", "[[REQ-004-v1.0]]", "[[REQ-005-v1.0]]"]
---

# ADR-002: Product-Specific Price History Tables

## Context
The platform needs to store historical price data permanently for analytics. The system currently handles hotels, and will expand to experiences (Phase 2) and flights (Phase 3). Each product type needs to store price history, but the meaningful fields differ significantly:

| Field | Hotel | Experience | Flight |
|-------|-------|------------|--------|
| room_type | ✅ core field | ❌ meaningless | ❌ meaningless |
| meal_plan | ✅ important | ⚠️ sometimes | ❌ not applicable |
| checkin / checkout | ✅ required | ❌ use activity_date | ❌ use departure_date |
| rooms | ✅ required | ❌ not applicable | ❌ not applicable |
| cabin_class | ❌ not applicable | ❌ not applicable | ✅ core field |
| airline | ❌ not applicable | ❌ not applicable | ✅ core field |
| duration_hours | ❌ not applicable | ✅ core field | ❌ not applicable |

Additionally, currency exchange rates change daily. Storing only the converted THB price without recording the rate used makes historical prices non-auditable and non-recalculable.

## Decision
**Use separate, product-specific price history tables** — one per product type — rather than a single polymorphic table.

- **`hotel_price_history`** — Phase 1 (now), with direct FK to `hotels`
- **`experience_price_history`** — Phase 2, with direct FK to `experiences`
- **`flight_price_history`** — Phase 3, with direct FK to `flight_routes`

All three share a **`currency_exchange_rates`** table and reference it via `exchange_rate_id`.

## Alternatives Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Single `price_history` table with `product_type` + `reference_id`** (original design) | One table to query, simple join pattern | Many NULL columns per row; no FK integrity on `reference_id`; query optimizer can't use indexes efficiently across product types; adding a new product type means altering the shared table |
| **B: Product-specific tables (chosen)** | Strong FK integrity; no NULLs; each table optimized for its product; adding a new product type is additive (new table only, no ALTER); indexes are product-specific and efficient | Multiple tables to manage; analytics across product types requires UNION queries |
| **C: JSONB column for product-specific fields** | Single table, flexible schema | No FK integrity; hard to index; hard to query; bad for analytics |
| **D: Inheritance (PostgreSQL table inheritance)** | Single logical table | PostgreSQL table inheritance has major limitations with partitioning and FK constraints; effectively deprecated for this use case |

## Consequences

### Positive
- **Strong referential integrity** — `hotel_price_history.hotel_id` is a real FK; the database enforces it
- **No wasted storage** — zero NULL columns from mismatched product fields
- **Efficient indexes** — indexes are tuned for hotel queries only; no cross-product noise in the index
- **Additive expansion** — adding `experience_price_history` in Phase 2 requires no changes to `hotel_price_history` or any existing table
- **Clear domain boundaries** — easy to understand what each table contains
- **Auditable currency conversion** — `exchange_rate_id` records exactly which rate was used and from which source

### Negative / Trade-offs
- **Cross-product analytics** need UNION queries (e.g., "total market size across hotels + experiences")
- **More tables to migrate** — each new product adds a migration
- **Materialized views are product-scoped** — `mv_hotel_market_position` is hotel-only; experience analytics needs its own views

### Mitigations
- Cross-product UNION queries are rare at current scale; when needed, wrap them in a view
- The migration pattern is identical for each product type — the Phase 2 template is just `hotel_price_history` with fields swapped

## Currency Exchange Rate Design

A separate `currency_exchange_rates` table stores one row per (from_currency, to_currency, date).

**Why store the rate separately instead of inline?**
- A rate is fetched once per day per currency pair and reused across thousands of rows scraped that day
- If a rate source had an error, all affected history rows can be identified via `exchange_rate_id` and recalculated
- Separating the rate from the price keeps each concern in one place

**Rate source priority:**
1. BOT (Bank of Thailand) — authoritative for THB
2. exchangerate-api.io — fallback if BOT is unavailable
3. `fallback` — use most recent known rate if today's isn't available yet (marked for review)

## Related
- [[data-model-v1.1]] — full schema with `hotel_price_history` and `currency_exchange_rates`
- [[REQ-002-v1.0]] — price history requirements
- [[REQ-004-v1.0]] — multi-product expansion plan (the reason this decision matters)
- [[REQ-005-v1.0]] — data platform and scalability requirements

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-04-27 | Initial — accepted after reviewing polymorphic vs. product-specific tradeoffs |
