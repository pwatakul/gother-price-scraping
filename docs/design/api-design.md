---
title: API Design
type: design
version: "1.1"
updated: 2026-08-04
tags: [design, api, rest]
---

# API Design

## Overview
REST JSON API served by the Rust/Axum backend. All endpoints are prefixed under `/api`.

- Base URL: `http://localhost:8080/api`
- Auth: None (v1.0 — no authentication)
- Format: JSON (request and response bodies)
- CORS: All origins allowed (development configuration)

---

## Endpoints

### Health

#### `GET /health`
**Description:** Service health check.

**Response 200:**
```json
{ "status": "ok" }
```

---

### Hotel Groups

#### `GET /hotel-groups`
**Description:** List all hotel groups with hotel count and last scraped timestamp.

**Response 200:**
```json
[
  {
    "id": "uuid",
    "name": "Bangkok Properties",
    "description": "Hotels in central Bangkok",
    "hotel_count": 12,
    "last_scraped_at": "2026-04-20T10:00:00Z",
    "created_at": "2026-04-01T08:00:00Z"
  }
]
```

---

#### `POST /hotel-groups`
**Description:** Create a new hotel group. Accepts `multipart/form-data`. An Excel file can be included to import hotels immediately.

**Request (multipart/form-data):**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | text | Yes | Group name |
| description | text | No | Optional description |
| file | file | No | Excel (.xlsx) with hotel list |

> [!NOTE]
> Excel columns expected: `hotel_name`, `city`, `country`. Download the template from `GET /templates/hotel-import`.

**Response 201:**
```json
{
  "id": "uuid",
  "name": "Bangkok Properties",
  "description": null,
  "created_at": "2026-04-22T10:00:00Z",
  "updated_at": "2026-04-22T10:00:00Z"
}
```

---

#### `GET /hotel-groups/:id`
**Description:** Get hotel group details including all member hotels with last price info.

**Response 200:**
```json
{
  "group": {
    "id": "uuid",
    "name": "Bangkok Properties",
    "description": null,
    "created_at": "...",
    "updated_at": "..."
  },
  "hotels": [
    {
      "id": "uuid",
      "name": "Mandarin Oriental Bangkok",
      "city": "Bangkok",
      "country": "Thailand",
      "last_price_thb": 12500.0,
      "last_price_source": "agoda",
      "last_scraped_at": "2026-04-20T10:00:00Z"
    }
  ]
}
```

**Response 404:**
```json
{ "error": { "code": "NOT_FOUND", "message": "Hotel group not found" } }
```

---

#### `PUT /hotel-groups/:id`
**Description:** Update a hotel group's name or description.

**Request Body:**
```json
{
  "name": "Updated Name",
  "description": "Updated description"
}
```

**Response 200:** Updated `HotelGroup` object.

---

#### `DELETE /hotel-groups/:id`
**Description:** Delete a hotel group (cascades to members and scrape jobs).

**Response 200:**
```json
{ "success": true }
```

---

#### `POST /hotel-groups/:id/import`
**Description:** Import hotels from an Excel file into an existing group.

**Request (multipart/form-data):**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| file | file | Yes | Excel (.xlsx) with `hotel_name`, `city`, `country` columns |

**Response 200:**
```json
{ "success": true, "imported_count": 8 }
```

---

#### `POST /hotel-groups/:id/import-master`
**Description:** Import hotels from the real master hotel-list format (REQ-001-v1.2 F-021), separate from the plain 3-column import above so neither path risks the other. Hotels are keyed by `HID` (find-or-create) rather than name+city+country.

**Request (multipart/form-data):**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| file | file | Yes | Excel (.xlsx) matching `hotel-list-2200.csv`: `No, HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH` |

**Response 200:**
```json
{ "success": true, "imported_count": 2200 }
```

---

#### `POST /hotel-groups/:id/hotels`
**Description:** Add a single hotel to a group (find-or-create by name + city + country).

**Request Body:**
```json
{
  "name": "Mandarin Oriental Bangkok",
  "city": "Bangkok",
  "country": "Thailand"
}
```

**Response 200:** `Hotel` object.

---

