use std::collections::HashMap;
use std::net::IpAddr;

use actix_toolbox::ws;
use actix_toolbox::ws::Message;
use chrono::{DateTime, Utc};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use utoipa::ToSchema;
use webauthn_rs::prelude::Uuid;

use crate::api::handler::users::SimpleUser;
use crate::api::handler::workspaces::SimpleWorkspace;

/// Entry of certificate transparency results
#[derive(Deserialize, Serialize, Clone, ToSchema)]
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
#[derive(Deserialize, Serialize, Clone, ToSchema)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// An invalid message was received.
    ///
    /// This message type is sent to the client.
    InvalidMessage {},
    /// An invitation to a workspace was issued
    InvitationToWorkspace {
        /// The workspace the user is invited to
        workspace: SimpleWorkspace,
        /// The user that has issued the invitation
        from: SimpleUser,
    },
    /// A notification about a finished attack
    AttackFinished {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// Whether the attack was finished successful
        finished_successful: bool,
    },
    // TODO: TaskFinished as generic result
    /// A notification about a finished search
    SearchFinished {
        /// The corresponding id of the search
        search_uuid: Uuid,
        /// Whether the search was finished successfully
        finished_successful: bool,
    },
    /// A notification about a search result
    SearchNotify {
        /// The corresponding id of the search results
        search_uuid: Uuid,
        /// A result entry
        result_uuid: Uuid,
    },
    /// A result for a subdomain enumeration using bruteforce DNS requests
    BruteforceSubdomainsResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The source address that was queried
        source: String,
        /// The destination address that was returned
        destination: String,
    },
    /// A result for hosts alive check
    HostsAliveCheck {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// A host which could be reached
        #[schema(value_type = String)]
        host: IpAddr,
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
    /// A result to service detection request
    ServiceDetectionResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// Name of the service
        service: String,
    },
    /// A result for a DNS resolution requests
    DnsResolutionResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The source address that was queried
        source: String,
        /// The destination address that was returned
        destination: String,
    },
}

/// A channel to send events to the ws manager
#[derive(Clone, Debug)]
pub struct WsManagerChan(mpsc::Sender<WsManagerEvent>);

impl WsManagerChan {
    /// Add a newly opened websocket for a user
    pub async fn add(&self, uuid: Uuid, socket: ws::Sender) {
        self.send(WsManagerEvent::Add(uuid, socket)).await;
    }

    /// Send a message to a user
    pub async fn message(&self, uuid: Uuid, msg: WsMessage) {
        self.send(WsManagerEvent::Message(uuid, msg)).await;
    }

    /// Close all websocket's owned by a user
    pub async fn close_all(&self, uuid: Uuid) {
        self.send(WsManagerEvent::CloseAll(uuid)).await;
    }

    async fn send(&self, event: WsManagerEvent) {
        if self.0.send(event).await.is_err() {
            error!("The ws_manager died! This should never happen!");
        }
    }
}

pub(crate) async fn start_ws_manager() -> WsManagerChan {
    let (sender, receiver) = mpsc::channel(16);
    tokio::spawn(run_ws_manager(receiver));
    WsManagerChan(sender)
}

enum WsManagerEvent {
    Add(Uuid, ws::Sender),
    Message(Uuid, WsMessage),
    CloseAll(Uuid),
}

async fn run_ws_manager(mut receiver: mpsc::Receiver<WsManagerEvent>) {
    let mut sockets: HashMap<Uuid, Vec<mpsc::Sender<WsMessage>>> = HashMap::new();

    while let Some(event) = receiver.recv().await {
        match event {
            WsManagerEvent::Add(uuid, socket) => {
                let (tx, rx) = mpsc::channel(16);
                tokio::spawn(run_single_socket(socket, rx));
                sockets.entry(uuid).or_default().push(tx);
            }
            WsManagerEvent::CloseAll(uuid) => {
                sockets.remove(&uuid);
            }
            WsManagerEvent::Message(uuid, msg) => {
                if let Some(sockets) = sockets.get_mut(&uuid) {
                    let mut closed = Vec::new();
                    for (index, socket) in sockets.iter().enumerate() {
                        // Try send
                        if socket.send(msg.clone()).await.is_err() {
                            // Note the closed ones
                            closed.push(index);
                        }
                    }
                    // Remove the closed ones
                    for index in closed.into_iter().rev() {
                        sockets.swap_remove(index);
                    }
                }
            }
        }
    }
}

async fn run_single_socket(actor_chan: ws::Sender, mut manager_chan: mpsc::Receiver<WsMessage>) {
    loop {
        // Receive
        let Some(msg) = manager_chan.recv().await else {
            if actor_chan.close().await.is_err() {
                debug!("Couldn't close websocket, because it is already closed");
            }
            return;
        };

        // Convert
        let txt = match serde_json::to_string(&msg) {
            Ok(v) => v,
            Err(err) => {
                error!("Error serializing WsMessage: {err}");
                continue;
            }
        };

        // Send
        let Ok(_) = actor_chan.send(Message::Text(txt.into())).await else {
            debug!("Couldn't send to websocket, because it is closed");
            manager_chan.close();
            return;
        };
    }
}
