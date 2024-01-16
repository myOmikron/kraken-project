//! All errors of the SDK are defined in this module

use kraken::api::handler::common::schema::ApiErrorResponse;
use thiserror::Error;

/// The main error type for handling
#[derive(Error, Debug)]
pub enum KrakenError {
    /// An error occurred while using the reqwest library
    #[error("Error from reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Could not deserialize body
    #[error("Error deserializing body: {0}")]
    DeserializeError(String),
    /// Authentication failed
    #[error("The authentication has failed")]
    AuthenticationFailed,
    /// An error was returned from kraken
    #[error("Kraken returned an error: {0:?}")]
    ApiError(ApiErrorResponse),
}
