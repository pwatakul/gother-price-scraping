---
title: "ADR-009: Widen the Provider Allowlist Beyond the Brief's Four Sources"
type: decision
date: 2026-08-16
status: Accepted
tags: [adr, scraping, providers, analytics]
related: ["[[ADR-005-provider-specific-scraping]]", "[[ADR-007-remove-chatgpt-scraper]]", "[[REQ-001-v1.4]]"]
---

# ADR-009: Widen the Provider Allowlist Beyond the Brief's Four Sources

## Context
[[ADR-005-provider-specific-scraping]] narrowed every scraper's output to the four providers the CEO brief names — Gother, Agoda, Trip, Wink — dropping everything else "so comparisons stay apples-to-apples". That was a reasonable reading of the brief, but operating the system showed what it costs.

SerpAPI returns **11–26 real, date-specific prices per hotel**. After filtering, most of that was discarded:

| Hotel | Prices returned | Kept under ADR-005 |
|---|---|---|
| Conrad Bangkok | 20 | 0 |
| Mandarin Oriental Bangkok | 18 | 0 |
| The Peninsula Bangkok | 11 | 0 |

Seven of twenty hotels reported "no rates" while real Booking.com, Expedia, Priceline, Klook and Traveloka prices sat in the same response. A market-intelligence product that cannot say what Booking.com charges is not doing the job the brief actually wants, even if it matches the brief's literal list.

The alternative of sourcing broader OTA data from an LLM was tested and rejected. `gemini-3.6-flash` with Google Search grounding declines outright ("search engines cannot scrape live dynamic checkouts"). `gemini-3.5-flash` answered confidently and wrongly — Agoda, Trip.com and Booking.com all quoted at exactly **6,551 THB**, a number it found in one search and pasted across three OTAs, when the measured Trip.com rate was **฿6,773**. Identical prices across OTAs is the one thing we know is false; the whole point is that they differ.

## Decision
**Widen the allowlist to the major OTAs, and capture the hotel's own rate as a distinct provider.**

Recognized providers become: `gother`, `agoda`, `trip`, `wink`, `booking`, `expedia`, `priceline`, `traveloka`, `klook`, and `direct`.

Supporting changes:

1. **One normalizer.** `normalize_source` moves into `scraper/providers.rs` as the single source of truth; `serpapi.rs` and `gemini_scraper.rs` both call it. Previously each had its own matching logic against shared constants — two places to drift.
2. **Whole-brand-token matching, never substring.** A `contains("trip.com")` test recorded **EaseMyTrip.com** and **Clicktrip.com** as Trip.com, and "Tripening Hotels" and "Etrip.net" would have followed — filing a competitor's rate under a named provider. Matching now compares the de-suffixed brand token exactly.
3. **Still dropped:** obscure resellers and metasearch (Evendo, Zzzello, Reserving, SKYLARK, Bluepillow, hutchgo, Wego, KAYAK, momondo, eDreams, Tripadvisor, Hotelscombined, "müv AI"). The list is "OTAs a traveller would plausibly book through", not "everything Google returns" — otherwise the comparison stops meaning anything.

## Hotel-direct rates
SerpAPI exposes **no `official` or `direct` flag**; the hotel's own rate is identifiable only by its source name resembling the hotel ("Conrad Bangkok", "Mandarinoriental.com", "Anantara.com"). Detection is therefore a heuristic, and deliberately conservative — a false positive files a competitor's price as the hotel's own, the same class of error as the Trip.com bug:

- Both names reduce to lowercase alphanumeric tokens.
- Generic hotel words (`hotel`, `resort`, `spa`, `the`, `by`, `residences`) **and the hotel's own city/country** are removed first — otherwise a source called "Bangkok Hotels" matches half the city.
- A match requires either the collapsed hotel name to start with the source brand, or every distinctive source token to appear in the hotel name.
- Anything ambiguous is dropped rather than guessed.

`direct` is deliberately **not** added to `DIRECT_CONTRACT_PROVIDERS`: REQ-001 F-026's direct-contract concept means a contract *with Gother* (Wink/HyperGuest), which a hotel's public website is not.

## Gother is not reachable via SerpAPI
Recorded so it is not re-attempted. Probing five Thai hotels returned **36 distinct sources** — Booking.com, Priceline, Agoda, Trip.com, Expedia, Traveloka, Klook, Etrip, Tiket, Wego, KAYAK, momondo, eDreams, CheapTickets, Orbitz, Travelocity, Hotels.com, Tripadvisor and others — and **Gother appeared in none of them**. Google Hotels does not list Gother as a booking partner, so no `engine=google_hotels` query can surface it.

Gother rates therefore require Gother's own API. `GotherScraper` already exists for exactly this and needs only `GOTHER_API_URL`/`GOTHER_API_KEY`. gother.com's own frontend calls `goapi.gother.com`, `goapi-bx.gother.com` and `api-gcp.gother.com`, which is the natural starting point when that work happens.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Major OTAs + hotel-direct (chosen)** | Turns 7 empty hotels into full comparisons; parity analysis gains the hotel's own price as a reference | Diverges from the brief's literal four-source list; direct detection is heuristic |
| B: Keep only Agoda + Trip | Strictly faithful to the brief | A third of hotels show nothing, and real competitor prices are discarded on arrival |
| C: Store every source SerpAPI returns | Maximum data | Obscure resellers and metasearch aggregators make "cheapest competitor" meaningless and inflate row counts |
| D: Supplement with LLM-sourced OTA prices | Would cover OTAs Google Hotels misses | Demonstrably fabricates — three OTAs quoted one identical wrong number |

## Consequences

### Positive
- Expect ~8–15 rows per hotel instead of 0–2, across genuinely comparable OTAs.
- Rate-parity analysis becomes meaningful: the hotel's own rate is the natural benchmark.
- One normalizer, one allowlist, tested in one place — including regression tests for every lookalike brand seen in real responses.

### Negative / Trade-offs
- **Win-rate and parity numbers shift.** `mv_hotel_win_rate` scores Gother against `MIN(price)` across all sources; more sources lowers that floor, so Gother's win rate will fall. More honest, but not comparable to previously reported figures. Currently moot — Gother has no data source.
- **Row volume grows ~8×** per hotel. Fine at 20 hotels; worth watching as the booking-window grid runs on a schedule.
- **The direct-rate heuristic can be wrong** in both directions — an unusual hotel/brand pairing may be missed, and a same-named competitor could in principle slip through. It is the one classification in the pipeline that fails silently and plausibly, so it should be spot-checked whenever the hotel list changes materially.
- The allowlist is now a judgement call rather than a quotation from the brief; adding or removing a provider is a decision that belongs in this ADR's change log.

## Related
- [[ADR-005-provider-specific-scraping]] — the narrowing this reverses; its no-fabrication rule is retained and is why unknown sources are still dropped
- [[ADR-007-remove-chatgpt-scraper]] — established that LLM scrapers cannot supply live rates
- [[REQ-001-v1.4]] — F-022, revised for the widened list

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-16 | Initial — accepted after finding 11–26 real prices per hotel being discarded, and confirming the LLM alternative fabricates |
