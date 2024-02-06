//! Holds all the errors for OS detection

use std::io;

use etherparse::{ReadError, TcpOptionReadError};
use thiserror::Error;
use tokio::task::JoinError;
use tokio::time::error::Elapsed;

use crate::modules::os_detection::OperatingSystemInfo;

/// OS detection error types
#[derive(Debug, Error)]
pub enum OsDetectionError {
    /// OS detection procedures reported too different operating systems.
    #[error("OS detection procedures reported too different operating systems. ({0:?})")]
    Ambiguous(Vec<OperatingSystemInfo>),
    /// Error joining a task
    #[error("Error joining task: {0}")]
    TaskJoin(#[from] JoinError),
    /// Internal error using raw TCP
    #[error("Internal error using raw TCP: {0}")]
    RawTcpError(#[from] RawTcpError),
}

/// Possible errors inside the `fingerprint_tcp` method
#[derive(Debug, Error)]
pub enum TcpFingerprintError {
    /// TCP connection timed out
    #[error("TCP connection timed out")]
    ConnectionTimeout,
    /// Internal error using raw TCP
    #[error("Internal error using raw TCP: {0}")]
    RawTcpError(#[from] RawTcpError),
}

impl From<Elapsed> for TcpFingerprintError {
    fn from(_: Elapsed) -> Self {
        TcpFingerprintError::ConnectionTimeout
    }
}

/// Raw TCP I/O errors
#[derive(Debug, Error)]
pub enum RawTcpError {
    /// Regular I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    /// TCP/IP parsing failed
    #[error("Failed parsing IP/TCP packet: {0}")]
    PacketParseError(#[from] ReadError),
    /// Failed parsing TCP option
    #[error("Failed parsing TCP option: {0}")]
    TcpOptionParseError(#[from] TcpOptionReadError),
    /// Socket was created in IPv4/v6 domain, but local_addr didn't match the domain
    #[error("Socket was created in IPv4/v6 domain, but local_addr didn't match the domain")]
    InvalidLocalAddrDomain,
    /// Could not find both opened and closed ports
    #[error("Could not find both opened and closed ports")]
    NoPortsFound,
}
