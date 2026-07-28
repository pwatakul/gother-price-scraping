---
title: UI Wireframes v1
type: design
version: "1.0"
updated: 2026-04-27
status: Draft
tags: [design, ui, wireframes, ux]
related: ["[[REQ-001-v1.1]]", "[[REQ-003-v1.0]]", "[[system-v1]]", "[[class-diagram-v1]]"]
---

# UI Wireframes v1

## Navigation Structure

```mermaid
graph TD
    Root["🏠 App Root"]

    Root --> Dashboard["📊 Dashboard\n/"]
    Root --> GroupDetail["🏨 Hotel Group Detail\n/groups/:id"]
    Root --> JobProgress["⏳ Job Progress\n/jobs/:id/progress"]
    Root --> Report["📋 Price Comparison Report\n/jobs/:id/results"]
    Root --> Analytics["📈 Analytics Dashboard\n/analytics\n(Phase 2 — REQ-003)"]

    Dashboard -->|"click group card"| GroupDetail
    Dashboard -->|"click new job"| JobProgress
    GroupDetail -->|"create job"| JobProgress
    JobProgress -->|"job complete"| Report
    Report -->|"export"| Excel["📥 Excel Download"]
    Report -->|"back"| Dashboard
```

---

## User Journey — Competition Demo Flow

```mermaid
flowchart TD
    Start(["👤 User opens app"])

    Start --> D1["Dashboard\nSee hotel groups list"]
    D1 -->|"Create new group"| NG["Modal: New Hotel Group\nName + Description"]
    NG -->|"Upload Excel"| UX["Excel Upload\nhotel_name, city, country,\ncheckin, checkout, rooms, adults"]
    UX -->|"confirm"| GD["Hotel Group Detail\nSee imported hotels list"]

    GD -->|"Start price search"| JF["New Job Modal\nConfirm: dates, method,\nSerpAPI / ChatGPT / Both"]
    JF -->|"submit"| JP["Job Progress Screen\nPer-hotel status\nreal-time polling"]
    JP -->|"job complete"| Rep["Price Comparison Report\nAll hotels × all sources\n+ evidence (URL, timestamp)"]
    Rep -->|"Download"| DL["📥 hotel-price-report.xlsx"]

    D1 -->|"open existing group"| GD
    GD -->|"view past job"| Rep
```

---

## Screen 1: Dashboard

```mermaid
graph LR
    subgraph Dashboard["📊 Dashboard — /"]
        direction TB
        NAV["─── Nav Bar: Gother Price Intelligence · [Analytics] ───"]
        STATS["┌────────────┐ ┌──────────────┐ ┌────────────────┐
│ Total Groups│ │ Total Hotels │ │  Last Scraped  │
│     12      │ │     248      │ │  2 hours ago   │
└────────────┘ └──────────────┘ └────────────────┘"]
        BTN["[+ New Hotel Group]"]
        CARDS["┌─────────────────────────────────────┐
│ 🏨 Bangkok City Hotels              │
│ 24 hotels  ·  Last scraped: today   │
│ [View]  [New Price Search]          │
├─────────────────────────────────────┤
│ 🏖️ Phuket Beach Resorts            │
│ 18 hotels  ·  Last scraped: 2d ago  │
│ [View]  [New Price Search]          │
└─────────────────────────────────────┘"]
        NAV --> STATS --> BTN --> CARDS
    end
```

---

## Screen 2: Hotel Group Detail

```mermaid
graph LR
    subgraph GroupDetail["🏨 Hotel Group Detail — /groups/:id"]
        direction TB
        HDR["← Back    Bangkok City Hotels    [Edit] [Delete]"]
        ACTS["[+ Add Hotel]  [📥 Import Excel]  [🔍 New Price Search]"]
        HTBL["┌──────────────────────────────────────────────────────────┐
│ Hotel Name          City       Last Price (THB)  Source  │
├──────────────────────────────────────────────────────────┤
│ Dusit Thani Bangkok Bangkok    ฿ 4,200           agoda   │
│ Mandarin Oriental   Bangkok    ฿ 12,500          gother  │
│ The Peninsula       Bangkok    —                 —       │
└──────────────────────────────────────────────────────────┘"]
        JOBS["Recent Jobs
┌────────────────────────────────────────────────────┐
│ Job #001  20–21 Jun  1rm/2ad  ✅ Completed  [View] │
│ Job #002  15–16 Jul  2rm/2ad  ⏳ Processing  [View] │
└────────────────────────────────────────────────────┘"]
        HDR --> ACTS --> HTBL --> JOBS
    end
```

