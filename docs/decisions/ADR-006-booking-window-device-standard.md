---
title: "ADR-006: Booking-Window × Device Grid as a System Constant"
type: decision
date: 2026-08-10
status: Accepted
tags: [adr, scheduling, automation, booking-window, device, analytics]
related: ["[[REQ-008-v1.0]]", "[[REQ-002-v1.1]]", "[[REQ-001-v1.3]]", "[[ADR-002-price-history-schema]]"]
---

# ADR-006: Booking-Window × Device Grid as a System Constant

## Context
[[REQ-002-v1.1]] gave each `scheduled_scrape_configs` row a free-form `lookahead_days: INTEGER[]`, and the scheduler fires one job per entry. That made scheduling flexible, but flexibility is the wrong property here: if hotel group A tracks `[1, 7]` and group B tracks `[2, 14]`, the two hotels' price series share no common x-axis, and `mv_hotel_booking_window` — which buckets by days-in-advance — ends up with sparse, non-overlapping buckets that cannot be compared or averaged.

[[REQ-008-v1.0]] asks for the opposite property: one uniform dataset that can be analysed directly. It also adds a second dimension (device: desktop vs. mobile_web) that must be present for *every* window, not selectively.

The question is whether the window set and device set should be configuration or code.

## Decision
**The booking-window × device grid is a hardcoded system constant in the scheduler, not per-config configuration.**

```rust
const STANDARD_BOOKING_WINDOWS: [i64; 5] = [1, 3, 7, 14, 30];
const STANDARD_DEVICES: [Device; 2] = [Device::Desktop, Device::MobileWeb];
```

Every scheduler fire expands to the full 10-job cross-product at fixed stay parameters (1 night, 1 room, 2 adults). `lookahead_days` and `los_variants` are no longer read on the scheduled path.

Changing the standard is a code change plus a new ADR — which is the point: the standard changing is a decision about the dataset's meaning, not an operational tweak, and it should leave a record explaining why the series before and after the change are not comparable.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Hardcoded constant (chosen)** | Guarantees every hotel has identical coverage; analytics need no normalization; impossible to misconfigure into an incomparable dataset; the grid is self-documenting in code | Changing the standard needs a deploy; no per-hotel tuning for hotels that would benefit from denser or sparser sampling |
| **B: Keep free-form `lookahead_days`, default it to `[1,3,7,14,30]`** | No schema or scheduler restructure; per-hotel tuning stays possible | A default is not a standard — one edited config silently breaks cross-hotel comparability, and nothing surfaces that it happened. This is the failure mode the requirement exists to prevent |
| **C: Global settings table holding the standard** | Changeable without deploy, still enforced uniformly | Adds a table, a cache and a failure mode (what does the scheduler do if the row is missing?) to store five integers that change roughly never. Also loses the ADR trail — the whole reason to make the change deliberate |
| **D: Per-config `device` flag rather than always firing both** | Halves job volume for hotels where device is known not to matter | Requirement is explicitly "mandatory"; a missing device makes the pair non-comparable for exactly the hotels someone chose to skip, and the saving is not needed at current scale |

## Consequences

### Positive
- Any two hotels can be compared directly at the same booking window and device without normalization or gap-filling.
- `mv_hotel_booking_window` buckets are dense and uniformly populated, so its averages are meaningful.
- Removes an entire class of misconfiguration — there is no way to schedule a partial or skewed grid.
- The scheduler becomes deterministic and unit-testable: given a config, the emitted job set is a pure function.

### Negative / Trade-offs
- **10× job volume per fire.** Previously 1–3 jobs, now always 10. Acceptable at current hotel counts; worth watching against scraper quotas as configs multiply.
- **`mv_hotel_booking_window` must be dropped and recreated** to add `device` to its grouping key and unique index (the unique index is required by the existing `REFRESH … CONCURRENTLY` path). The view is briefly empty mid-migration; data is not lost since it repopulates from `hotel_price_history`.
- **`lookahead_days`, `los_variants`, `rooms` and `adults` were dropped** from `scheduled_scrape_configs` (migration `019`). Keeping them would have left the API requiring, and the UI collecting, values that nothing reads — so the columns went rather than becoming permanently vestigial. A schedule now configures only *when*, *how* and *whether*, never *what*.
- **No per-hotel sampling density.** A hotel that would genuinely benefit from, say, a +60 window cannot get one without changing the standard for everyone.

### Mitigations
- On-demand scrape jobs via the HTTP API keep full free-form parameters, so ad-hoc investigation of any window/device/LOS combination is still possible — it just does not pollute the standardized series.
- A manual "Run now" trigger (`POST /scheduled-scrape-configs/:id/run`) covers the "I need this data now" case without anyone needing to edit the standard. It shares `scheduler::fire_grid` with the cron path, so a manual run and a scheduled run produce identical job sets.

## Note on the device dimension
This ADR standardizes *how* the device dimension is collected. It does not assert that it is meaningful yet: [[REQ-001-v1.3]] flags that neither SerpAPI nor the Gother API exposes a documented device parameter, so until that is resolved the two device series are expected to be identical and `device` is a recorded label rather than an observed difference. Collecting it now means that the day an upstream device parameter becomes available, the schema, the views and the UI already accommodate it and history is not retroactively lost.

## Related
- [[REQ-008-v1.0]] — the requirement this decision implements
- [[REQ-002-v1.1]] — the scheduler and `lookahead_days` design being superseded on the scheduled path
- [[REQ-001-v1.3]] — origin of the device/login-state dimensions and the upstream-support risk
- [[ADR-002-price-history-schema]] — why `hotel_price_history` is the table the device column belongs on

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-10 | Initial — accepted alongside REQ-008 v1.0 |
| 1.1 | 2026-08-11 | Recorded that the superseded config columns were dropped (migration `019`) rather than left vestigial; noted the manual-run trigger as a mitigation |
