//! This modules manages the grpc connections to the leeches

mod errors;

use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

use actix_web::web::Data;
use log::{debug, error, warn};
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use rorm::prelude::*;
use rorm::{query, Database};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

pub use self::errors::*;
use crate::models::Leech;
use crate::rpc::rpc_definitions::req_attack_service_client::ReqAttackServiceClient;

/// Handle for interacting with the leech manager
#[derive(Debug)]
pub struct LeechManager {
    /// Channel for sending command to the managing task
    sender: mpsc::Sender<LeechManagerEvent>,

    /// Collection of all connected clients
    clients: RwLock<HashMap<Uuid, LeechClient>>,
}

/// Rpc client for sending attack requests to the leech
pub type LeechClient = ReqAttackServiceClient<Channel>;

impl LeechManager {
    /// Starts a new leech manager and returns a handle to it
    ///
    /// ## Errors
    /// if the leeches currently in the database couldn't be queried.
    pub async fn start(db: Database) -> Result<Data<Self>, rorm::Error> {
        let initial_leeches = query!(&db, Leech).all().await?;

        let (sender, receiver) = mpsc::channel(16);

        let handle = Data::new(Self {
            sender,
            clients: RwLock::new(HashMap::new()),
        });

        tokio::spawn(
            handle
                .clone()
                .into_inner()
                .run(db, receiver, initial_leeches),
        );

        Ok(handle)
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

    /// Signals the leech manager, that a new leech has been created
    ///
    /// This method has to be called after modifying the database
    pub async fn created_leech(&self, uuid: Uuid) {
        self.send(LeechManagerEvent::Created(uuid)).await;
    }

    /// Signals the leech manager, that an existing leech has been updated
    ///
    /// This method has to be called after modifying the database
    pub async fn updated_leech(&self, uuid: Uuid) {
        self.send(LeechManagerEvent::Updated(uuid)).await;
    }

    /// Signals the leech manager, that an existing leech has been deleted
    pub async fn deleted_leech(&self, uuid: Uuid) {
        self.send(LeechManagerEvent::Deleted(uuid)).await;
    }
}

/**********************************/
/* Private implementation details */
/**********************************/

enum LeechManagerEvent {
    Created(Uuid),
    Updated(Uuid),
    Deleted(Uuid),
}

impl LeechManager {
    /// Lock the `clients` for reading
    ///
    /// **Don't panic while holding this lock**
    fn read(&self) -> RwLockReadGuard<'_, HashMap<Uuid, LeechClient>> {
        match self.clients.read() {
            Ok(guard) => guard,
            Err(poison) => {
                error!("The leech manager's lock has been poisoned! This should never happen!");
                poison.into_inner()
            }
        }
    }

    /// Lock the `clients` for writing
    ///
    /// **Don't panic while holding this lock**
    fn write(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, LeechClient>> {
        match self.clients.write() {
            Ok(guard) => guard,
            Err(poison) => {
                error!("The leech manager's lock has been poisoned! This should never happen!");
                poison.into_inner()
            }
        }
    }

    /// Send an event to the manager task
    async fn send(&self, event: LeechManagerEvent) {
        if self.sender.send(event).await.is_err() {
            error!("The leech manager died! This should never happen!");
        }
    }

    /// Runs the actual manager task in an endless loop
    async fn run(
        self: Arc<Self>,
        db: Database,
        mut receiver: mpsc::Receiver<LeechManagerEvent>,
        initial_leeches: Vec<Leech>,
    ) -> Infallible {
        let mut client_join_handles: HashMap<Uuid, JoinHandle<()>> = HashMap::new();

        for leech in initial_leeches {
            let leech_uuid = leech.uuid;
            debug!("Spawning rpc client loop for {leech_uuid}");
            let join_handle = tokio::spawn(connect_to_leech(leech, self.clone()));
            client_join_handles.insert(leech_uuid, join_handle);
        }

        loop {
            let Some(event) = receiver.recv().await else {
                // `recv` returns `None` iff the channel is closed which won't happen:
                // This function never calls `close` on the receiver and owns a sender.
                unreachable!("The leech manager died! This should never happen!");
            };

            match event {
                LeechManagerEvent::Created(uuid) => {
                    if let Ok(Some(leech)) = query!(&db, Leech)
                        .condition(Leech::F.uuid.equals(uuid))
                        .optional()
                        .await
                    {
                        debug!("Starting rpc client loop for {uuid}");
                        let join_handle = tokio::spawn(connect_to_leech(leech, self.clone()));
                        client_join_handles.insert(uuid, join_handle);
                    }
                }
                LeechManagerEvent::Updated(uuid) => {
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
                            *join_handle = tokio::spawn(connect_to_leech(leech, self.clone()));
                        }
                    }
                }
                LeechManagerEvent::Deleted(id) => {
                    if let Some(join_handle) = client_join_handles.remove(&id) {
                        // TODO: Graceful shutdown instead of killing
                        debug!("Stopping rpc client loop for {id}");
                        join_handle.abort();
                    }
                }
            }
        }
    }
}

/// Connect to the leech (retry if necessary) and add it to the manager upon established connection.
async fn connect_to_leech(leech: Leech, handle: Arc<LeechManager>) {
    const CLIENT_RETRY_INTERVAL: Duration = Duration::from_secs(10);

    let endpoint = match Endpoint::from_str(&leech.address) {
        Ok(endpoint) => endpoint,
        Err(err) => {
            warn!(
                "Invalid leech address for leech {}: {}: {err}",
                leech.uuid, leech.address
            );
            return;
        }
    };

    let client = loop {
        match endpoint.connect().await {
            Ok(connection) => {
                break ReqAttackServiceClient::new(connection);
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
    };

    handle.write().insert(leech.uuid, client);
}
