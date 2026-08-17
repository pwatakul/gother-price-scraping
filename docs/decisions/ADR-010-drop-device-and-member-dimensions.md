---
title: "ADR-010: Scrape Desktop/Public Only — Stop Varying Device and Login State"
type: decision
date: 2026-08-16
status: Accepted
tags: [adr, scraping, data-integrity, cost]
related: ["[[REQ-008-v1.1]]", "[[REQ-001-v1.4]]", "[[ADR-005-provider-specific-scraping]]", "[[ADR-006-booking-window-device-standard]]"]
---

# ADR-010: Scrape Desktop/Public Only — Stop Varying Device and Login State

## Context
[[ADR-006-booking-window-device-standard]] made a 5-window × 2-device grid mandatory so that mobile and desktop prices would be directly comparable. It carried an explicit caveat, inherited from REQ-001-v1.3: *"SerpAPI and the Gother API have no documented parameters for varying results by device or login state … the dimension is recorded as metadata only, not proven to actually change scrape behavior."*

That caveat has now been tested, and it resolves against the dimension.

**Device — no API support.** SerpAPI's `google_hotels` engine does not document a `device` parameter, and one supplied anyway is silently ignored: the value is never echoed back in `search_parameters`.

**Device — no observable difference.** Same hotels, same dates, `device=desktop` vs `device=mobile`:

| Hotel | Desktop | Mobile | Common sources | Differing |
|---|---|---|---|---|
| Anantara Riverside Bangkok Resort | ฿6,387 | ฿6,387 | 25 | 0 |
| InterContinental Pattaya Resort | ฿5,036 | ฿5,036 | 18 | 0 |
| Rayavadee Krabi | ฿15,500 | ฿15,500 | 26 | 0 |

**69 common sources, zero price differences.**

**Login state — not reachable at all.** The API documents no parameter for logged-in status, membership tier or member rates, and we have no credentials for the target sites. Member pricing cannot be observed by any means currently available.

So the grid was writing two identical copies of every row, and the hotel page's "mobile vs desktop" comparison could only ever display a zero difference — presenting a measured finding where none exists. That is the same failure mode [[ADR-005-provider-specific-scraping]]'s no-fabrication rule exists to prevent: not an invented number, but an invented *comparison*.

It was also expensive. The grid is 5 windows × 2 devices × 20 hotels = **200 SerpAPI searches per scheduler fire**, against a free tier of roughly 250 per month.

## Decision
**Scrape desktop/public only. Keep the columns; stop varying the axes.**

1. `worker/scheduler.rs` drops `STANDARD_DEVICES`; `standard_grid()` returns the 5 booking windows and `fire_grid` sets `STANDARD_DEVICE = Desktop` / `STANDARD_LOGIN_STATE = Public`. Jobs per fire: 10 → **5**.
2. The Device and Login State pickers are removed from the scrape form, and the Device columns from the hotel detail page's coverage and price-history tables. The form still sends `desktop`/`public` explicitly, so the API contract is unchanged.
3. **The database columns stay.** `device` and `login_state` on `scrape_jobs`, `scrape_results` and `hotel_price_history` continue to record what a row was captured under, and `mv_hotel_booking_window` keeps `device` in its grouping key (degenerating harmlessly to one value).

The columns are kept deliberately. Unlike the scheduled-config columns dropped in migration 019 — which no code read and which the UI invited users to set pointlessly — these are *written on every row and are accurate*. A future direct-OTA scraper or the Gother API could genuinely vary by device, and keeping the columns means that data lands in a schema that already accommodates it, with history intact.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Keep columns, stop varying (chosen)** | Halves cost immediately; removes a comparison that can only read zero; schema stays ready if a device-sensitive source appears | The columns are single-valued for now, which looks redundant to a reader who hasn't seen this ADR |
| B: Full removal (migration dropping the columns and enums) | Zero dead surface, consistent with how the scheduled-config columns were handled | Discards accurate per-row provenance and a dimension REQ-001 F-023/F-024 defined; re-adding means another migration plus redoing the requirement work — for a dimension we expect to want eventually |
| C: Leave everything, just stop scheduling both | No code change beyond the scheduler | Form keeps offering pickers that cannot affect results — the misleading-control problem already corrected for "Lookahead days" |
| D: Keep the 2-device grid | No work | Pays double for duplicate rows and publishes a fabricated comparison |

## Consequences

### Positive
- **Scheduler cost halves** — 100 SerpAPI searches per fire instead of 200.
- The hotel page no longer implies a mobile/desktop finding that does not exist.
- Per-row provenance is preserved; nothing about existing history is invalidated.

### Negative / Trade-offs
- **`device` and `login_state` are now single-valued in practice.** Anyone reading the schema cold will wonder why they exist — hence this ADR, and the note in [[REQ-001-v1.4]] that they are *recorded, not varied*.
- **Member/private pricing is out of scope indefinitely**, not merely unimplemented. Any competitive analysis assuming member rates are covered would be wrong.
- **This conclusion is source-specific.** It says SerpAPI's Google Hotels aggregation does not vary by device — not that OTAs never do. Scraping Agoda or Trip.com directly could show app/mobile-only rates, which is exactly the kind of difference the original requirement was after. If that becomes a goal, it needs a real per-OTA scraper, and this ADR should be revisited rather than cited as proof that device never matters.

### Still true
Even at 100 searches per fire, daily scheduling is ~3,000/month — beyond the free tier, requiring SerpAPI's paid Developer plan. Scrape cadence should be a deliberate decision, not a default.

## Related
- [[ADR-006-booking-window-device-standard]] — established the 2-device grid this supersedes; its booking-window standard is unchanged
- [[REQ-008-v1.1]] — the requirement revised by this decision
- [[REQ-001-v1.4]] — F-023/F-024 device/login-state dimensions, now "recorded, not varied"
- [[ADR-005-provider-specific-scraping]] — the no-fabrication principle applied here to a comparison rather than a price

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-16 | Initial — accepted after measuring zero desktop/mobile price difference across 69 sources and confirming no API support for either dimension |
