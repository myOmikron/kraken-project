//! Some utility functions for `DateTime<Utc>`

use chrono::DateTime;
use chrono::Utc;
use log::warn;

/// Convert a raw timestamp in seconds into a `DateTime<Utc>`
///
/// The out of range error by `chrono` is handled by clipping to the range.
pub fn utc_from_seconds(seconds: i64) -> DateTime<Utc> {
    if let Some(time) = DateTime::from_timestamp(seconds, 0) {
        time
    } else if seconds < 0 {
        warn!("Timestamp is out of DateTime's range. Falling back to its minimum.");
        DateTime::<Utc>::MIN_UTC
    } else {
        warn!("Timestamp is out of DateTime's range. Falling back to its maximum.");
        DateTime::<Utc>::MAX_UTC
    }
}
