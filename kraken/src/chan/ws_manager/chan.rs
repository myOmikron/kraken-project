use std::collections::HashMap;

use actix_toolbox::ws;
use actix_toolbox::ws::Message;
use bytestring::ByteString;
use log::{debug, error};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;

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
        match serde_json::to_string(&msg) {
            Ok(string) => {
                self.send(WsManagerEvent::Message(uuid, string.into()))
                    .await
            }
            Err(err) => error!("Error serializing WsMessage: {err}"),
        }
    }

    /// Send a message to a workspace
    pub async fn message_workspace(&self, workspace: Uuid, msg: WsMessage) {
        match GLOBAL
            .workspace_cache
            .get_users(workspace, &GLOBAL.db)
            .await
        {
            Ok(Some(users)) => {
                for user in users {
                    self.message(user, msg.clone()).await;
                }
            }
            Ok(None) => debug!("No users in cache, nothing to do"),
            Err(err) => error!("Cache error: {err}"),
        }
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

/// Start the WS manager
pub async fn start_ws_manager() -> WsManagerChan {
    let (sender, receiver) = mpsc::channel(16);
    tokio::spawn(run_ws_manager(receiver));
    WsManagerChan(sender)
}

enum WsManagerEvent {
    Add(Uuid, ws::Sender),
    /// The [`ByteString`] contains the serialized form of [`WsMessage`]
    Message(Uuid, ByteString),
    CloseAll(Uuid),
}

async fn run_ws_manager(mut receiver: mpsc::Receiver<WsManagerEvent>) {
    let mut sockets: HashMap<Uuid, Vec<mpsc::Sender<ByteString>>> = HashMap::new();

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

async fn run_single_socket(actor_chan: ws::Sender, mut manager_chan: mpsc::Receiver<ByteString>) {
    loop {
        // Receive
        let Some(msg) = manager_chan.recv().await else {
            if actor_chan.close().await.is_err() {
                debug!("Couldn't close websocket, because it is already closed");
            }
            return;
        };

        // Send
        let Ok(_) = actor_chan.send(Message::Text(msg)).await else {
            debug!("Couldn't send to websocket, because it is closed");
            manager_chan.close();
            return;
        };
    }
}