#### `DELETE /hotel-groups/:group_id/hotels/:hotel_id`
**Description:** Remove a hotel from a group (does not delete the hotel record itself).

**Response 200:**
```json
{ "success": true }
```

---

#### `GET /hotel-groups/:id/jobs`
**Description:** List scrape jobs for a group (newest first).

**Query Params:**
| Param | Type | Required | Description |
|-------|------|----------|-------------|
| limit | integer | No | Max results (default 20) |
| offset | integer | No | Pagination offset (default 0) |

**Response 200:** Array of `ScrapeJob` objects.

---

### Hotels

#### `GET /hotels/search`
**Description:** Search hotel records by name (case-insensitive substring match).

**Query Params:**
| Param | Type | Required | Description |
|-------|------|----------|-------------|
| q | string | Yes | Search query |
| limit | integer | No | Max results (default 10) |

**Response 200:**
```json
[
  {
    "id": "uuid",
    "name": "Mandarin Oriental Bangkok",
    "city": "Bangkok",
    "country": "Thailand",
    "normalized_name": "mandarin oriental",
    "created_at": "...",
    "updated_at": "..."
  }
]
```

---

### Scrape Jobs

#### `POST /scrape-jobs`
**Description:** Create and enqueue a new scrape job. Returns immediately with `status: pending`; processing happens asynchronously via the RabbitMQ worker.

**Request Body:**
```json
{
  "hotel_group_id": "uuid",
  "checkin_date": "2026-05-01",
  "checkout_date": "2026-05-03",
  "rooms": 1,
  "adults": 2,
  "force_refresh": false,
  "method": "serpapi",
  "los_variants": [1],
  "device": "desktop",
  "login_state": "public"
}
```
`method`, `los_variants`, `device`, `login_state` are all optional; defaults are `"serpapi"`, `[1]`, `"desktop"`, `"public"` respectively. `method: "chatgpt"` and `"both"` are REQ-001 F-020 (bonus); `"chatgpt"` is silently skipped if `OPENAI_API_KEY` isn't configured, same as `"serpapi"` falling back to the mock scraper when `SERPAPI_KEY` is unset.

**Response 200:**
```json
{
  "id": "uuid",
  "hotel_group_id": "uuid",
  "checkin_date": "2026-05-01",
  "checkout_date": "2026-05-03",
  "rooms": 1,
  "adults": 2,
  "status": "pending",
  "force_refresh": false,
  "method": "serpapi",
  "los_variants": [1],
  "device": "desktop",
  "login_state": "public",
  "created_at": "2026-04-22T10:00:00Z",
  "completed_at": null
}
```

---

#### `POST /scrape-jobs/with-overrides`
**Description:** Same as `POST /scrape-jobs`, but accepts an optional per-hotel search-parameter override sheet (REQ-001 F-002 JobDefaults fallback). Multipart, not JSON.

**Request (multipart/form-data):**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| job | text (JSON) | Yes | Same body shape as `POST /scrape-jobs` |
| overrides | file | No | Excel (.xlsx) keyed by `hid` (or `hotel_name`, currently unresolved — see note below), with optional `checkin_date`/`checkout_date`/`rooms`/`adults`/`currency` columns. Blank cells fall back to the job-level defaults above. |

> [!NOTE]
> Rows keyed only by `hotel_name` (no `hid`) are currently **not** resolved — matching by name across potential duplicates is ambiguous, so only `hid`-keyed override rows take effect. `hid` is the reliable key because it comes from the master hotel-list import.

**Response 200:** Same as `POST /scrape-jobs`.

---

#### `GET /scrape-jobs/:id`
**Description:** Get job status and progress counters. Poll this endpoint to track completion.

**Response 200:**
```json
{
  "id": "uuid",
  "hotel_group_id": "uuid",
  "checkin_date": "2026-05-01",
  "checkout_date": "2026-05-03",
  "rooms": 1,
  "adults": 2,
  "status": "processing",
  "force_refresh": false,
  "created_at": "...",
  "completed_at": null,
  "progress": {
    "total": 12,
    "completed": 7,
    "failed": 1,
    "pending": 4
  }
}
```

---

#### `DELETE /scrape-jobs/:id`
**Description:** Cancel a running scrape job. The worker checks for cancellation between hotel batches.

