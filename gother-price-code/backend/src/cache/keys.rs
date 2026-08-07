//! Cache Keys
//!
//! Defines all cache key patterns used in the application.

use chrono::NaiveDate;
use uuid::Uuid;

/// Cache key builder
pub struct CacheKeys;

impl CacheKeys {
    /// Key for cached hotel price results
    /// Format: price:{hotel_id}:{checkin}:{checkout}:{rooms}:{adults}
    pub fn hotel_price(
        hotel_id: Uuid,
        checkin: NaiveDate,
        checkout: NaiveDate,
        rooms: i32,
        adults: i32,
    ) -> String {
        format!(
            "price:{}:{}:{}:{}:{}",
            hotel_id, checkin, checkout, rooms, adults
        )
    }

    /// Key for cached hotel price results, including the dimensions added
    /// in REQ-001-v1.2 (method/device/login_state/los_nights) so different
    /// configurations don't collide in the cache.
    #[allow(clippy::too_many_arguments)]
    pub fn hotel_price_v2(
        hotel_id: Uuid,
        checkin: NaiveDate,
        checkout: NaiveDate,
        rooms: i32,
        adults: i32,
        method: &str,
        device: &str,
        login_state: &str,
        los_nights: i32,
    ) -> String {
        format!(
            "price:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            hotel_id, checkin, checkout, rooms, adults, method, device, login_state, los_nights
        )
    }

    /// Key for rate limiting SerpAPI calls
    /// Format: ratelimit:serpapi:{minute}
    pub fn serpapi_rate_limit(minute: i64) -> String {
        format!("ratelimit:serpapi:{}", minute)
    }

    /// Key for rate limiting Gother API calls
    /// Format: ratelimit:gother:{minute}
    pub fn gother_rate_limit(minute: i64) -> String {
        format!("ratelimit:gother:{}", minute)
    }

    /// Key for job processing lock
    /// Format: lock:job:{job_id}
    pub fn job_lock(job_id: Uuid) -> String {
        format!("lock:job:{}", job_id)
    }

    /// Key for hotel processing lock within a job
    /// Format: lock:hotel:{job_id}:{hotel_id}
    pub fn hotel_lock(job_id: Uuid, hotel_id: Uuid) -> String {
        format!("lock:hotel:{}:{}", job_id, hotel_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotel_price_key() {
        let hotel_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let checkin = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
        let checkout = NaiveDate::from_ymd_opt(2026, 4, 21).unwrap();

        let key = CacheKeys::hotel_price(hotel_id, checkin, checkout, 1, 2);
        
        assert!(key.starts_with("price:"));
        assert!(key.contains("2026-04-20"));
        assert!(key.contains("2026-04-21"));
    }
}
