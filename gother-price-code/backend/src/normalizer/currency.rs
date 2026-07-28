//! Currency Normalizer
//!
//! Converts prices to THB.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Approximate exchange rates to THB (updated periodically)
static EXCHANGE_RATES: LazyLock<HashMap<&'static str, f64>> = LazyLock::new(|| {
    let mut rates = HashMap::new();
    rates.insert("THB", 1.0);
    rates.insert("USD", 35.5);
    rates.insert("EUR", 38.5);
    rates.insert("GBP", 45.0);
    rates.insert("JPY", 0.24);
    rates.insert("CNY", 4.9);
    rates.insert("SGD", 26.5);
    rates.insert("MYR", 7.5);
    rates.insert("IDR", 0.0023);
    rates.insert("VND", 0.0014);
    rates.insert("PHP", 0.63);
    rates.insert("INR", 0.43);
    rates.insert("KRW", 0.027);
    rates.insert("HKD", 4.55);
    rates.insert("TWD", 1.1);
    rates.insert("AUD", 23.5);
    rates.insert("NZD", 21.5);
    rates.insert("CHF", 40.0);
    rates.insert("AED", 9.7);
    rates
});

/// Convert price to THB
pub fn convert_to_thb(price: f64, currency: &str) -> f64 {
    let currency_upper = currency.to_uppercase();

    if currency_upper == "THB" {
        return price;
    }

    let rate = EXCHANGE_RATES
        .get(currency_upper.as_str())
        .copied()
        .unwrap_or(1.0);

    (price * rate * 100.0).round() / 100.0
}

/// Get exchange rate for a currency
pub fn get_exchange_rate(currency: &str) -> f64 {
    let currency_upper = currency.to_uppercase();
    EXCHANGE_RATES
        .get(currency_upper.as_str())
        .copied()
        .unwrap_or(1.0)
}

/// Format price in THB
pub fn format_thb(price: f64) -> String {
    let s = format!("{:.2}", price);
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let frac_part = parts[1];

    let mut formatted_int = String::new();
    let mut count = 0;

    for c in int_part.chars().rev() {
        if count == 3 && c != '-' {
            formatted_int.push(',');
            count = 0;
        }
        formatted_int.push(c);
        if c != '-' {
            count += 1;
        }
    }

    let final_int: String = formatted_int.chars().rev().collect();
    format!("฿{}.{}", final_int, frac_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_usd() {
        let thb = convert_to_thb(100.0, "USD");
        assert!((thb - 3550.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_thb() {
        let thb = convert_to_thb(1000.0, "THB");
        assert!((thb - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_format_thb() {
        assert_eq!(format_thb(4500.0), "฿4,500.00");
        assert_eq!(format_thb(12500.50), "฿12,500.50");
    }
}
