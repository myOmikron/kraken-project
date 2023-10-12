//! This module implements all attacks as tasks to be spawned with `tokio::spawn`
//!
//! To start any attack create an [`AttackContext`] ([give it a leech](AttackContext::leech))
//! and call your desired attack method.

mod bruteforce_subdomains;
mod dns_resolution;
mod host_alive_check;
mod query_certificate_transparency;
mod query_dehashed;
mod service_detection;
mod tcp_port_scan;

use std::error::Error as StdError;
use std::fmt;

use chrono::Utc;
use futures::{TryFuture, TryStreamExt};
use log::error;
use rorm::prelude::*;
use rorm::{update, Database};
use thiserror::Error;
use tonic::{Response, Status, Streaming};
use uuid::Uuid;

#[cfg(doc)]
use crate::api::handler;
use crate::chan::{LeechClient, WsManagerChan, WsManagerMessage, WsMessage};
use crate::models::Attack;
use crate::rpc::rpc_definitions::AddressConvError;

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
    async fn set_finished(&self, error: Option<AttackError>) {
        self.send_ws(WsMessage::AttackFinished {
            attack_uuid: self.attack_uuid,
            finished_successful: error.is_none(),
        })
        .await;

        if let Some(error) = error.as_ref() {
            error!(
                "Attack {attack_uuid} failed: {error}",
                attack_uuid = self.attack_uuid
            );
        }

        if let Err(err) = update!(&self.db, Attack)
            .condition(Attack::F.uuid.equals(self.attack_uuid))
            .set(Attack::F.finished_at, Some(Utc::now()))
            .set(
                Attack::F.error,
                error.map(|err| {
                    let mut string = err.to_string();
                    for (char_index, (byte_index, _)) in string.char_indices().enumerate() {
                        if char_index == 256 {
                            string.truncate(byte_index);
                            break;
                        }
                    }
                    string
                }),
            )
            .exec()
            .await
        {
            error!(
                "Failed to set the attack {attack_uuid} to finished: {err}",
                attack_uuid = self.attack_uuid
            );
        }
    }

    async fn handle_streamed_response<T, Fut>(
        streamed_response: Result<Response<Streaming<T>>, Status>,
        handler: impl FnMut(T) -> Fut,
    ) -> Result<(), AttackError>
    where
        Fut: TryFuture<Ok = (), Error = AttackError>,
    {
        let stream = streamed_response?.into_inner();

        stream
            .map_err(AttackError::from)
            .try_for_each(handler)
            .await
    }
}

/// An error occurring during an attack which is logged and stored on the db
#[derive(Error, Debug)]
pub enum AttackError {
    /// An error returned by grpc i.e. a [`Status`]
    Grpc(#[from] Status),

    /// An error produced by the database
    Database(#[from] rorm::Error),

    /// A malformed grpc message
    ///
    /// For example "optional" fields which have to be set
    Malformed(&'static str),

    /// An error produced by address conversion
    AddressConv(#[from] AddressConvError),

    /// Catch all variant for everything else
    Custom(Box<dyn StdError + Send + Sync>),
}
impl fmt::Display for AttackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttackError::Grpc(status) => write!(
                f,
                "GRPC: {code:?}, {msg:?}",
                code = status.code(),
                msg = status.message()
            ),
            AttackError::Database(err) => write!(f, "DB: {err}"),
            AttackError::Malformed(err) => write!(f, "Malformed response: {err}"),
            AttackError::AddressConv(err) => write!(f, "Error during address conversion: {err}"),
            AttackError::Custom(err) => write!(f, "{err}"),
        }
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
