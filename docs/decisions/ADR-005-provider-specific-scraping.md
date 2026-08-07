---
title: "ADR-005: Provider-specific scraping — filter SerpAPI to named providers, stub Wink"
type: decision
date: 2026-08-04
status: Accepted
tags: [adr, scraping, serpapi, wink]
---

# ADR-005: Provider-specific scraping — filter SerpAPI to named providers, stub Wink

## Context
The CEO brief requires exactly four named sources per hotel: Gother, Agoda, Trip, and Wink (Wink domestic-only). The existing `SerpApiScraper::normalize_source_name` attributes results to `agoda, booking, trip.com, expedia, hotels.com, official` — more sources than the brief wants, and missing Wink entirely (SerpAPI's Google Hotels aggregation cannot return Wink as a distinct source; it isn't indexed there).

## Decision
1. `normalize_source_name` now returns `Option<String>`, mapping only `agoda` → `"agoda"` and `trip.com`/`ctrip` → `"trip"`; everything else (`booking`, `expedia`, `hotels.com`, `official`) returns `None` and is dropped at the scraper boundary — never appears in a comparison result.
2. `"wink"` is added to `scraper::providers::KNOWN_PROVIDERS` as a recognized name throughout the stack (schema, response types, frontend column), but **no scraper implementation produces it**. It is structurally always-absent and renders blank (REQ-001 F-027 — never a fabricated `0` or mock price), suppressed entirely on the frontend for non-domestic hotels.
3. `MockScraper` was updated to only ever generate `agoda`/`trip`/`gother` — it never fabricates a `"wink"` row, so the blank-rendering behavior is testable even without real API keys.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| Filter to named providers, stub Wink (chosen) | Ships on time; apples-to-apples comparison matches the brief exactly; honest about the Wink gap | Wink column is permanently blank until a real data source is found |
| Mock Wink prices for the demo | Looks complete to judges | Fabricated data — misrepresents a working integration that doesn't exist |
| Investigate a real Wink/HyperGuest source before deciding | Might close the gap for real | Open-ended research against a hard Aug 17 deadline; likely lands on "stub it" anyway after burning sprint time |

## Consequences
### Positive
- Comparison results are exactly the four named providers, no unexpected extra columns.
- No fabricated data anywhere in the pipeline — every price shown is a real scrape result.

### Negative / Trade-offs
- Wink is a permanently blank column until a follow-up investigates a real Wink/HyperGuest integration (not scheduled in this pass).
- `is_direct_contract` (REQ-001 F-026) is wired end-to-end but currently a no-op, since nothing produces a `"wink"` or `"hyperguest"` row yet.

## Related
- REQ: [[REQ-001-v1.3]]
- ADR: [[ADR-001-scraper-choice]]
