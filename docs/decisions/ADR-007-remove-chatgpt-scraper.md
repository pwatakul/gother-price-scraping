---
title: "ADR-007: Remove the ChatGPT Scraper"
type: decision
date: 2026-08-11
status: Accepted
tags: [adr, scraping, architecture, simplification]
related: ["[[ADR-001-scraper-choice]]", "[[ADR-005-provider-specific-scraping]]", "[[REQ-001-v1.3]]"]
---

# ADR-007: Remove the ChatGPT Scraper

## Context
[[ADR-001-scraper-choice]] adopted two scraping methods: SerpAPI (primary, live Google Hotels data) and a `ChatGptScraper` (bonus "Method 1"), later joined by a Gemini scraper built on the same pattern. That left three price sources plus the Gother internal API.

Operating the system exposed what the LLM-based methods actually deliver. Running the scraper's exact prompt against Gemini for a real, well-known hotel returns:

```
finishReason: STOP
{"rates": []}
```

That is the design working correctly — the prompt instructs the model to omit any rate it isn't confident about rather than guess, per the no-fabrication rule. But it means an LLM scraper structurally cannot supply live inventory pricing: it has no access to Agoda/Trip rates for a specific date, so the honest answer is almost always "no rates". ChatGPT is the same architecture against the same wall, and it was never configured in any environment (`OPENAI_API_KEY` empty; 0 `scrape_jobs` and 0 `scrape_results` rows ever used `chatgpt`).

Keeping two LLM scrapers meant carrying two API integrations, two sets of credentials, and two user-facing menu options for one capability that yields nothing — while presenting them in the UI as equal peers to SerpAPI.

## Decision
**Remove the ChatGPT scraper entirely.** Price sources are **SerpAPI** (live rates) and **Gemini** (AI knowledge-based, retained as the single second opinion), plus the Gother internal API for Gother's own rates.

Scope of the removal:
- `backend/src/scraper/chatgpt.rs` and `ChatGptFactory` deleted; `default_registry()` drops to three factories.
- `openai_api_key` / `openai_model` removed from `Config`.
- `ScrapeMethod::Chatgpt` removed from the Rust enum, and `'chatgpt'` removed from the Postgres `scrape_method` enum (migration `020`).
- ChatGPT removed from both method pickers in the frontend; `ScrapeMethod` narrowed to `'serpapi' | 'gemini' | 'both'`. `both` now means **SerpAPI + Gemini**.

**Gother is deliberately retained** even though `GOTHER_API_URL`/`GOTHER_API_KEY` are currently empty and the factory therefore skips itself. Gother's own rates are the reference the win-rate, market-position and parity-violation analytics compare OTA prices against — the product's core question is "is Gother winning?". The integration stays in place, dormant, for when API access is available.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: Remove ChatGPT, keep Gemini (chosen)** | Removes an unused, unconfigured integration and its credentials; one LLM path is enough to keep the "second method" idea alive if model capabilities change; smallest honest menu | Loses the bonus-points framing from the original brief; Gemini still returns empty in practice |
| B: Remove both ChatGPT and Gemini | Simplest possible system — one real source | Discards the only non-SerpAPI method; if a future model gains live retrieval, the whole integration would have to be rebuilt |
| C: Keep both, hide them in the UI | No code deleted, trivially reversible | The exact "dead surface" problem already corrected once for the scheduled-config columns — schema and menus advertising capabilities nothing delivers |
| D: Keep ChatGPT and configure it | Restores the bonus method | Same structural limitation as Gemini; adds a paid integration to obtain the same empty result |

## Consequences

### Positive
- One fewer external API, credential and failure mode to operate.
- The method picker now reflects what the system can actually do: SerpAPI is marked recommended (the only live source), Gemini is described as "AI knowledge-based; declines if unsure".
- `scrape_method` in both Rust and Postgres lists only methods that are implemented.

### Negative / Trade-offs
- **Destructive enum change.** Postgres has no `DROP VALUE`, so migration `020` recreates `scrape_method` and re-points `scrape_jobs.method`, `scrape_results.via_method` and `scheduled_scrape_configs.method` at it, dropping and restoring each column default around the swap. Verified against a `pg_dump` copy of the live database: 5 jobs and 315 results survived unchanged. Safe only because no row referenced `'chatgpt'` — re-adding it later means another migration.
- **SerpAPI is now the only source of live prices.** With `SERPAPI_KEY` unset the system has no working real source at all. At the time this ADR was written, `method=serpapi` silently fell back to `MockScraper` — fabricated data presented as success. That trap has since been closed: mock is opt-in only and a missing key now fails loudly, per [[ADR-008-no-silent-mock-fallback]].
- The original brief's two-method bonus framing no longer matches the implementation.

## Related
- [[ADR-001-scraper-choice]] — the decision this partially supersedes; its "Method 1 (bonus): ChatGptScraper" no longer holds
- [[ADR-005-provider-specific-scraping]] — the no-fabrication rule that makes LLM scrapers return empty rather than guess
- [[REQ-001-v1.3]] — scraping requirements, including the `method=both` duplicate-row risk

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-11 | Initial — accepted after observing that LLM scrapers structurally cannot return live rates, and that ChatGPT was never configured or used |
