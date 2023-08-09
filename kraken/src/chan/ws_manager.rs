use std::collections::HashMap;

use actix_toolbox::ws;
use actix_toolbox::ws::Message;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task;
use webauthn_rs::prelude::Uuid;

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
pub struct CertificateTransparencyEntry {
    /// The serial number of the certificate
    pub serial_number: String,
    /// The name of the issuer for the certificate
    pub issuer_name: String,
    /// The common name of the certificate
    pub common_name: String,
    /// The value names of the certificate
    pub value_names: Vec<String>,
    /// The point in time after the certificate is valid
    pub not_before: Option<DateTime<Utc>>,
    /// The point in time before the certificate is valid
    pub not_after: Option<DateTime<Utc>>,
}

/// Message that is sent via websocket
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// The message for the websocket worker to stop and quit.
    ///
    /// This message is not sent to the client.
    #[serde(skip)]
    ServerQuitSocket,
    /// An invalid message was received.
    ///
    /// This message type is sent to the client.
    InvalidMessage,
    /// A notification about a finished attack
    AttackFinished {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// Whether the attack was finished successful
        finished_successful: bool,
    },
    /// A result for a subdomain enumeration using bruteforce DNS requests
    BruteforceSubdomainsResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The source address that was queried
        source: String,
        /// The to address that was returned
        to: String,
    },
    /// A result for a tcp scan
    ScanTcpPortsResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The address of the result
        address: String,
        /// The port of the result
        port: u16,
    },
    /// A result to a certificate transparency request
    CertificateTransparencyResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The entries of the result
        entries: Vec<CertificateTransparencyEntry>,
    },
}

/// A channel to send [WsManagerMessage] to the ws manager
pub type WsManagerChan = Sender<WsManagerMessage>;

/// Messages to control the websocket manager
pub enum WsManagerMessage {
    /// Close the socket from the server side
    CloseSocket(Uuid),
    /// Client with given uuid initialized a websocket
    OpenedSocket(Uuid, ws::Sender),
    /// Send a message to given uuid
    Message(Uuid, WsMessage),
}

pub(crate) async fn start_ws_manager() -> Result<WsManagerChan, String> {
    let mut lookup: HashMap<Uuid, Vec<Sender<WsMessage>>> = HashMap::new();

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
