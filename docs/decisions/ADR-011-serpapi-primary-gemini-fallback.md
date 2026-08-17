---
title: "ADR-011: SerpAPI Primary, Gemini Fallback-Only, with Recorded Provenance"
type: decision
date: 2026-08-16
status: Accepted
tags: [adr, scraping, data-integrity, provenance]
related: ["[[ADR-005-provider-specific-scraping]]", "[[ADR-007-remove-chatgpt-scraper]]", "[[ADR-008-no-silent-mock-fallback]]", "[[ADR-009-widen-provider-allowlist]]", "[[REQ-001-v1.5]]"]
---

# ADR-011: SerpAPI Primary, Gemini Fallback-Only, with Recorded Provenance

## Context
`scrape_hotel_prices` ran every scraper matching the job's method and concatenated the results (`all_results.extend`). Under `method=both` that meant SerpAPI and Gemini rows landed side by side for the same hotel and date, with no precedence and no dedup — the "`method=both` duplication" risk REQ-001 has carried since v1.3.

The two sources are not interchangeable. SerpAPI returns live, date-specific rates read from Google Hotels. Gemini answers from training knowledge, and on this task it is demonstrably unreliable: asked to compare Agoda, Trip.com and Booking.com for one hotel and date, `gemini-3.5-flash` returned **all three at exactly ฿6,551** — a number it had found in a single search and pasted across three OTAs. The measured Trip.com rate that night was **฿6,773**. Identical prices across OTAs is the one thing known to be false, since price differences are the entire point of the comparison. (`gemini-3.6-flash` declines the question outright, which is the better failure.)

Treating those two as equal contributors to the same table is how a fabricated price becomes indistinguishable from a scraped one. `hotel_price_history` had no provenance column at all, so there was no way to tell them apart after the fact — the same situation that forced truncating the 315 mock rows wholesale in [[ADR-008-no-silent-mock-fallback]] rather than filtering them.

## Decision
**Under `method=both`, SerpAPI is authoritative and Gemini only fills total blanks. Every stored price records which scraper produced it.**

1. **Two-tier execution.** `ScraperFactory` gains `fn is_fallback(&self) -> bool` (default `false`); `GeminiFactory` overrides it to `true`. `scrape_hotel_prices` partitions the matching factories and runs the primary tier first; the deferred tier runs only when `should_run_fallback(method, primary_rows)` — a pure, unit-tested function returning `method == Both && primary_rows == 0`.
2. **Explicit choice is never deferred.** A factory is only deferred when the method is `Both`. Running `method=gemini` makes Gemini the primary — the user asked for it directly.
3. **Gother is not a fallback.** It supplies our own rate, not a competitor price, so it stays in the primary tier and is always attempted.
4. **Skips are distinguishable from failures.** A deferred scraper that never ran reports `Outcome::SkippedPrimaryHadPrices`, rendering as `gemini: skipped (primary source had prices)` — not an error.
5. **Provenance is mandatory.** Migration `021` adds `via_method VARCHAR(20) NOT NULL DEFAULT 'serpapi'` to `hotel_price_history`. `ScrapeResult` gains a `via_method` field that **scrapers do not set** — the registry loop stamps it from `factory.name()` at the point of production, so provenance cannot disagree with what actually ran. It is exposed through `/price-history`, the CSV export, and a **Method** column on the hotel detail page where Gemini renders as a warning-styled badge.

`VARCHAR`, not the `scrape_method` enum: this records the actual producer (`serpapi`/`gemini`/`gother`/`mock`) and the enum has no `gother` variant. Note `scrape_results.via_method` stores the job's *requested* method — a pre-existing imprecision deliberately not copied.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Primary/fallback tiers + provenance (chosen)** | A scraped price is never sat next to an AI estimate for the same hotel/date; AI rows are labelled, filterable and auditable | Gemini rows can still enter the dataset — labelled, but present |
| B: Keep concatenating both sources | No work | Duplicate rows per hotel with no precedence; a fabricated price silently competes with a real one in every aggregate |
| C: Per-provider gap filling (ask Gemini only for OTAs SerpAPI missed) | Fills the comparison grid more often | Puts a guessed price directly beside real ones for the same hotel and date — the highest-risk shape, and Gemini's failure mode is exactly inventing per-OTA numbers |
| D: Drop Gemini entirely | No fabrication risk at all | Removes the only fallback for hotels SerpAPI cannot resolve, and discards the precedence architecture that a better future source would slot into |

## Consequences

### Positive
- `method=both` now has a defined precedence instead of arbitrary concatenation; the long-standing duplication risk is resolved.
- Every price row is attributable to the scraper that produced it, retrospectively filterable, and visibly labelled in the UI.
- The precedence mechanism is a registry-level concept, so any future fallback source (a direct-OTA scraper, a second aggregator) slots in by overriding one method.

### Negative / Trade-offs
- **A fabricated price can still be persisted**, where previously `both` would at least have had a real row alongside it. The mitigation is labelling, not prevention. Any `gemini` row should be spot-checked before being trusted.
- **Expect this to fire rarely and usually return nothing.** SerpAPI covered 18 of 20 hotels in the last full run, and Gemini's prompt forbids guessing, so it typically returns `{"rates": []}`. The value here is a correct precedence rule and an audit trail, not data volume.
- **Trigger semantics shift if Gother is connected.** The rule is "the primary tier produced zero rows". With Gother unconfigured that is exactly "SerpAPI returned nothing". Once Gother supplies rows, a Gother-only result with no OTA prices would *not* trigger the fallback, because the primary tier did produce rows. That is arguably wrong — a comparison with no competitors is still a gap — and should be revisited when the Gother API lands.
- One more column on a partitioned, high-volume table.

## Related
- [[ADR-008-no-silent-mock-fallback]] — the precedent: unlabelled fabricated data had to be deleted wholesale because it could not be identified
- [[ADR-007-remove-chatgpt-scraper]] — why LLM scrapers cannot be primary price sources
- [[ADR-009-widen-provider-allowlist]] — the allowlist both tiers normalize onto
- [[REQ-001-v1.5]] — resolves the `method=both` duplication open risk

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-16 | Initial — accepted after Gemini was measured quoting three OTAs at one identical, wrong price, making unlabelled merging unsafe |
