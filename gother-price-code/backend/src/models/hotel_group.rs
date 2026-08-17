//! Hotel Group Model

use crate::models::scrape_job::ScrapeMethod;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Hotel Group - a collection of hotels for batch processing, plus the
/// saved price-search config used by manual runs and by the scheduler
/// (ADR-012). `search_days_ahead` is an offset rather than a calendar date
/// so a saved search never goes stale.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HotelGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub search_method: ScrapeMethod,
    pub search_days_ahead: Vec<i32>,
    pub search_rooms: i16,
    pub search_adults: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to update only the search config, kept separate from
/// `UpdateHotelGroupRequest` so renaming a group cannot silently clobber
/// its search settings (or vice versa).
#[derive(Debug, Deserialize)]
pub struct UpdateGroupSearchConfigRequest {
    pub search_method: Option<ScrapeMethod>,
    pub search_days_ahead: Option<Vec<i32>>,
    pub search_rooms: Option<i16>,
    pub search_adults: Option<i16>,
}

/// Check-in/check-out for one booking window, at the fixed one-night stay
/// every windowed run uses. Pure so the offset arithmetic is testable
/// without a DB — an off-by-one on check-out is the easy mistake here.
pub fn stay_dates(today: chrono::NaiveDate, days_ahead: i32) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let checkin = today + chrono::Duration::days(days_ahead.max(0) as i64);
    (checkin, checkin + chrono::Duration::days(1))
}

/// Hotel Group with hotel count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelGroupWithCount {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub hotel_count: i64,
    pub last_scraped_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new hotel group
#[derive(Debug, Deserialize)]
pub struct CreateHotelGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update a hotel group
#[derive(Debug, Deserialize)]
pub struct UpdateHotelGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Hotel Group Member - junction table entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HotelGroupMember {
    pub id: Uuid,
    pub hotel_group_id: Uuid,
    pub hotel_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn checkout_is_the_night_after_checkin() {
        let (ci, co) = stay_dates(d(2026, 8, 16), 7);
        assert_eq!(ci, d(2026, 8, 23));
        assert_eq!(co, d(2026, 8, 24));
    }

    #[test]
    fn zero_days_ahead_means_today() {
        assert_eq!(stay_dates(d(2026, 8, 16), 0).0, d(2026, 8, 16));
    }

    #[test]
    fn crosses_month_and_year_boundaries() {
        let (ci, co) = stay_dates(d(2026, 12, 28), 7);
        assert_eq!(ci, d(2027, 1, 4));
        assert_eq!(co, d(2027, 1, 5));
    }

    /// Defensive: bad data must not produce a checkout before checkin.
    #[test]
    fn negative_offset_clamps_to_today() {
        let (ci, co) = stay_dates(d(2026, 8, 16), -5);
        assert_eq!(ci, d(2026, 8, 16));
        assert!(co > ci);
    }
}
