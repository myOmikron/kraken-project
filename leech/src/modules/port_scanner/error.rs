//! The errors of a port scan

use std::io;

use thiserror::Error;
use tokio::task::JoinError;

use crate::modules::host_alive::error::IcmpScanError;

/// The errors of a tcp port scan
#[derive(Debug, Error)]
pub enum TcpPortScanError {
    /// Error while creating the icmp client
    #[error("Could not create icmp client: {0}")]
    CreateIcmpClient(io::Error),

    /// Error while rising the NO_FILE limit
    #[error("Could not increase NO_FILE limit: {0}")]
    RiseNoFileLimit(io::Error),

    /// Error joining a task
    #[error("Error joining task: {0}")]
    TaskJoin(#[from] JoinError),
}

impl From<IcmpScanError> for TcpPortScanError {
    fn from(value: IcmpScanError) -> Self {
        match value {
            IcmpScanError::CreateIcmpClient(v) => Self::CreateIcmpClient(v),
            IcmpScanError::RiseNoFileLimit(v) => Self::RiseNoFileLimit(v),
        }
    }
}
