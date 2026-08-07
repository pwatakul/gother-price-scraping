-- Migration: Add 'gemini' to the scrape_method enum
-- Description: Gemini becomes a real price-scraping method (alongside
-- serpapi/chatgpt), not just an optional room-type normalizer.

ALTER TYPE scrape_method ADD VALUE 'gemini';
