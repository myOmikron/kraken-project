use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use actix_web::web::Data;
use log::{debug, warn};
use rorm::{query, Database, Model};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint};

use crate::models::Leech;
use crate::rpc::rpc_attacks::req_attack_service_client::ReqAttackServiceClient;

pub(crate) type RpcManagerChannel = Sender<RpcManagerEvent>;
pub(crate) type RpcClients = Data<RwLock<HashMap<i64, ReqAttackServiceClient<Channel>>>>;

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
                leech.id, leech.address
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
                    leech.id,
                    CLIENT_RETRY_INTERVAL.as_secs()
                );
            }
        }

        sleep(CLIENT_RETRY_INTERVAL).await;
    }

    let client = ReqAttackServiceClient::new(chan);

    let mut write = rpc_clients.write().await;
    write.insert(leech.id, client);
}

/**
Events for the RpcManager.

Make sure to commit any pending database state regarding the event
as the RpcManager must be able to retrieve the new state.
 */
pub enum RpcManagerEvent {
    /// Leech got deleted.
    Deleted(i64),
    /// Leech got created.
    Created(i64),
    /// Leech got updated.
    Updated(i64),
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

    let rpc_clients: RpcClients = Data::new(RwLock::new(HashMap::new()));

    let clients = rpc_clients.clone();
    tokio::spawn(async move {
        let mut client_join_handles: HashMap<i64, JoinHandle<()>> = HashMap::new();

        for leech in leeches {
            let leech_id = leech.id;
            debug!("Spawning rpc client loop for {leech_id}");
            let join_handle = tokio::spawn(rpc_client_loop(leech, clients.clone()));
            client_join_handles.insert(leech_id, join_handle);
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
                RpcManagerEvent::Created(id) => {
                    if let Ok(Some(leech)) = query!(&db, Leech)
                        .condition(Leech::F.id.equals(id))
                        .optional()
                        .await
                    {
                        debug!("Starting rpc client loop for {id}");
                        let join_handle = tokio::spawn(rpc_client_loop(leech, clients.clone()));
                        client_join_handles.insert(id, join_handle);
                    }
                }
                RpcManagerEvent::Updated(id) => {
                    if let Some(join_handle) = client_join_handles.get_mut(&id) {
                        // TODO: Graceful shutdown instead of killing
                        debug!("Stopping rpc client loop for {id}");
                        join_handle.abort();

                        if let Ok(Some(leech)) = query!(&db, Leech)
                            .condition(Leech::F.id.equals(id))
                            .optional()
                            .await
                        {
                            debug!("Starting rpc client loop for {id}");
                            *join_handle = tokio::spawn(rpc_client_loop(leech, clients.clone()));
                        }
                    }
                }
            }
        }
    });

    Ok((tx, rpc_clients))
}