---

## Screen 3: New Job Modal

```mermaid
graph LR
    subgraph JobModal["🔍 New Price Search Modal"]
        direction TB
        T["Start New Price Search"]
        GRP["Hotel Group: Bangkok City Hotels (24 hotels)"]
        DATES["Check-in  [📅 2026-06-20]    Check-out  [📅 2026-06-21]"]
        GUESTS["Rooms  [1 ▾]    Adults  [2 ▾]"]
        METHOD["Scraping Method
○ SerpAPI (Google Hotels)  ← Method 2
○ ChatGPT                  ← Method 1 (bonus)  
● Both                     ← Recommended (max sources)"]
        FORCE["☐ Force refresh (skip cache)"]
        BTNS["[Cancel]                    [🚀 Start Search]"]
        T --> GRP --> DATES --> GUESTS --> METHOD --> FORCE --> BTNS
    end
```

---

## Screen 4: Job Progress

```mermaid
graph LR
    subgraph Progress["⏳ Job Progress — /jobs/:id/progress"]
        direction TB
        HDR["← Back    Price Search: 20–21 Jun 2026  1rm / 2ad"]
        STATUS["Status: ⏳ Processing"]
        BAR["Progress: ████████████░░░░░░░░  14 / 24 hotels"]
        STATS2["┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│  Total   │ │  Done ✅  │ │ Failed ❌ │ │Pending ⏳│
│   24     │ │   14      │ │    0      │ │   10     │
└──────────┘ └──────────┘ └──────────┘ └──────────┘"]
        HTBL2["Hotel Status Log
┌─────────────────────────────────────────────────┐
│ ✅ Dusit Thani Bangkok      3 sources found      │
│ ✅ Mandarin Oriental        2 sources found      │
│ ⏳ The Peninsula            searching...         │
│ ❌ Hotel XYZ                not found / error    │
└─────────────────────────────────────────────────┘"]
        CANCEL["[❌ Cancel Job]"]
        HDR --> STATUS --> BAR --> STATS2 --> HTBL2 --> CANCEL
    end
```

---

## Screen 5: Price Comparison Report (KEY SCREEN — Judges will see this)

