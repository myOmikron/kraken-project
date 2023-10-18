//! All host alive errors

use std::io;

use thiserror::Error;

/// The errors originating from an icmp scan
#[derive(Debug, Error)]
pub enum IcmpScanError {
    /// Error while creating the icmp client
    #[error("Could not create icmp client: {0}")]
    CreateIcmpClient(io::Error),

    /// Error while rising the NO_FILE limit
    #[error("Could not increase NO_FILE limit: {0}")]
    RiseNoFileLimit(io::Error),
}
