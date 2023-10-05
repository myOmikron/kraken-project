//! All host alive errors

use std::fmt::{Display, Formatter};
use std::io;

/// The errors originating from an icmp scan
#[derive(Debug)]
pub enum IcmpScanError {
    /// Error while creating the icmp client
    CreateIcmpClient(io::Error),
    /// Error while rising the NO_FILE limit
    RiseNoFileLimit(io::Error),
}

impl Display for IcmpScanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IcmpScanError::CreateIcmpClient(err) => {
                write!(f, "Could not create icmp client: {err}")
            }
            IcmpScanError::RiseNoFileLimit(err) => {
                write!(f, "Could not increase NO_FILE limit: {err}")
            }
        }
    }
}
