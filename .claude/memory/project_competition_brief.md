---
name: Gother Challenge Competition Brief
description: Official CEO competition brief — prizes, deadline, scope, methods, and judging criteria (COMPLETE)
type: project
---

# Gother Challenge: Hotel Price Scraping Tools (Prototype Competition)

## Prizes
- 🥇 1st place: ฿120,000
- 🥈 2nd place: ฿20,000
- 🫶 Consolation: ฿10,000

## Team
- Size: 1–5 people, cross-department allowed
- Registration deadline: April 10, 2026 (closed)
- **Submission deadline: May 15, 2026**
- Project advisor: พี่หนุ่ย — via LINE group

## Official Goal
Build a working prototype for a Hotel Price Scraping / Price Compare tool that:
1. Pulls room prices from multiple OTAs (Agoda, Booking.com, Trip.com, etc.) AND from gother.com
2. Normalizes into a common pattern: hotel, room type, meal plan, cancellation, currency
3. Displays as UI / report / comparison table — easy to compare
4. Includes evidence of source: URL + timestamp of when data was collected

## Methods (choose one or both — both = bonus points)
- ✅ **Method 1**: Prompt via ChatGPT + Gother Hotel Room Price Search API
- ✅ **Method 2**: SerpAPI (e.g., Scrapingdog) + Gother Hotel Room Price Search API

**We have already implemented Method 2 (SerpAPI + Gother API). Implementing Method 1 (ChatGPT) is the path to bonus points.**

## Input Data Requirements (CRITICAL)
The system MUST support batch input via Excel file.

**Excel is the "source of data" — system reads Excel first, then calls API/prompts.**

Required Excel columns (minimum):
| Column | Description |
|--------|-------------|
| hotel_name | Hotel name to search |
| city / country | Location |
| checkin_date | Check-in date |
| checkout_date | Check-out date |
| rooms | Number of rooms |
| adults | Number of adults |
| currency | THB |

> **GAP IDENTIFIED**: Current implementation has checkin_date/checkout_date/rooms/adults set at the JOB level (same for all hotels). The brief requires these fields PER ROW in the Excel — each hotel can have different dates and guest config.

## Required Features
- ✅ Batch input via Excel upload
- ✅ Search prices from OTAs + gother.com
- ✅ Normalize results (room type, meal, cancellation, currency → THB)
- ✅ Display comparison table with evidence (URL + timestamp)
- ✅ Export / Download results as Excel report

## Main Rules
- ✅ Must have working prototype — demo to judges live
- ✅ Must handle rate limiting / retry / basic logging
- ⚠️ Do NOT disturb source websites (respect robots.txt / rate limits)
- ⚠️ Do NOT use real customer data

## Demo Flow (what judges will see)
```
Upload/Read Excel (Batch)
  → System creates search requests (per row)
  → Fetch prices (OTA + gother.com)
  → Normalize (room type, meal, currency)
  → Display comparison table + evidence (URL, scraped_at)
  → Export / Download as Excel
```

## Judging Criteria
🎯 **Primary: Accuracy and correctness of results**
- Prices, currency, conditions must match correctly
- Apple-to-apple comparison (same room type, same conditions)

🎯 **Secondary: Speed of search results**

## Example Prompt (Method 1 — ChatGPT approach)
> "ขอราคาโรงแรมแบบเปรียบเทียบราคาหลายๆ เจ้า เช่น ขอราคาโรงแรม+แสดง room type ของ dusit thani hua hin วันที่ checkin 20 มิถุนายน 2026 checkout 21 มิถุนายน 2026 จากเว็บ agoda.com trip.com booking.com ขอให้ทำตารางเปรียบเทียบ แสดงราคาเป็น thb และใช้ api ของ tripadvisor condition ให้เทียบราคาแบบ apple to apple ถ้ามีข้อมูลราคา official website ให้เอามาแสดงด้วย โดยข้อมูล official website ต้องมาจาก tripadvisor api เท่านั้น"

Note: This example uses TripAdvisor API as a source — worth investigating if TripAdvisor API is accessible.

## Template / Schema
No template was provided by the organizers. The CEO brief is the complete specification. We define the Excel schema and data model ourselves — confirmed 2026-04-26.

## Why: and How to apply:
**Why:** This is the authoritative specification from the CEO. All requirements and implementation decisions must trace back to this brief.

**How to apply:**
- The competition judges on **accuracy + speed** — prioritize getting prices right over adding features
- "Prototype" standard — functional demo beats perfect architecture
- May 15 deadline is hard — scope everything against it
- Both methods = bonus — aim for both if time allows
- The Excel per-row search params is a critical gap vs. current implementation
