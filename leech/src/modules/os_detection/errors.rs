//! Holds all the errors for OS detection

use std::io;

use etherparse::err::ip::LaxHeaderSliceError;
use etherparse::TcpOptionReadError;
use thiserror::Error;
use tokio::task::JoinError;
use tokio::time::error::Elapsed;

use crate::modules::os_detection::OperatingSystemInfo;

/// OS detection error types
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum OsDetectionError {
    #[error("OS detection procedures reported multiple different operating systems. ({0:?})")]
    Ambiguous(Vec<OperatingSystemInfo>),
    #[error("Error joining task: {0}")]
    TaskJoin(#[from] JoinError),
    #[error("Internal error using raw TCP: {0}")]
    RawTcpError(#[from] RawTcpError),
    #[error("Invalid setting: {0}")]
    InvalidSetting(String),
}

/// Possible errors inside the `fingerprint_tcp` method
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum TcpFingerprintError {
    #[error("TCP connection timed out")]
    ConnectionTimeout,
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
#[allow(missing_docs)]
pub enum RawTcpError {
    /// Regular I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("Failed parsing IP packet: {0}")]
    IpParseError(#[from] etherparse::err::ip::SliceError),
    #[error("Failed parsing packet: {0}")]
    PacketParseError(#[from] LaxHeaderSliceError),
    #[error("Failed parsing TCP option: {0}")]
    TcpOptionParseError(#[from] TcpOptionReadError),
    #[error("Socket was created in IPv4/v6 domain, but local_addr didn't match the domain")]
    InvalidLocalAddrDomain,
    #[error("Could not find both opened and closed ports")]
    NoPortsFound,
}
