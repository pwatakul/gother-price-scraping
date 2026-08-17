---
title: "ADR-013: The Comparison Unit is (Hotel, Check-in Date)"
type: decision
date: 2026-08-16
status: Accepted
tags: [adr, analytics, data-integrity, comparability]
related: ["[[ADR-006-booking-window-device-standard]]", "[[ADR-009-widen-provider-allowlist]]", "[[REQ-003-v1.1]]", "[[REQ-008-v1.1]]"]
---

# ADR-013: The Comparison Unit is (Hotel, Check-in Date)

## Context
Hotel prices depend heavily on how far ahead the stay is booked. Two analytics surfaces ignored that entirely:

**The price trend chart.** `mv_hotel_daily_avg_price` grouped by `(hotel_id, source, day)` — the day the scrape *ran*, with no booking-window dimension. Real data for Anantara Riverside on a single day showed the problem is not just a blurred average:

| Provider | Booking windows present |
|---|---|
| booking, direct | +30 **and** +35 — averaged into one number |
| klook, traveloka | +30 only |
| agoda, expedia, trip, priceline | +35 only |

So for most providers the chart plotted a +30-day quote against a +35-day quote as if they were the same product. The lines were not comparable at all.

**The provider benchmark.** It read `mv_hotel_market_position`, which is the latest row per `(hotel, source)` regardless of check-in date. "Klook is cheapest 58.8% of the time" was partly measuring one provider's +30 price against another's +35 price. It also counted stays only one provider had quoted, where that provider is trivially "cheapest" and the comparison says nothing.

This is the analytics-side counterpart to [[ADR-006-booking-window-device-standard]]: that decision made the *collection* comparable by fixing the booking-window grid. This one makes the *reading* comparable.

## Decision
**A price is only ever compared with another price for the same hotel and the same check-in date.**

1. **`mv_hotel_daily_avg_price` gains `days_in_advance`** in its grouping key and unique index (migration `023`). The trend API takes `?booking_window=`, and `GET /price-history/hotel/:id/trend/windows` reports which windows actually have data.
2. **New `mv_hotel_price_by_stay`** — latest price per `(hotel, source, check-in date)`. That triple is the comparison unit; the provider benchmark is rebuilt on it.
3. **Stays quoted by fewer than two providers are excluded** from the benchmark (`HAVING COUNT(*) >= 2`), and `quotes_compared` is reported so thin coverage is visible next to a high win rate.
4. **The chart has no "all windows" option.** The mixed average is the defect being fixed; leaving it selectable would keep a chart that invites a false conclusion. The selector defaults to the window with the most samples.
5. **Selector options come from the data, not a constant.** Scheduled runs produce the standard +1/+3/+7/+14/+30 grid, but manual runs produce arbitrary offsets — the live data had +30/+35. A hardcoded list would offer options that render blank.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Booking window as an explicit filter (chosen)** | Every series is like for like; the window is stated on the chart so a screenshot is self-describing | One more control, and the chart shows less data at once |
| B: Keep the mixed view, add a window filter alongside | Nothing removed | The default stays wrong, and the misleading chart is the one most people would look at |
| C: Normalise prices across windows with a correction factor | One chart, all data | Would need a model of how price varies with lead time — that model is exactly what this data is meant to *measure*, so it would be circular |
| D: Only ever show the standard grid windows | Simple, fixed axis | Blank charts for hotels scraped manually; hides real data that exists |

## Consequences

### Positive
- Trend lines and benchmark rankings are genuinely comparable.
- The correction was material: median premiums fell from 13–19% to 2–5% for the main OTAs once compared within the same stay, because the old figures were measuring a provider at one window against the cheapest at another. The old numbers overstated the gaps roughly three- to five-fold.
- `quotes_compared` exposes coverage, so a high win rate on few comparisons is visibly weaker evidence.

### Negative / Trade-offs
- **Previously reported benchmark figures are not comparable to the new ones** and should be regenerated wherever they were quoted or screenshotted.
- **Charts show fewer points.** With one window selected, sparse history is obvious — which is honest, but looks emptier than the old mixed chart.
- `mv_hotel_daily_avg_price` has more rows (one per window), and there is a sixth materialized view to refresh after every job.
- **`mv_hotel_market_position` still ignores check-in date** and continues to back Market Position and the heatmap. Those surfaces have the same flaw and were left unchanged in this pass — a known follow-up, not an oversight.
- Bad rows still distort `MIN(price_thb)`: six Four Seasons Koh Samui rows scraped at ฿52–88 remain in the data. The median premium is robust to them; `times_cheapest` is not.

## Related
- [[ADR-006-booking-window-device-standard]] — fixes the collection grid; this fixes how it is read
- [[ADR-009-widen-provider-allowlist]] — the providers being compared
- [[REQ-003-v1.1]] — the analytics requirement this revises

## Addendum (2026-08-17): Market Position and the Heatmap moved too

The Negative section above recorded that `mv_hotel_market_position` still ignored check-in date and continued to back Market Position and the Competitor Heatmap — "a known follow-up, not an oversight". That follow-up is now closed.

Both read `mv_hotel_price_by_stay` and compare providers within **one stay per hotel**: the most recent check-in date that has data. Both return that date, and the UI states it, so a reader can see what is being compared rather than having to trust it.

What forced the change was the winner highlight. Marking the cheapest provider per hotel is meaningless on the old basis — the "winner" would be whichever provider happened to have been scraped for the nearest check-in date, not the cheapest for the same stay. A highlight is a stronger claim than a number in a column, and it made an existing quiet flaw untenable.

`mv_hotel_market_position` itself is now unused by these two surfaces. It is left in place (still refreshed) rather than dropped, since removing a materialized view is a migration with no benefit until something else needs the space.

Verified against raw data: for Shangri-La Bangkok the highlighted cell is Klook at ฿5,673, matching `mv_hotel_price_by_stay` for that stay exactly, and no hotel row mixes check-in dates. Where several providers tie at the identical cheapest price — five at ฿15,707 for The Peninsula — all are highlighted, which is accurate rather than an arbitrary pick.

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-16 | Initial — accepted after finding the trend chart plotted different booking windows against each other |
| 1.1 | 2026-08-17 | Addendum: Market Position and the Competitor Heatmap moved onto the per-stay basis, closing the follow-up named above |
