//! The errors of a port scan

use std::fmt::{Display, Formatter};
use std::io;

use tokio::task::JoinError;

use crate::modules::host_alive::error::IcmpScanError;

/// The errors of a tcp port scan
#[derive(Debug)]
pub enum TcpPortScanError {
    /// Error while creating the icmp client
    CreateIcmpClient(io::Error),
    /// Error while rising the NO_FILE limit
    RiseNoFileLimit(io::Error),
    /// Error joining a task
    TaskJoin(JoinError),
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
            TcpPortScanError::TaskJoin(err) => {
                write!(f, "Error joining task: {err}")
            }
        }
    }
}

impl From<IcmpScanError> for TcpPortScanError {
    fn from(value: IcmpScanError) -> Self {
        match value {
            IcmpScanError::CreateIcmpClient(v) => Self::CreateIcmpClient(v),
        }
    }
}
