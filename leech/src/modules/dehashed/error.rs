//! The errors of the dehashed module

use std::fmt::{Display, Formatter};

/// The errors from the dehashed module
#[derive(Debug)]
pub enum DehashedError {
    /// Errors that originated from dehashed_rs
    Dehashed(dehashed_rs::DehashedError),
    /// The dehashed scheduler could not be reached
    DehashedSchedulerUnreachable,
}

impl Display for DehashedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DehashedError::Dehashed(err) => write!(f, "Dehashed error: {err}"),
            DehashedError::DehashedSchedulerUnreachable => {
                write!(
                    f,
                    "An error occurred while communicating with the dehashed scheduler"
                )
            }
        }
    }
}

impl From<dehashed_rs::DehashedError> for DehashedError {
    fn from(value: dehashed_rs::DehashedError) -> Self {
        Self::Dehashed(value)
    }
}
