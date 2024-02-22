//! Some utility functions for `DateTime<Utc>`

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Utc;
use log::warn;

/// Convert a raw timestamp in seconds into a `DateTime<Utc>`
///
/// The out of range error by `chrono` is handled by clipping to the range.
pub fn utc_from_seconds(seconds: i64) -> DateTime<Utc> {
    Utc.from_utc_datetime(
        &if let Some(time) = NaiveDateTime::from_timestamp_opt(seconds, 0) {
            time
        } else if seconds < 0 {
            warn!("Timestamp is out of NaiveDateTime's range. Falling back to its minimum.");
            NaiveDateTime::MIN
        } else {
            warn!("Timestamp is out of NaiveDateTime's range. Falling back to its maximum.");
            NaiveDateTime::MAX
        },
    )
}
