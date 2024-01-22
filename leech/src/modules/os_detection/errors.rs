//! Holds all the errors for OS detection

use thiserror::Error;
use tokio::task::JoinError;

/// DNS Resolution error types
#[derive(Debug, Error)]
pub enum OsDetectionError {
    /// Error joining a task
    #[error("Error joining task: {0}")]
    TaskJoin(#[from] JoinError),
}
