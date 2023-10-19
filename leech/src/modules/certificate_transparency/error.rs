//! The error that can be returned by certificate transparency

use thiserror::Error;

/// The error that can be returned by certificate transparency
#[derive(Debug, Error)]
pub enum CertificateTransparencyError {
    /// Couldn't fetch data from crt.sh
    #[error("Couldn't fetch the data")]
    CouldntFetchData,

    /// Reqwest error
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Couldn't deserialize data from json
    #[error("Could not deserialize: {0}")]
    DeserializeError(#[from] serde_json::Error),
}
