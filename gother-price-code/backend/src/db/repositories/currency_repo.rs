//! Currency Exchange Rate Repository
//!
//! Backs `hotel_price_history.exchange_rate_id`. Rate values come from the
//! existing static table in `normalizer::currency` (tagged `source =
//! "static"`) — no live rate-provider integration, per ADR/plan scope.

use crate::error::AppResult;
use crate::normalizer::currency::get_exchange_rate;
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct CurrencyRepo;

impl CurrencyRepo {
    /// Look up today's (from, to) rate, inserting one from the static
    /// rate table if none exists yet for today. `to_currency` is always
    /// "THB" at present (the only conversion target in this system).
    pub async fn get_or_create_rate(
        pool: &PgPool,
        from_currency: &str,
        to_currency: &str,
    ) -> AppResult<Uuid> {
        let today = Utc::now().date_naive();
        let from_upper = from_currency.to_uppercase();
        let to_upper = to_currency.to_uppercase();

        if let Some(row) = sqlx::query(
            r#"
            SELECT id FROM currency_exchange_rates
            WHERE from_currency = $1 AND to_currency = $2 AND rate_date = $3
            "#,
        )
        .bind(&from_upper)
        .bind(&to_upper)
        .bind(today)
        .fetch_optional(pool)
        .await?
        {
            return Ok(row.get("id"));
        }

        let rate = get_exchange_rate(&from_upper);

        let row = sqlx::query(
            r#"
            INSERT INTO currency_exchange_rates (from_currency, to_currency, rate, rate_date, source)
            VALUES ($1, $2, $3, $4, 'static')
            ON CONFLICT (from_currency, to_currency, rate_date) DO UPDATE SET from_currency = EXCLUDED.from_currency
            RETURNING id
            "#,
        )
        .bind(&from_upper)
        .bind(&to_upper)
        .bind(rate)
        .bind(today)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }
}
