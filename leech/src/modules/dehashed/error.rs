//! The errors of the dehashed module

use thiserror::Error;

/// The errors from the dehashed module
#[derive(Debug, Error)]
pub enum DehashedError {
    /// Errors that originated from dehashed_rs
    #[error("Dehashed error: {0}")]
    Dehashed(#[from] dehashed_rs::DehashedError),

    /// The dehashed scheduler could not be reached
    #[error("An error occurred while communicating with the dehashed scheduler")]
    DehashedSchedulerUnreachable,
}
