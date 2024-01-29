//! Holds all the errors for OS detection

use std::io;

use etherparse::ReadError;
use thiserror::Error;
use tokio::task::JoinError;

/// OS detection error types
#[derive(Debug, Error)]
pub enum OsDetectionError {
    /// Error joining a task
    #[error("Error joining task: {0}")]
    TaskJoin(#[from] JoinError),
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
}
