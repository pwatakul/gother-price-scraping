---
title: "ADR-001: Scraper Approach Choice"
type: decision
date: 2026-08-04
status: Partially superseded
tags: [adr, scraping, architecture]
superseded_by: "[[ADR-007-remove-chatgpt-scraper]]"
---

# ADR-001: Scraper Approach Choice

> **Partially superseded (2026-08-11):** two decisions below no longer hold.
> The ChatGPT scraper ("Method 1", bonus) was removed —
> see [[ADR-007-remove-chatgpt-scraper]]. The `MockScraper` fallback for a
> missing `SERPAPI_KEY` was removed after it wrote fabricated prices into
> `hotel_price_history` and reported them as successful scrapes; mock is now
> opt-in via `ENABLE_MOCK_SCRAPER` — see [[ADR-008-no-silent-mock-fallback]].
> The SerpAPI, Gother and no-fabrication decisions still stand.

## Context
The platform needs hotel price data from multiple OTAs (Agoda, Trip) plus Gother's own internal API, for a competition demo with a hard deadline and no guaranteed access to per-OTA scraping infrastructure or credentials. The brief also awards bonus points for a second, independent method (Method 1: ChatGPT + Gother API) alongside the primary method (Method 2: SerpAPI + Gother API). This ADR was originally left as an unfilled template despite SerpAPI already being the implemented choice — backfilled here (2026-08-04) to actually record the reasoning, now extended to cover the ChatGPT scraper and the provider-naming decision from [[ADR-005-provider-specific-scraping]].

## Decision
- **Method 2 (primary): SerpAPI's Google Hotels engine** for Agoda/Trip price data, plus a direct **Gother internal API** integration for Gother's own rates. SerpAPI's aggregated results are attributed to named providers via `normalize_source_name`, filtered to only `agoda`/`trip` (see [[ADR-005-provider-specific-scraping]]) — no dedicated per-OTA scraper was built for either.
- **Method 1 (bonus): a `ChatGptScraper`** (`backend/src/scraper/chatgpt.rs`) that asks OpenAI's chat completions API for current rates via a strict JSON-schema response, normalized onto the same `agoda`/`trip` naming. Missing `OPENAI_API_KEY` causes this scraper to be silently skipped (not replaced with mock data) — see the no-fabrication rule below.
- **`MockScraper`** remains the dev/demo fallback when `SERPAPI_KEY` is unset, so the full pipeline (job → worker → report) is demoable without real credentials.
- No fabricated data anywhere: every price shown always traces back to a real scrape result from one of the above, never a stand-in value.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| SerpAPI (chosen, Method 2) | Single integration covers multiple OTAs via one API; no per-OTA scraping infrastructure to build/maintain; reasonable free/low-cost tier for a competition demo | Aggregated results, not a first-party OTA integration; can't reach Wink (not indexed by Google Hotels) — see [[ADR-005-provider-specific-scraping]] |
| ChatGPT + Gother API (chosen, Method 1, bonus) | Bonus points from the brief; independent second method for comparison/validation; no scraping infrastructure at all | LLM price knowledge is not guaranteed current or accurate; strict JSON schema + "omit if unsure" instruction mitigates but does not eliminate this |
| Scrapingdog / other scraping-API vendors | Similar aggregation model to SerpAPI | No existing integration or evaluated pricing/coverage advantage over SerpAPI; would not resolve the Wink gap either |
| Direct HTTP scraping per OTA | Full control over exact source attribution, could reach Wink directly | Each OTA needs its own scraper, anti-bot handling, and maintenance — far more engineering time than the Aug 17 deadline allows |

## Consequences
### Positive
- Two independent scraping methods satisfy both the core requirement and the bonus points, sharing one merge/dedup path in the worker.
- No new scraping infrastructure to operate or maintain beyond what's already built.

### Negative / Trade-offs
- Wink remains unreachable via either method — a real per-provider (or direct-contract) integration is future work, not part of this pass.
- SerpAPI/ChatGPT-derived prices are trusted as-is; neither method independently re-verifies the other's numbers, and `method=both` can produce duplicate agoda/trip rows per hotel with no reconciliation logic (flagged as an open risk in REQ-001-v1.3).

## Addendum (2026-08-08): Scraper adapter/registry pattern
Gemini was added as a third method (`backend/src/scraper/gemini_scraper.rs`) after this ADR was first written, using the same `Scraper` trait + no-fabrication pattern as ChatGPT. Adding it required editing an if/else chain in `worker/jobs/scrape_job.rs::scrape_hotel_prices` by hand, and in the process a real bug was found: `ScrapeMethod::Gemini` was checked with `==` instead of `matches!(.., Both)`, so `method=Both` silently never ran Gemini even when configured.

To make future scrapers pluggable without touching the worker, `backend/src/scraper/registry.rs` now defines:
- `ScraperFactory` trait — `methods() -> &[ScrapeMethod]` (which methods this factory participates in) and `build(&Config) -> Option<Box<dyn Scraper>>` (build if configured, `None` to skip — same no-fabrication rule as always).
- `default_registry()` — the list of factories, built once at startup and stored on `AppState` behind an `Arc`.
- The worker loop just iterates the registry, skipping factories whose `methods()` don't include the job's method, and skipping (not faking) any factory whose `build()` returns `None`.

**To add a new scraper going forward:**
1. Implement the existing `Scraper` trait (`fn name()`, `async fn scrape(&self, params: &ScrapeParams) -> anyhow::Result<Vec<ScrapeResult>>`) — unchanged.
2. Add a small `ScraperFactory` wrapper (see `SerpApiFactory`/`ChatGptFactory`/`GeminiFactory` in `registry.rs` for the pattern) declaring which `ScrapeMethod`(s) it serves and how to build itself from `Config` (or `None` if unconfigured).
3. Push it into `default_registry()`.
4. If it needs its own selectable method (not just piggybacking on `Both`), add a new `ScrapeMethod` enum variant + a migration (`ALTER TYPE scrape_method ADD VALUE '...'`, see migration `011` for the Gemini precedent).

No changes to `worker/jobs/scrape_job.rs`, `AppState`, or any handler are needed for a new scraper that only adds a factory — the loop is already generic over the registry.

## Related
- REQ: [[REQ-001-v1.3]]
- ADR: [[ADR-005-provider-specific-scraping]]
