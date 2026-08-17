---
title: "ADR-008: No Silent Mock Fallback"
type: decision
date: 2026-08-11
status: Accepted
tags: [adr, scraping, data-integrity, observability]
related: ["[[ADR-001-scraper-choice]]", "[[ADR-005-provider-specific-scraping]]", "[[ADR-007-remove-chatgpt-scraper]]"]
---

# ADR-008: No Silent Mock Fallback

## Context
[[ADR-001-scraper-choice]] decided that `MockScraper` "remains the dev/demo fallback when `SERPAPI_KEY` is unset, so the full pipeline (job → worker → report) is demoable without real credentials", alongside a no-fabrication rule: "every price shown always traces back to a real scrape result … never a stand-in value."

Those two statements are in direct conflict, and in practice the fallback won. With an empty `SERPAPI_KEY`, `scrape_hotel_prices` substituted `MockScraper` and logged it at INFO only. The observed result: **two jobs reported 52/52 hotels successfully scraped, and 315 fabricated price rows were written to `hotel_price_history`** — the table backing the trend chart, win-rate, parity and booking-window analytics. Nothing in the UI distinguished them from real scrapes; the rows even carried `via_method = 'serpapi'`.

The failure mode is worse than an outage. A missing credential is a five-second fix if it surfaces; here it produced green checkmarks over invented numbers, and the fabricated data was only discovered by reading worker logs while investigating an unrelated complaint.

A second, compounding problem: when scrapers *did* return nothing, the worker raised a single flat `"No results from any source"`. That message cannot distinguish "no API key" from "the provider returned no rows" from "the LLM declined to guess" — and it directly caused time to be spent hunting a Gemini bug that did not exist (Gemini was correctly declining, per [[ADR-005-provider-specific-scraping]]).

## Decision
**A missing credential never resolves to fabricated data.** Specifically:

1. **The implicit fallback is deleted.** `scrape_hotel_prices` no longer special-cases a missing `SERPAPI_KEY`.
2. **`MockScraper` becomes a normal registry entry** (`MockFactory`) gated on an explicit `ENABLE_MOCK_SCRAPER` env flag, default **false**. Unlike every other factory, "configured" means an operator deliberately asked for fake data, not that a credential happens to exist. When it builds, it logs a WARN naming itself.
3. **Failures name the responsible source.** Each factory gains `ScraperFactory::name()`; the worker records one `Outcome` per scraper (`Ok(n)` / `Empty` / `NotConfigured` / `Failed(err)`) and formats them via a pure `summarize_outcomes()`. A failed hotel now reads e.g. `No prices found — serpapi: not configured; gemini: returned no rates; gother: not configured` instead of the flat string.
4. **Startup warns when there is no live price source.** If SerpAPI is unconfigured and mock is off, `main.rs` emits a WARN. Previously startup was all green checkmarks even when the system could not scrape at all.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Fail loudly, mock opt-in (chosen)** | Fabricated data cannot reach the database by accident; failures are self-explanatory; the demo path still exists for anyone who deliberately wants it | A demo with no API key now shows failures instead of pretty numbers — which is the honest state, but has to be understood before a live demo |
| B: Keep the fallback, tag mock rows and badge them in the UI | Demo keeps working with zero configuration | Requires a schema column, UI work on every surface, and *still* writes fabricated rows into the analytics tables — one missed badge and the problem returns |
| C: Delete `MockScraper` entirely | Simplest; no fake-data path at all | Removes the ability to exercise the job → worker → report pipeline without a paid key, which is genuinely useful for development and for testing the worker itself |
| D: Leave as-is | No work | The failure mode already caused fabricated data in the live database; leaving it guarantees a recurrence |

## Consequences

### Positive
- Fabricated prices cannot enter `hotel_price_history` without an operator explicitly setting `ENABLE_MOCK_SCRAPER`.
- A missing key is now visible at three points — startup WARN, per-hotel error message, job failure — instead of zero.
- `summarize_outcomes` is pure and unit-tested, so the wording of failures is covered by tests rather than discovered in production (precedent: `is_due`, `standard_grid`, `partition_ranges`).

### Negative / Trade-offs
- **Existing fabricated rows are not retroactively identifiable.** The 315 rows already written are indistinguishable from real ones, so they were removed wholesale by `scripts/seed-thailand-demo.sql` rather than filtered. Any future audit of data written before 2026-08-11 should assume it was mock.
- With no key configured, the app now demonstrably does nothing — correct, but a worse first-run experience than before for anyone who hasn't read the README.
- One more environment variable to document.

## Related
- [[ADR-001-scraper-choice]] — its "MockScraper remains the dev/demo fallback" decision is superseded by this one; its no-fabrication rule is what this ADR finally enforces
- [[ADR-005-provider-specific-scraping]] — the same no-fabrication principle applied to provider filtering
- [[ADR-007-remove-chatgpt-scraper]] — noted this trap as an unaddressed risk; addressed here

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-11 | Initial — accepted after fabricated mock data was found in `hotel_price_history` and reported to the user as successful scrapes |
