//! # kraken-sdk
//!
//! The SDK to [Kraken](https://github.com/myOmikron/kraken-project).

#![warn(clippy::unwrap_used, clippy::expect_used, missing_docs)]
#![forbid(unsafe_code)]

use std::sync::Arc;

use reqwest::cookie::Jar;
use reqwest::{ClientBuilder, Url};

use crate::error::KrakenError;

pub mod error;
pub(crate) mod sdk;

/// The result that is used throughout the API
pub type KrakenResult<T> = Result<T, KrakenError>;

/// The client for the [kraken](https://github.com/myOmikron/kraken-project) API.
pub struct KrakenClient {
    username: String,
    password: String,
    jar: Arc<Jar>,
    client: reqwest::Client,
    base_url: Url,
}

impl KrakenClient {
    /// Create a new instance of the [KrakenClient]
    pub fn new(
        base_url: Url,
        username: String,
        password: String,
        do_not_verify_certs: bool,
    ) -> Result<Self, KrakenError> {
        let jar = Arc::new(Jar::default());
        let mut client = ClientBuilder::new().cookie_provider(jar.clone());

        if do_not_verify_certs {
            client = client.danger_accept_invalid_certs(true);
        }

        Ok(Self {
            base_url,
            jar,
            client: client.build()?,
            username,
            password,
        })
    }
}
