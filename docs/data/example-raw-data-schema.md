---
title: "Example Raw Scraped Data — Output Schema"
type: design
date: 2026-07-27
status: Reference
tags: [data, schema, raw-brief]
source: "docs/raw/Req price scrapping - 17 July 26.xlsx (sheet: Example RAW DATA)"
related: ["[[REQ-001-v1.2]]", "[[data-model-v1.1]]"]
---

# Example Raw Scraped Data — Output Schema

This documents the **target output schema** shown in the CEO brief's "Example RAW DATA" sheet (~47,353 sample rows). It is a flat, one-row-per-price-point format — not the current `hotel_price_history` schema. See [[REQ-001-v1.2]] for the gap this raises against [[data-model-v1.1]].

A 16-row representative sample (covering all 3 providers and both Notes codes seen in the source) is saved at `docs/data/example-raw-data-sample.csv`. The full 47k-row sheet is not committed — see the raw xlsx in `docs/raw/` if the full dataset is needed.

## Columns

| Column | Example | Notes |
|--------|---------|-------|
| `task_id` | `78010` | Unique ID per scrape task/row |
| `Scrapping Round` | `0`, `1` | Observed values: 0 and 1 in the sample — meaning not defined in the brief; likely a batch/retry counter |
| `hotel_name` | `the ritz carlton osaka` | Lowercase, no slug formatting (differs from `hotel-list-2200.csv`'s `Hotel-Name`) |
| `hotel_country` | `japan` | |
| `hotel_province` | `osaka` | |
| `hotel_city` | `osaka` | |
| `url_gother_th` | `https://www.gother.com/th-th/hotels/japan/osaka/osaka/the-ritz-carlton-osaka` | Full URL (vs. bare-domain `UPDATE URL` in the hotel list sheet) |
| `hotel_id_gother_th` | `6912` | Matches `HID` in `hotel-list-2200.csv` |
| `Scrapping date` | `2026-02-06` | Date the scrape ran |
| `Booking Window` | `90` | Days between scrape date and check-in — one of the brief's configured windows (domestic: 0/1/3/7/14/30; international: 7/14/30/60/90) |
| `check_in` / `check_out` | `2026-05-07` / `2026-05-08` | |
| `num_nights` | `1` | Matches brief's "1 Night" LOS |
| `occupancy_adults` | `2` | Matches brief's "2 Adult" |
| `occupancy_children` | `0` | Not mentioned in brief's scope table — always 0 in sample |
| `num_room` | `1` | |
| `currency` | `THB` | |
| `provider` | `trip`, `agoda`, `gother` | Only 3 of the brief's 4 providers appear in the sample (no `wink` row observed) |
| `room_type` | free text (may be Thai) | ⚠️ Thai-language values in the source sheet appear to have a text-encoding issue (mojibake) — re-export from the original source if this field is needed verbatim |
| `price_final` | `1022.94` | |
| `price_original` | `1204` | |
| `price_final_with_VAT` | `1204` | |
| `price_original_with_VAT` | `1204` | |
| `Notes` | `Room Not Available (Code 1A)`, `Warning: Price Below THB500` | Structured status/warning codes — no equivalent field in `hotel_price_history` today |

## Gap vs. current data model
`hotel_price_history` ([[data-model-v1.1]]) has one `price_thb` + one `original_price` field and a free-text `error_message` on `scrape_hotel_status` — it does not distinguish tax-exclusive vs. tax-inclusive prices, doesn't carry `task_id`/`Scrapping Round`, and has no structured notice-code field for cases like "Room Not Available (Code 1A)". Reconciling this is a data-model decision for a future ADR, not resolved here.
