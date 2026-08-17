---
title: "ADR-012: Per-Group Saved Search Config, Stored as an Offset"
type: decision
date: 2026-08-16
status: Accepted
tags: [adr, scraping, ux, scheduling]
related: ["[[ADR-006-booking-window-device-standard]]", "[[ADR-011-serpapi-primary-gemini-fallback]]", "[[REQ-002-v1.2]]", "[[REQ-008-v1.1]]"]
---

# ADR-012: Per-Group Saved Search Config, Stored as an Offset

## Context
A price search was configured from scratch on every run. `ScrapeJobForm` asked for check-in, check-out, rooms, adults, LOS and method each time, and nothing was persisted — `hotel_groups` held only `name` and `description`. Re-running the same search next week meant re-entering the same values from memory and hoping they matched.

The requirement: a group carries its own search settings, one button edits them, another runs them, and scheduled runs use the same settings.

Two design questions had non-obvious answers.

**How to store the dates.** A saved config holding literal calendar dates goes stale silently: a search configured today for "25 August" still says 25 August next month, quietly querying a past date and failing. Nothing surfaces that — it just stops producing data.

**How far the config should reach into scheduled runs.** [[ADR-006-booking-window-device-standard]] made the scheduled grid a system constant (+1/+3/+7/+14/+30 at 1 night / 1 room / 2 adults) precisely so every hotel's history shares one x-axis. If a per-group config could change those, two groups configured differently would produce series that cannot be compared, and editing a config would retroactively change what an existing series means.

## Decision

1. **Dates are a days-ahead offset.** `hotel_groups.search_days_ahead` stores an integer; check-in is computed as `today + days_ahead` at run time, check-out as check-in plus the longest requested LOS. A saved search stays valid indefinitely. The arithmetic lives in a pure `search_window()` in `models/hotel_group.rs`, unit-tested for the single-night case, multi-LOS, zero offset, month/year boundaries and degenerate input.

2. **The config governs manual runs fully; scheduled runs take only the method.** `fire_grid` reads `group.search_method` and keeps `STANDARD_*` for windows, LOS, rooms and adults. So "which scraper" is configurable in one place, while "what shape of observation" stays constant — ADR-006 survives intact.

3. **One config per group.** The settings live on `hotel_groups`. `scheduled_scrape_configs.method` is **dropped** (migration `022`): with the group as the source of truth, a per-schedule method would be a second place to set the same thing, free to diverge silently. That is the dead-config problem migration 019 already had to correct once.

4. **Two endpoints, deliberately separate.** `PUT /hotel-groups/:id/search-config` is distinct from the existing `PUT /hotel-groups/:id`, so renaming a group cannot clobber its search settings or vice versa. `POST /hotel-groups/:id/search-runs` resolves the offset and reuses `create_and_publish_job` — the same path the on-demand API and the scheduler use, so all three produce identical job shapes.

5. **A manual run forces a cache refresh.** Pressing the button means "get me current prices", so `force_refresh: true`. The scheduler leaves it false — it is happy with cached rows.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Days-ahead offset (chosen)** | One-click run stays valid forever; mirrors how booking windows already work | "7 days ahead" is a less direct way to express "the 25th" if that is genuinely what someone wants |
| B: Fixed calendar dates | Explicit and obvious in the UI | Goes stale silently — the failure appears weeks later as unexplained empty results |
| C: Prompt for dates at run time | Never stale | Not the one-click run that was asked for; reintroduces the re-entry problem the config exists to solve |
| D: Config also drives scheduled stay params | Maximum flexibility per group | Breaks cross-group comparability and makes existing history change meaning when a config is edited — reverses ADR-006 for little gain |

## Consequences

### Positive
- A search is configured once and re-run in one click; the settings are visible as a summary line without opening the dialog.
- Method is set in exactly one place for both manual and scheduled runs.
- Verified live: saving `gemini / 21 days / 2 rooms / 3 adults / LOS [1,3]` produced a manual job with check-in = today + 21 and check-out = check-in + 3, while a scheduled "Run now" used **gemini** with the standard 1 room / 2 adults / LOS 1 across +1/+3/+7/+14/+30 — exactly the intended split.

### Negative / Trade-offs
- **Editing the config changes what future manual-search history means.** Rooms/adults/LOS are recorded per row so past data stays interpretable, but a group whose config changes mid-series will show a step change in the manual-search trend. The scheduled series is unaffected — that is the point of item 2.
- **A one-click run makes ad-hoc scraping easier to trigger repeatedly.** At 20 hotels that is 20 SerpAPI searches per press, against a ~250/month free tier.
- **`search_days_ahead` and the booking-window constants are two separate notions of "days ahead"** that could confuse a reader. They are deliberately independent: one is user-configurable and drives manual runs, the other is fixed and drives the comparable series.
- Schedules can no longer differ from their group's method — intentional, but it removes a capability that existed (unused) before.

## Related
- [[ADR-006-booking-window-device-standard]] — the standard grid this decision deliberately does not touch
- [[ADR-011-serpapi-primary-gemini-fallback]] — what `method=both` means for a configured run
- [[REQ-002-v1.2]] — the requirement this implements

## Addendum (2026-08-17): the offset became a set, and LOS became a constant

Item 1 above stored a single days-ahead offset alongside a list of stay lengths. That was the wrong way round in practice: what's wanted is **several check-in dates at one night each**, not one date at several lengths.

Two changes follow, in migrations `024`/`025`:

- **`search_days_ahead` is now `INTEGER[]`**, selected from the scheduler's standard **1/3/7/14/30**. Restricting to that set is deliberate: [[ADR-013-booking-window-comparison-basis]] established that a comparison only means something within one booking window, and manual runs on arbitrary offsets (the live data had +30/+35) created series that could never be compared against scheduled data. Picking from the standard set means a manual run reinforces the same windows the scheduler produces.
- **`search_los_variants` is dropped**; one night is a constant shared with the scheduler. A booking window only measures something if the stay is identical across windows.

A run therefore queues **one job per window**, through a `queue_window_jobs` helper now shared with `fire_grid` — the two were already doing the same thing from separate copies of the loop. Item 2 of the original decision is unchanged: the scheduler still uses the fixed standard grid and takes only the *method* from the group. Verified live — with the group set to `{1,7,30}`, a scheduled run still fired all five standard windows.

**Cost is now proportional to the selection**: windows × hotels searches per press (all five against 20 hotels is 100 SerpAPI searches, against a ~250/month free tier). The settings dialog shows that figure inline, because it is the number that decides whether pressing the button is affordable.

**One implementation note worth remembering:** migration `024`'s `CHECK (array_length(search_days_ahead, 1) >= 1)` did not reject an empty array — `array_length('{}', 1)` returns NULL, and a CHECK evaluating to NULL passes. Migration `025` uses `cardinality()`, which returns 0. An empty selection had already been stored and silently made the saved search a no-op.

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-16 | Initial — accepted alongside the per-group saved search config |
| 1.1 | 2026-08-17 | Addendum: offset became a set drawn from the standard windows; LOS fixed at 1 night; shared job-creation helper; constraint fix |