**Response 200:** Updated `ScrapeJob` object with `status: "cancelled"`.

---

#### `GET /scrape-jobs/:id/results`
**Description:** Get full price comparison results for a completed job.

**Response 200:**
```json
{
  "job": {
    "id": "uuid",
    "checkin_date": "2026-05-01",
    "checkout_date": "2026-05-03",
    "rooms": 1,
    "adults": 2,
    "status": "completed",
    "created_at": "...",
    "completed_at": "..."
  },
  "summary": {
    "total_hotels": 12,
    "successful": 11,
    "failed": 1,
    "avg_best_price": 4850.50
  },
  "results": [
    {
      "hotel": { "id": "uuid", "name": "Mandarin Oriental Bangkok", "city": "Bangkok", "country": "Thailand", "hid": 10234 },
      "status": "success",
      "error_message": null,
      "prices": [
        {
          "source": "agoda",
          "room_type": "Deluxe Room",
          "price_thb": 12500.0,
          "original_price": 12500.0,
          "currency": "THB",
          "meal_plan": "Room Only",
          "cancellation": "Free cancellation",
          "source_url": "https://...",
          "scraped_at": "2026-08-04T10:00:00Z",
          "los_nights": 1,
          "who_id": null,
          "is_direct_contract": false,
          "mismatch_warning": null
        }
      ],
      "best_source": "agoda",
      "best_price": 12500.0,
      "gother_price": 11800.0,
      "price_difference": -700.0,
      "price_diff_percent": -5.6
    }
  ]
}
```

`prices` only ever contains the four named providers (`gother`, `agoda`, `trip`, `wink`) — REQ-001-v1.2 F-022. A provider with no rate for this hotel/date is simply absent from the array (F-027: render blank, never `0`); the frontend renders `—` for missing columns. `wink` only appears for domestic (Thailand) hotels, and currently has no real scraper behind it, so it is always absent until a real Wink/HyperGuest data source is wired in. `who_id` is populated only for `source: "gother"` entries, and only if Gother's upstream API actually returns one (unconfirmed — see REQ-001-v1.3 open risk). `mismatch_warning` is set when a non-Gother entry's room type or cancellation policy differs from Gother's entry for the same hotel.

---

#### `GET /scrape-jobs/:id/export`
**Description:** Download price comparison results as an Excel file.

**Response 200:** Binary `.xlsx` file.
- `Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`
- `Content-Disposition: attachment; filename="hotel-price-report-{id}.xlsx"`

---

### Templates

#### `GET /templates/hotel-import`
**Description:** Download a blank Excel template for importing hotels into a group.

**Response 200:** Binary `.xlsx` file with header row: `hotel_name`, `city`, `country`.
- `Content-Disposition: attachment; filename="hotel-import-template.xlsx"`

---

## Error Response Format
All error responses use this structure:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Hotel group with id ... not found",
    "details": null
  }
}
```

## Error Codes
| HTTP Code | Error Code | Meaning |
|-----------|------------|---------|
| 400 | `VALIDATION_ERROR` | Invalid request parameters or body |
| 400 | `EXCEL_ERROR` | Excel file parse failure |
| 404 | `NOT_FOUND` | Resource not found |
| 502 | `EXTERNAL_API_ERROR` | SerpAPI or Gother API call failed |
| 500 | `DATABASE_ERROR` | PostgreSQL error |
| 500 | `CACHE_ERROR` | Redis error |
| 500 | `QUEUE_ERROR` | RabbitMQ publish error |
| 500 | `SCRAPER_ERROR` | All scrapers returned no results |
| 500 | `INTERNAL_ERROR` | Unclassified server error |

## Related
- [[data-model]] — entity shapes behind each response
- [[REQ-001-v1.3]] — functional requirements this API satisfies

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-04-22 | Initial — documented from implemented router and handlers |
| 1.1 | 2026-08-04 | Added `import-master` and `with-overrides` endpoints; `method`/`los_variants`/`device`/`login_state` on scrape jobs; `scraped_at`/`who_id`/`is_direct_contract`/`mismatch_warning`/`price_diff_percent`/`hid` on results — REQ-001-v1.3 (F-002, F-011, F-020–F-027) |
