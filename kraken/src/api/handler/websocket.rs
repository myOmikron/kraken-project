use std::sync::Arc;
use std::time::{Duration, Instant};

use actix_toolbox::tb_middleware::Session;
use actix_toolbox::ws;
use actix_toolbox::ws::{MailboxError, Message};
use actix_web::web::{Data, Payload};
use actix_web::{HttpRequest, HttpResponse};
use bytes::Bytes;
use log::{debug, error};
use tokio::sync::Mutex;

use crate::api::handler::ApiError;
use crate::chan::{WsManagerChan, WsManagerMessage, WsMessage};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

#[utoipa::path(
    get,
    context_path = "/api/v1",
    path = "/ws",
    tag = "Websocket",
    responses(
        (status = 101, description = "Websocket connection established"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
pub(crate) async fn websocket(
    request: HttpRequest,
    payload: Payload,
    session: Session,
    ws_manager_chan: Data<WsManagerChan>,
) -> Result<HttpResponse, actix_web::Error> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let (tx, mut rx, response) = ws::start(&request, payload)?;
    debug!("Initializing websocket connection");
    let last_hb = Arc::new(Mutex::new(Instant::now()));

    // heartbeat
    let heartbeat_tx = tx.clone();
    let heartbeat_time = last_hb.clone();
    tokio::spawn(async move {
        loop {
            if Instant::now().duration_since(*heartbeat_time.lock().await) > CLIENT_TIMEOUT
                && heartbeat_tx.close().await.is_ok()
            {
                debug!("Closed websocket due to missing heartbeat responses");
            }

            tokio::time::sleep(Duration::from_secs(10)).await;

            if let Err(err) = heartbeat_tx.send(Message::Ping(Bytes::from(""))).await {
                match err {
                    MailboxError::Closed => {
                        debug!("Websocket was closed by another tx instance")
                    }
                    MailboxError::Timeout => {
                        debug!("Got timeout sending to client, trying to close socket");
                        if heartbeat_tx.close().await.is_err() {
                            debug!("Error closing socket")
                        }
                    }
                }
                break;
            }
        }
    });

    // Receiver
    let recv_tx = tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Ok(msg) => {
                    {
                        *last_hb.lock().await = Instant::now();
                    }

                    match msg {
                        Message::Text(data) => {
                            match serde_json::from_str::<WsMessage>(data.as_ref()) {
                                Ok(_) => {
                                    // TODO
                                }
                                Err(err) => {
                                    debug!("Error deserializing data: {err}");

                                    let msg =
                                        serde_json::to_string(&WsMessage::InvalidMessage).unwrap();
                                    if let Err(err) = recv_tx.send(Message::Text(msg.into())).await
                                    {
                                        error!("Error sending message: {err}");
                                    }
                                }
                            }
                        }
                        Message::Ping(data) => {
                            if let Err(err) = recv_tx.send(Message::Pong(data)).await {
                                debug!("Error while sending pong: {err}");
                                if let Err(err) = recv_tx.close().await {
                                    debug!("Error closing socket: {err}");
                                }
                            }
                        }
                        Message::Close(_) => {
                            if let Err(err) = recv_tx.close().await {
                                debug!("Error closing websocket: {err}");
                            }
                        }
                        _ => {}
                    }
                }
                Err(err) => {
                    debug!("Error while receiving from websocket: {err}");
                    if let Err(err) = recv_tx.close().await {
                        debug!("Error while closing websocket: {err}");
                    }
                }
            }
        }
    });

    // Give sender to ws manager
    if let Err(err) = ws_manager_chan
        .send(WsManagerMessage::OpenedSocket(uuid, tx.clone()))
        .await
    {
        error!("Could not send ws tx to ws manager: {err}. Closing websocket");
        if let Err(err) = tx.close().await {
            error!("Couldn't close websocket: {err}");
        }
    }

    Ok(response)
}
