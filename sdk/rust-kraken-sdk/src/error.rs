//! All errors of the SDK are defined in this module

use std::io;

use kraken::api::handler::common::schema::ApiErrorResponse;
use thiserror::Error;
use tokio_native_tls::native_tls;

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
    /// No cookie was found in jar
    #[error("No cookie found")]
    MissingCookie,
    /// Io error
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    /// WS error
    #[error("WS error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    /// TLS error
    #[error("Io error: {0}")]
    TLS(#[from] native_tls::Error),
}
