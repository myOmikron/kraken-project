//! The websocket to the frontend client is defined in this module

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use actix_toolbox::ws;
use actix_toolbox::ws::MailboxError;
use actix_toolbox::ws::Message;
use actix_web::get;
use actix_web::web::Payload;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use bytes::Bytes;
use log::debug;
use log::error;
use tokio::sync::Mutex;

use crate::api::extractors::SessionUser;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::EditorTarget;
use crate::chan::ws_manager::schema::WsClientMessage;
use crate::chan::ws_manager::schema::WsMessage;

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// Start a websocket connection
///
/// A heartbeat PING packet is sent constantly (every 10s).
/// If no response is retrieved within 30s of the last transmission, the socket
/// will be closed.
#[utoipa::path(
    tag = "Websocket",
    context_path = "/api/v1",
    responses(
        (status = 101, description = "Websocket connection established"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/ws")]
pub async fn websocket(
    request: HttpRequest,
    payload: Payload,
    SessionUser(user_uuid): SessionUser,
) -> Result<HttpResponse, actix_web::Error> {
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
                            match serde_json::from_str::<WsClientMessage>(data.as_ref()) {
                                Ok(msg) => match msg {
                                    WsClientMessage::EditorChangedContent {
                                        target:
                                            EditorTarget::FindingDefinition {
                                                finding_definition,
                                                finding_section,
                                            },
                                        change,
                                    } => {
                                        tokio::spawn(async move {
                                            GLOBAL
                                                .editor_sync
                                                .process_client_edit_finding_definition(
                                                    user_uuid,
                                                    finding_definition,
                                                    finding_section,
                                                    change,
                                                )
                                                .await;
                                        });
                                    }
                                    WsClientMessage::EditorChangedContent {
                                        target: EditorTarget::WorkspaceNotes { workspace },
                                        change,
                                    } => {
                                        tokio::spawn(async move {
                                            GLOBAL
                                                .editor_sync
                                                .process_client_edit_ws_notes(
                                                    user_uuid, workspace, change,
                                                )
                                                .await;
                                        });
                                    }
                                    WsClientMessage::EditorChangedCursor {
                                        target:
                                            EditorTarget::FindingDefinition {
                                                finding_definition,
                                                finding_section,
                                            },
                                        cursor,
                                    } => {
                                        tokio::spawn(async move {
                                            GLOBAL
                                                .editor_sync
                                                .process_client_cursor_update_finding_definition(
                                                    user_uuid,
                                                    finding_definition,
                                                    finding_section,
                                                    cursor,
                                                )
                                                .await;
                                        });
                                    }
                                    WsClientMessage::EditorChangedCursor {
                                        target: EditorTarget::WorkspaceNotes { workspace },
                                        cursor,
                                    } => {
                                        tokio::spawn(async move {
                                            GLOBAL
                                                .editor_sync
                                                .process_client_cursor_update_ws_notes(
                                                    user_uuid, workspace, cursor,
                                                )
                                                .await;
                                        });
                                    }
                                    _ => {}
                                },
                                Err(err) => {
                                    debug!("Error deserializing data: {err}");

                                    // Unwrap is okay, as an empty variant can always be parsed to json
                                    #[allow(clippy::unwrap_used)]
                                    let msg = serde_json::to_string(&WsMessage::InvalidMessage {})
                                        .unwrap();
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
    GLOBAL.ws.add(user_uuid, tx.clone()).await;

    Ok(response)
}
