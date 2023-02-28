//! The errors of a port scan

use std::fmt::{Display, Formatter};
use std::io;

/// The errors of a tcp port scan
#[derive(Debug)]
pub enum TcpPortScanError {
    /// Error while creating the icmp client
    CreateIcmpClient(io::Error),
    /// Error while rising the NO_FILE limit
    RiseNoFileLimit(io::Error),
}

impl Display for TcpPortScanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TcpPortScanError::CreateIcmpClient(err) => {
                write!(f, "Could not create icmp client: {err}")
            }
            TcpPortScanError::RiseNoFileLimit(err) => {
                write!(f, "Could not increase NO_FILE limit: {err}")
            }
        }
    }
}
