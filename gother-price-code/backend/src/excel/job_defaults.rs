//! JobDefaults fallback merging (REQ-001 F-002).
//!
//! A per-hotel override sheet may leave any field blank; blank fields fall
//! back to the job-level defaults supplied when the scrape job was created.

use crate::models::{JobDefaults, JobHotelParamOverride};
use chrono::NaiveDate;

/// Resolved, fully-populated search parameters for one hotel in a job.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedHotelParams {
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub rooms: i32,
    pub adults: i32,
}

/// Merge a (possibly absent) per-hotel override onto job-level defaults.
/// Every field is `override.unwrap_or(default)` independently.
pub fn merge_job_params(
    defaults: JobDefaults,
    override_row: Option<&JobHotelParamOverride>,
) -> ResolvedHotelParams {
    let o = override_row;

    ResolvedHotelParams {
        checkin_date: o.and_then(|o| o.checkin_date).unwrap_or(defaults.checkin_date),
        checkout_date: o
            .and_then(|o| o.checkout_date)
            .unwrap_or(defaults.checkout_date),
        rooms: o.and_then(|o| o.rooms).unwrap_or(defaults.rooms),
        adults: o.and_then(|o| o.adults).unwrap_or(defaults.adults),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> JobDefaults {
        JobDefaults {
            checkin_date: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            checkout_date: NaiveDate::from_ymd_opt(2026, 8, 2).unwrap(),
            rooms: 1,
            adults: 2,
        }
    }

    #[test]
    fn no_override_uses_defaults() {
        let resolved = merge_job_params(defaults(), None);
        assert_eq!(resolved.checkin_date, defaults().checkin_date);
        assert_eq!(resolved.rooms, 1);
        assert_eq!(resolved.adults, 2);
    }

    #[test]
    fn partial_override_fills_only_set_fields() {
        let override_row = JobHotelParamOverride {
            hid: Some(1),
            hotel_name: None,
            checkin_date: None,
            checkout_date: None,
            rooms: Some(3),
            adults: None,
            currency: None,
        };

        let resolved = merge_job_params(defaults(), Some(&override_row));
        assert_eq!(resolved.checkin_date, defaults().checkin_date);
        assert_eq!(resolved.rooms, 3);
        assert_eq!(resolved.adults, 2);
    }
}
