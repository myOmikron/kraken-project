//! This module implements all attacks as tasks to be spawned with `tokio::spawn`
//!
//! To start any attack create an [`AttackContext`] ([give it a leech](AttackContext::leech))
//! and call your desired attack method.

mod bruteforce_subdomains;
mod host_alive_check;
mod query_certificate_transparency;
mod query_dehashed;
mod service_detection;
mod tcp_port_scan;

use chrono::Utc;
use futures::{TryFuture, TryStreamExt};
use log::error;
use rorm::prelude::*;
use rorm::{update, Database};
use tonic::{Response, Status, Streaming};
use uuid::Uuid;

#[cfg(doc)]
use crate::api::handler;
use crate::chan::{LeechClient, WsManagerChan, WsManagerMessage, WsMessage};
use crate::models::Attack;

/// Common data required to start any attack
#[derive(Clone)]
pub struct AttackContext {
    /// Handle to the database to insert results into
    pub db: Database,

    /// Handle to send status updates over websocket
    pub ws_manager: WsManagerChan,

    /// The user starting the attack
    pub user_uuid: Uuid,

    /// The workspace the attack is started in
    pub workspace_uuid: Uuid,

    /// The attack's uuid
    pub attack_uuid: Uuid,
}

impl AttackContext {
    /// Add a leech to the context
    pub fn leech(self, leech: LeechClient) -> LeechAttackContext {
        LeechAttackContext {
            common: self,
            leech,
        }
    }
}

/// Common data required to start attacks on a leech
#[derive(Clone)]
pub struct LeechAttackContext {
    /// Common data required to start any attack
    pub common: AttackContext,

    /// Client for talking with the leech
    pub leech: LeechClient,
}

/* Some utility methods and impls */
impl AttackContext {
    /// Send a websocket message and log the error
    async fn send_ws(&self, message: WsMessage) {
        if self
            .ws_manager
            .send(WsManagerMessage::Message(self.user_uuid, message))
            .await
            .is_err()
        {
            error!("Couldn't send websocket message, the websocket manager died!");
        }
    }

    /// Send the user a notification and update the [`Attack`] model
    async fn set_finished(&self, error: Option<String>) {
        self.send_ws(WsMessage::AttackFinished {
            attack_uuid: self.attack_uuid,
            finished_successful: error.is_none(),
        })
        .await;

        if let Some(error) = error.as_ref() {
            error!("{error}");
        }

        if error.is_none() {
            if let Err(err) = update!(&self.db, Attack)
                .condition(Attack::F.uuid.equals(self.attack_uuid))
                .set(Attack::F.finished_at, Some(Utc::now()))
                .exec()
                .await
            {
                error!(
                    "Failed to set the attack {attack_uuid} to finished: {err}",
                    attack_uuid = self.attack_uuid
                );
            }
        }
    }

    async fn handle_streamed_response<T, Fut>(
        streamed_response: Result<Response<Streaming<T>>, Status>,
        handler: impl FnMut(T) -> Fut,
    ) -> Result<(), String>
    where
        Fut: TryFuture<Ok = (), Error = String>,
    {
        let stream = streamed_response
            .map_err(|status| format!("Failed getting stream: {status}"))?
            .into_inner();

        stream
            .map_err(|status| format!("Failed reading stream: {status}"))
            .try_for_each(handler)
            .await
    }
}
impl std::ops::Deref for LeechAttackContext {
    type Target = AttackContext;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}
impl std::ops::DerefMut for LeechAttackContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}
