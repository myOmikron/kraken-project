use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use log::{debug, error, warn};
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use rorm::{query, Database, FieldAccess, Model};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use crate::api::handler::ApiError;
use crate::models::Leech;
use crate::rpc::rpc_definitions::req_attack_service_client::ReqAttackServiceClient;

pub(crate) type RpcManagerChannel = Sender<RpcManagerEvent>;

/// Synchronized map of all connected leeches
///
/// ## actix
/// This type behaves like a [`Data<_>`]
///
/// i.e. pass it to `App::app_data` and directly access it in your handlers
#[derive(Debug, Clone, Default)]
pub struct RpcClients(Arc<RwLock<HashMap<Uuid, LeechClient>>>);

/// Rpc client for sending attack requests to the leech
pub type LeechClient = ReqAttackServiceClient<Channel>;

impl RpcClients {
    /// Create an empty map
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve a concrete leech by its id.
    ///
    /// You might want a concrete leech instead of a random one,
    /// when it is deployed in a specific environment for example a private network
    pub fn get_leech(&self, uuid: &Uuid) -> Result<LeechClient, InvalidLeech> {
        self.read().get(uuid).cloned().ok_or(InvalidLeech)
    }

    /// Retrieve a random leech
    pub fn random_leech(&self) -> Result<LeechClient, NoLeechAvailable> {
        self.read()
            .iter()
            .choose(&mut thread_rng())
            .map(|(_, leech)| leech.clone())
            .ok_or(NoLeechAvailable)
    }

    fn read(&self) -> RwLockReadGuard<'_, HashMap<Uuid, LeechClient>> {
        match self.0.read() {
            Ok(guard) => guard,
            Err(poison) => {
                error!("The RpcClients' lock has been poisoned! This should never happen!");
                poison.into_inner()
            }
        }
    }

    fn write(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, LeechClient>> {
        match self.0.write() {
            Ok(guard) => guard,
            Err(poison) => {
                error!("The RpcClients' lock has been poisoned! This should never happen!");
                poison.into_inner()
            }
        }
    }
}

/// The error returned by [`RpcClients::random_leech`] which can be converted into [`ApiError::NoLeechAvailable`]
#[derive(Debug, Error)]
pub struct NoLeechAvailable;
impl From<NoLeechAvailable> for ApiError {
    fn from(_: NoLeechAvailable) -> Self {
        ApiError::NoLeechAvailable
    }
}
impl fmt::Display for NoLeechAvailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ApiError::NoLeechAvailable.fmt(f)
    }
}

/// The error returned by [`RpcClients::get_leech`] which can be converted into [`ApiError::InvalidLeech`]
#[derive(Debug, Error)]
pub struct InvalidLeech;
impl From<InvalidLeech> for ApiError {
    fn from(_: InvalidLeech) -> Self {
        ApiError::InvalidLeech
    }
}
impl fmt::Display for InvalidLeech {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ApiError::InvalidLeech.fmt(f)
    }
}

impl FromRequest for RpcClients {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, actix_web::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(data) = req.app_data::<Self>() {
            ready(Ok(data.clone()))
        } else {
            debug!(
                "Failed to extract `RpcClients` for `{}` handler. \
                Please pass your `RpcClients` instance to `App::app_data()` and \
                don't wrap it into a `Data<RpcClients>`",
                req.match_name().unwrap_or_else(|| req.path())
            );

            ready(Err(actix_web::error::ErrorInternalServerError(
                "Requested application data is not configured correctly. \
                View/enable debug logs for more details.",
            )))
        }
    }
}

const CLIENT_RETRY_INTERVAL: Duration = Duration::from_secs(10);

/**
Starts the rpc connection to a leech.

**Parameter**:
- `leech`: [Leech]: Instance of a leech
- `rpc_clients`: [RpcClients]
 */
pub async fn rpc_client_loop(leech: Leech, rpc_clients: RpcClients) {
    let endpoint = match Endpoint::from_str(&leech.address) {
        Ok(v) => v,
        Err(err) => {
            warn!(
                "Invalid leech address for leech {}: {}: {err}",
                leech.uuid, leech.address
            );

            return;
        }
    };

    let chan;
    loop {
        match endpoint.connect().await {
            Ok(c) => {
                chan = c;
                break;
            }
            Err(err) => {
                warn!(
                    "Couldn't connect to leech {}: {err}. Retrying in {} seconds.",
                    leech.uuid,
                    CLIENT_RETRY_INTERVAL.as_secs()
                );
            }
        }

        sleep(CLIENT_RETRY_INTERVAL).await;
    }

    let client = ReqAttackServiceClient::new(chan);

    rpc_clients.write().insert(leech.uuid, client);
}

/**
Events for the RpcManager.

Make sure to commit any pending database state regarding the event
as the RpcManager must be able to retrieve the new state.
 */
pub enum RpcManagerEvent {
    /// Leech got deleted.
    Deleted(Uuid),
    /// Leech got created.
    Created(Uuid),
    /// Leech got updated.
    Updated(Uuid),
}

/**
Start the event loop to manage the rpc client connections.

Returns an channel to push events to.

**Parameter**:
- `db`: [Database]: Instance of the database
 */
pub async fn start_rpc_manager(db: Database) -> Result<(RpcManagerChannel, RpcClients), String> {
    let (tx, mut rx) = mpsc::channel(16);

    let leeches = query!(&db, Leech)
        .all()
        .await
        .map_err(|e| format!("Error while querying leeches: {e}"))?;

    let rpc_clients = RpcClients::new();

    let clients = rpc_clients.clone();
    tokio::spawn(async move {
        let mut client_join_handles: HashMap<Uuid, JoinHandle<()>> = HashMap::new();

        for leech in leeches {
            let leech_uuid = leech.uuid;
            debug!("Spawning rpc client loop for {leech_uuid}");
            let join_handle = tokio::spawn(rpc_client_loop(leech, clients.clone()));
            client_join_handles.insert(leech_uuid, join_handle);
        }

        while let Some(event) = rx.recv().await {
            match event {
                RpcManagerEvent::Deleted(id) => {
                    if let Some(join_handle) = client_join_handles.remove(&id) {
                        // TODO: Graceful shutdown instead of killing
                        debug!("Stopping rpc client loop for {id}");
                        join_handle.abort();
                    }
                }
                RpcManagerEvent::Created(uuid) => {
                    if let Ok(Some(leech)) = query!(&db, Leech)
                        .condition(Leech::F.uuid.equals(uuid))
                        .optional()
                        .await
                    {
                        debug!("Starting rpc client loop for {uuid}");
                        let join_handle = tokio::spawn(rpc_client_loop(leech, clients.clone()));
                        client_join_handles.insert(uuid, join_handle);
                    }
                }
                RpcManagerEvent::Updated(uuid) => {
                    if let Some(join_handle) = client_join_handles.get_mut(&uuid) {
                        // TODO: Graceful shutdown instead of killing
                        debug!("Stopping rpc client loop for {uuid}");
                        join_handle.abort();

                        if let Ok(Some(leech)) = query!(&db, Leech)
                            .condition(Leech::F.uuid.equals(uuid))
                            .optional()
                            .await
                        {
                            debug!("Starting rpc client loop for {uuid}");
                            *join_handle = tokio::spawn(rpc_client_loop(leech, clients.clone()));
                        }
                    }
                }
            }
        }
    });

    Ok((tx, rpc_clients))
}
