use std::sync::Arc;

use kraken::chan::ws_manager::schema::WsMessage;
use reqwest::cookie::Jar;
use reqwest::ClientBuilder;
use reqwest::Url;
use tokio::sync::mpsc::Sender;

use crate::error::KrakenError;

mod attacks;
mod auth;
mod domains;
mod global_tags;
mod hosts;
mod invitations;
mod ports;
mod services;
mod utils;
mod wordlists;
mod workspace_tags;
mod workspaces;
mod ws;

/// The result that is used throughout the API
pub type KrakenResult<T> = Result<T, KrakenError>;

/// The client for the [kraken](https://github.com/myOmikron/kraken-project) API.
pub struct KrakenClient {
    username: String,
    password: String,
    jar: Arc<Jar>,
    client: reqwest::Client,
    base_url: Url,
    do_not_verify_certs: bool,
    user_ws_tx: Option<Sender<WsMessage>>,
}

impl KrakenClient {
    /// Create a new instance of the [KrakenClient]
    pub fn new(
        base_url: Url,
        username: String,
        password: String,
        ws_tx: Option<Sender<WsMessage>>,
        do_not_verify_certs: bool,
    ) -> Result<Self, KrakenError> {
        let jar = Arc::new(Jar::default());
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(do_not_verify_certs)
            .cookie_provider(jar.clone());

        Ok(Self {
            base_url,
            jar,
            client: client.build()?,
            username,
            password,
            do_not_verify_certs,
            user_ws_tx: ws_tx,
        })
    }
}