```mermaid
graph LR
    subgraph Report["📋 Price Comparison Report — /jobs/:id/results"]
        direction TB
        HDR3["← Back   Bangkok City Hotels   20–21 Jun 2026   1rm/2ad   ✅ Completed"]
        SUMMARY["┌─────────────┐ ┌──────────────┐ ┌───────────────┐ ┌────────────────┐
│ 24 Hotels   │ │ 22 Success  │ │ 2 Failed      │ │ Avg Best Price│
│ scraped     │ │     ✅       │ │    ❌         │ │  ฿ 3,840 THB  │
└─────────────┘ └──────────────┘ └───────────────┘ └────────────────┘"]
        ACTIONS["[📥 Export Excel]   [🔄 New Search]   Search: [🔍 ____________]"]
        TABLE["One row per hotel — cheapest price per source shown
Each row expandable: click ▶ to see all room types for that hotel

┌──┬───────────────────┬──────────┬──────────┬──────────┬──────────┬──────┬──────────┬────────┐
│  │ Hotel             │ Gother   │ Agoda    │ Booking  │ Trip.com │ Best │ Gap THB  │ Gap %  │
├──┼───────────────────┼──────────┼──────────┼──────────┼──────────┼──────┼──────────┼────────┤
│▶ │ Dusit Thani BKK   │ ฿ 4,200  │ ฿ 4,500  │ ฿ 4,350  │ ฿ 4,600  │Gother│ 🟢 —    │Cheapest│
│▼ │ Mandarin Oriental │ ฿ 12,500 │ ฿ 11,800 │ ฿ 12,000 │ —        │Agoda │ 🔴 +฿700 │ +5.9% │
│  │   ⚠️ Room types differ across sources                                                    │
│  │   · Gother:  Superior Room    ฿12,500                                                   │
│  │   · Agoda:   Deluxe Room      ฿11,800  ← different type, not apple-to-apple             │
│  │   · Booking: Superior Room    ฿12,000                                                   │
│▶ │ The Peninsula     │ ฿ 18,000 │ ฿ 17,200 │ ฿ 17,500 │ ฿ 16,900 │Trip  │🔴 +฿1,100│ +6.5% │
│▶ │ Hotel XYZ         │ —        │ —        │ —        │ —        │  —   │ ❌ Not found     │
└──┴───────────────────┴──────────┴──────────┴──────────┴──────────┴──────┴──────────┴────────┘"]
        EVIDENCE["Evidence — expand-on-click inside each row (not always visible)
▼ Mandarin Oriental  [click row to expand]
  ┌──────────────────────────────────────────────────────────────────────────┐
  │ Source   Room Type      Price THB  URL                  Scraped at       │
  ├──────────────────────────────────────────────────────────────────────────┤
  │ Gother   Superior Rm    ฿12,500   gother.com/...       2026-04-27 02:14 │
  │ Agoda    Deluxe Rm ⚠️  ฿11,800   agoda.com/...        2026-04-27 02:14 │
  │ Booking  Superior Rm    ฿12,000   booking.com/...      2026-04-27 02:14 │
  └──────────────────────────────────────────────────────────────────────────┘
  ⚠️ = room type differs from Gother — comparison may not be apple-to-apple"]
        HDR3 --> SUMMARY --> ACTIONS --> TABLE --> EVIDENCE
    end
```

---

## Screen 6: Excel Import Modal

```mermaid
graph LR
    subgraph ImportModal["📥 Import Hotels from Excel"]
        direction TB
        T2["Import Hotels from Excel"]
        TMPL["[📄 Download Template] ← hotel_name, city, country, checkin_date,\n                         checkout_date, rooms, adults, currency"]
        DROP["┌─────────────────────────────────────────┐
│                                         │
│   📁 Drop .xlsx file here               │
│      or click to browse                 │
│                                         │
└─────────────────────────────────────────┘"]
        PREVIEW["Preview (first 3 rows)
┌──────────────────┬─────────┬─────────┬────────────┬─────────────┬──────┬────────┐
│ hotel_name       │ city    │ country │ checkin    │ checkout    │rooms │adults  │
├──────────────────┼─────────┼─────────┼────────────┼─────────────┼──────┼────────┤
│ Dusit Thani BKK  │ Bangkok │Thailand │ 2026-06-20 │ 2026-06-21  │  1   │  2     │
│ Mandarin Oriental│ Bangkok │Thailand │ 2026-06-20 │ 2026-06-21  │  1   │  2     │
└──────────────────┴─────────┴─────────┴────────────┴─────────────┴──────┴────────┘
24 rows detected"]
        BTNS2["[Cancel]                              [✅ Import 24 hotels]"]
        T2 --> TMPL --> DROP --> PREVIEW --> BTNS2
    end
```

---

## Screen 7: Analytics Dashboard (Phase 2 — REQ-003)

