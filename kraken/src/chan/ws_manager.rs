use std::collections::HashMap;

use actix_toolbox::ws;
use actix_toolbox::ws::Message;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task;

pub(crate) async fn start_ws_sender(tx: ws::Sender, mut rx: mpsc::Receiver<WsMessage>) {
    while let Some(msg) = rx.recv().await {
        match msg {
            WsMessage::ServerQuitSocket => {
                if let Err(err) = tx.close().await {
                    error!("Error while closing ws sender: {err}");
                }
                break;
            }
            _ => {
                let txt = match serde_json::to_string(&msg) {
                    Ok(v) => v,
                    Err(err) => {
                        error!("Error serializing WsMessage: {err}");
                        continue;
                    }
                };

                if let Err(err) = tx.send(Message::Text(txt.into())).await {
                    error!("Error sending to client: {err}, closing socket");
                    if let Err(err) = tx.close().await {
                        error!("Error closing socket: {err}");
                    }
                }
            }
        }
    }
}

/// Entry of certificate transparency results
#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct CertificateTransparencyEntry {
    pub(crate) serial_number: String,
    pub(crate) issuer_name: String,
    pub(crate) common_name: String,
    pub(crate) value_names: Vec<String>,
    pub(crate) not_before: Option<DateTime<Utc>>,
    pub(crate) not_after: Option<DateTime<Utc>>,
}

/// Message that is sent via websocket
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub(crate) enum WsMessage {
    #[serde(skip)]
    ServerQuitSocket,
    InvalidMessage,
    AttackFinished {
        attack_id: i64,
        finished_successful: bool,
    },
    BruteforceSubdomainsResult {
        attack_id: i64,
        source: String,
        to: String,
    },
    ScanTcpPortsResult {
        attack_id: i64,
        address: String,
        port: u16,
    },
    CertificateTransparencyResult {
        attack_id: i64,
        entries: Vec<CertificateTransparencyEntry>,
    },
}

pub(crate) type WsManagerChan = Sender<WsManagerMessage>;

/// Messages to control the websocket manager
pub(crate) enum WsManagerMessage {
    /// Close the socket from the server side
    CloseSocket(Vec<u8>),
    /// Client with given uuid initialized a websocket
    OpenedSocket(Vec<u8>, ws::Sender),
    /// Send a message to given uuid
    Message(Vec<u8>, WsMessage),
}

pub(crate) async fn start_ws_manager() -> Result<WsManagerChan, String> {
    let mut lookup: HashMap<Vec<u8>, Vec<Sender<WsMessage>>> = HashMap::new();

    let (tx, mut rx) = mpsc::channel(16);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                WsManagerMessage::CloseSocket(uuid) => {
                    // Trigger close for all websockets associated with uuid
                    if let Some(sockets) = lookup.get(&uuid) {
                        for s in sockets {
                            if !s.is_closed() {
                                if let Err(err) = s.send(WsMessage::ServerQuitSocket).await {
                                    error!("Couldn't send close to ws sender: {err}");
                                }
                            }
                        }
                    }

                    lookup.remove(&uuid);
                }
                WsManagerMessage::OpenedSocket(uuid, ws_tx) => {
                    let (tx, rx) = mpsc::channel(16);
                    task::spawn(start_ws_sender(ws_tx, rx));

                    // Add new client connection to state
                    if let Some(sockets) = lookup.get_mut(&uuid) {
                        sockets.push(tx);
                    }
                    // Insert new client connection
                    else {
                        lookup.insert(uuid, vec![tx]);
                    }
                }
                WsManagerMessage::Message(uuid, msg) => {
                    if let Some(sender) = lookup.get(&uuid) {
                        for tx in sender {
                            if let Err(err) = tx.send(msg.clone()).await {
                                error!("Could not send to ws sender: {err}");
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(tx)
}
