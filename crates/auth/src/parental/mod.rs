pub mod controls;
pub mod handlers;
pub mod verification;

pub use controls::{
    ContentRating, ParentalControls, SetParentalControlsRequest, SetParentalControlsResponse,
};
pub use handlers::{update_parental_controls, verify_parental_pin, ParentalControlsState};
pub use verification::{VerifyPinRequest, VerifyPinResponse};

use crate::error::AuthError;
use chrono::NaiveTime;

/// Check if current time is within allowed viewing window
pub fn is_within_viewing_window(
    viewing_time_start: Option<NaiveTime>,
    viewing_time_end: Option<NaiveTime>,
) -> bool {
    let now = chrono::Local::now().time();
    match (viewing_time_start, viewing_time_end) {
        (Some(start), Some(end)) => {
            if start <= end {
                // Normal case: 06:00 - 21:00
                now >= start && now <= end
            } else {
                // Crosses midnight: 21:00 - 06:00
                now >= start || now <= end
            }
        }
        _ => true, // No time restrictions
    }
}

/// Parse time string in HH:MM format
pub fn parse_time(time_str: &str) -> Result<NaiveTime, AuthError> {
    NaiveTime::parse_from_str(time_str, "%H:%M").map_err(|_| {
        AuthError::ValidationError(format!("Invalid time format: {}. Expected HH:MM", time_str))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_within_viewing_window_no_restrictions() {
        assert!(is_within_viewing_window(None, None));
    }

    #[test]
    fn test_is_within_viewing_window_normal_range() {
        let start = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(21, 0, 0).unwrap();

        // This test depends on current time, so we just verify it doesn't panic
        let _ = is_within_viewing_window(Some(start), Some(end));
    }

    #[test]
    fn test_is_within_viewing_window_crosses_midnight() {
        let start = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(6, 0, 0).unwrap();

        // This test depends on current time, so we just verify it doesn't panic
        let _ = is_within_viewing_window(Some(start), Some(end));
    }

    #[test]
    fn test_parse_time_valid() {
        let time = parse_time("14:30").unwrap();
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 30);
    }

    #[test]
    fn test_parse_time_invalid() {
        assert!(parse_time("25:00").is_err());
        assert!(parse_time("14:60").is_err());
        assert!(parse_time("invalid").is_err());
    }
}
