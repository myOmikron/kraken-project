use thiserror::Error;

/// The error type of Whois queries
#[derive(Debug, Error)]
pub enum WhoisError {
    /// Error in reqwest
    #[error("Error in reqwest: {0}")]
    ReqwestError(#[from] reqwest::Error),
    /// Error in response
    #[error("Invalid response found")]
    InvalidResponse,
    /// Deserialize error when parsing response from ripe
    #[error("Couldn't deserialize response: {0}")]
    DeserializeError(#[from] serde_json::Error),
}