```mermaid
graph LR
    subgraph Analytics["📈 Analytics Dashboard — /analytics (Phase 2)"]
        direction TB
        HDR4["Analytics  Group: [All Groups ▾]  Period: [Last 30 days ▾]  [📥 Export]"]
        KPIS["┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│  Hotels Tracked  │ │  Gother Win Rate │ │  Avg Price Gap   │
│       248        │ │      64%  🟢     │ │   +฿320 vs OTA   │
└──────────────────┘ └──────────────────┘ └──────────────────┘"]
        CHART["Price Trend Chart (line)
฿20,000 ┤                         Agoda ——
฿15,000 ┤           ╱╲            Booking ----
฿10,000 ┤──────╱────  ──────────  Gother ═══
 ฿5,000 ┤                    
        └─────────────────────────────────
        Apr 01    Apr 15    Apr 27"]
        HEATMAP["Competitor Heatmap
         Agoda   Booking  Trip.com  Gother
Hotel A  🟢       🟢       ——        🟢 cheapest
Hotel B  🔴       🔴       🔴        🔴 +5.9%
Hotel C  🟢       ——       🟢        🟢 cheapest"]
        HDR4 --> KPIS --> CHART --> HEATMAP
    end
```

---

## Color Coding & States

| State | Color | Meaning |
|-------|-------|---------|
| 🟢 Green | Gother is cheapest | Win |
| 🔴 Red | Gother is more expensive | Lose — action needed |
| ⚪ Grey / — | No data available | No match found |
| ⏳ Yellow | Processing | In progress |
| ❌ Red badge | Error | Scrape failed |

---

## Decisions

> [!NOTE]
> All UI open questions resolved 2026-04-27.

| Decision | Answer | UI Impact |
|----------|--------|-----------|
| Report table rows | ✅ **One row per hotel, cheapest price per source** | Row shows minimum price per OTA column. Click `▶` to expand all room types |
| Apple-to-apple mismatch | ✅ **Show ⚠️ warning badge** | Badge appears on the price cell and inside expanded detail when room types differ across sources |
| Evidence (URL + scraped_at) | ✅ **Expand-on-click** | Click anywhere on a hotel row to expand. Evidence shown inside the expanded panel per source with URL link + timestamp |
| Mobile layout | ✅ **No mobile — desktop only** | Min width: 1280px. No responsive breakpoints needed for v1 |
| Filters persist via URL params | ✅ **Yes — URL params** | `?group=uuid&period=30d&checkin=2026-06-20` etc. so users can bookmark and share filtered views |

## Row Interaction Model

```
Default (collapsed):
▶ Hotel Name │ Gother ฿ │ Agoda ฿ │ Booking ฿ │ Trip ฿ │ Best │ Gap ฿ │ Gap %

Click anywhere on row → expands:
▼ Hotel Name │ Gother ฿ │ Agoda ฿ │ Booking ฿ │ Trip ฿ │ Best │ Gap ฿ │ Gap %
  ┌─────────────────────────────────────────────────────────────┐
  │ All room types per source + URL + scraped_at timestamp      │
  │ ⚠️ badge on any price where room type ≠ Gother room type   │
  └─────────────────────────────────────────────────────────────┘
```

## Warning Badge Rule
⚠️ shown when ANY of the following differ between OTA source X and Gother:
- **Room type** — the normalized room type does not match Gother's room type
- **Meal plan** — e.g., Gother = Breakfast Included, Agoda = Room Only
- **Cancellation policy** — e.g., Gother = Free Cancellation, OTA = Non-Refundable (not apple-to-apple on value)

Tooltip on hover shows which dimension mismatched:
- "Room types differ — comparison may not be apple-to-apple"
- "Meal plan differs — Gother includes breakfast, OTA does not"
- "Cancellation policy differs — Gother is refundable, OTA is non-refundable"

## Color Coding & States

| State | Indicator | Meaning |
|-------|-----------|---------|
| 🟢 Green | Gap = Cheapest | Gother is cheapest |
| 🔴 Red | Gap +฿X (+Y%) | Gother is more expensive |
| ⚠️ Warning | Badge on cell | Room types differ across sources |
| ⚪ — | Dash | No data from this source |
| ❌ | Not found | Scrape failed for this hotel |
| ⏳ | Processing | Job still running |

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-04-27 | Initial wireframes — all 7 screens |
| 1.1 | 2026-04-27 | Closed all open questions; updated report table with expand/collapse, ⚠️ badge, evidence panel; added row interaction model, warning badge rule, URL param filter decision |
