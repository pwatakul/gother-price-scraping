-- Migration: Create currency_exchange_rates table
-- Description: REQ-002/ADR-002 — auditable currency conversion record for
-- hotel_price_history. One row per (from_currency, to_currency, date);
-- rate values are sourced from the existing static table in
-- normalizer::currency (see currency_repo::get_or_create_rate).

CREATE TABLE currency_exchange_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_currency VARCHAR(10) NOT NULL,
    to_currency VARCHAR(10) NOT NULL,
    rate DECIMAL(18, 6) NOT NULL,
    rate_date DATE NOT NULL,
    source VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (from_currency, to_currency, rate_date)
);

CREATE INDEX idx_cer_currencies_date ON currency_exchange_rates (from_currency, to_currency, rate_date DESC);
