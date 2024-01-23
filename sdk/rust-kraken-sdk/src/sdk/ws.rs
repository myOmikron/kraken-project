use futures::{SinkExt, StreamExt, TryStreamExt};
use kraken::chan::ws_manager::schema::WsMessage;
use log::{debug, error, info};
use reqwest::cookie::CookieStore;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio_native_tls::{native_tls, TlsConnector, TlsStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::error::KrakenError;
use crate::{KrakenClient, KrakenResult};

impl KrakenClient {
    pub(crate) async fn start_ws(&self) -> KrakenResult<()> {
        info!("Trying to connect to websocket ..");

        #[allow(clippy::expect_used)]
        let mut url = self.base_url.join("api/v1/ws").expect("Valid url");
        #[allow(clippy::expect_used)]
        url.set_scheme("wss").expect("This must work");
        #[allow(clippy::expect_used)]
        let mut req = url.clone().into_client_request().expect("Valid request");

        req.headers_mut().insert(
            "Cookie",
            // tokio_tungstenite uses http v1.0, reqwest v0.2 so conversion is fine
            #[allow(clippy::unwrap_used)]
            HeaderValue::from_bytes(
                self.jar
                    .cookies(&self.base_url)
                    .ok_or(KrakenError::MissingCookie)?
                    .as_bytes(),
            )
            .unwrap(),
        );
        debug!("{url}");

        #[allow(clippy::expect_used)]
        let sock = TcpStream::connect(format!(
            "{}:{}",
            self.base_url.host_str().expect("Host is in self.base"),
            self.base_url.port().unwrap_or(443)
        ))
        .await?;

        let tls_conn = TlsConnector::from(
            // We are only changing the danger_accept_invalid_certs option, this can not cause a panic
            #[allow(clippy::unwrap_used)]
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(self.do_not_verify_certs)
                .build()
                .unwrap(),
        );

        let conn = tls_conn.connect(url.as_ref(), sock).await?;
        let (ws, _) = tokio_tungstenite::client_async(req, conn).await?;
        let user_tx = self.user_ws_tx.clone();
        tokio::spawn(ws_recv(ws, user_tx));

        Ok(())
    }
}

async fn ws_recv(
    ws: WebSocketStream<TlsStream<TcpStream>>,
    user_tx: Option<Sender<WsMessage>>,
) -> Result<(), WsError> {
    let (mut tx, mut rx) = ws.split();

    while let Ok(Some(msg)) = rx.try_next().await {
        match msg {
            Message::Text(txt) => {
                let msg = match serde_json::from_str(&txt) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Error deserializing value via ws: {err}");
                        continue;
                    }
                };

                if let Some(user_tx) = user_tx.as_ref() {
                    user_tx.send(msg).await.map_err(|_| WsError::UserTxDown)?;
                }
            }
            Message::Ping(data) => {
                if tx.send(Message::Pong(data)).await.is_err() {
                    return Err(WsError::WsDown);
                }
            }
            Message::Close(_) => {
                error!("Websocket was closed");
                return Err(WsError::WsDown);
            }
            _ => {
                debug!("Received invalid message over websocket")
            }
        }
    }

    error!("Websocket exited");

    Ok(())
}

#[derive(Error, Debug)]
pub enum WsError {
    /// The tx of the user mpsc channel is unable to be sent to
    #[error("User tx is not available anymore")]
    UserTxDown,
    /// The websocket has been closed
    #[error("Websocket is down")]
    WsDown,
}
