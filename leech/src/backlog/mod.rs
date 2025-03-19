//! This modules handles all backlog tasks

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use futures::TryStreamExt;
use kraken_proto::any_attack_response;
use kraken_proto::backlog_service_client::BacklogServiceClient;
use kraken_proto::AnyAttackResponse;
use kraken_proto::BacklogRequest;
use log::debug;
use log::error;
use log::info;
use rorm::delete;
use rorm::insert;
use rorm::prelude::*;
use rorm::query;
use rorm::Database;
use tokio::sync::Notify;
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::Endpoint;
use tonic::Request;
use uuid::Uuid;

use crate::config::KRAKEN;
use crate::config::LEECH_SECRET;
use crate::models::BacklogEntry;
use crate::models::Proto;
use crate::utils::kraken_endpoint;

/// The main struct for the Backlog,
/// holds a connection to the database
#[derive(Clone)]
pub struct Backlog {
    db: Database,
    notify: Arc<Notify>,
}

impl Backlog {
    /// Store any attack's response to the backlog
    pub(crate) async fn store(&self, attack_uuid: Uuid, response: any_attack_response::Response) {
        let result = insert!(&self.db, BacklogEntry)
            .return_nothing()
            .single(&BacklogEntry {
                uuid: Uuid::new_v4(),
                attack_uuid,
                response: Proto(AnyAttackResponse {
                    attack_uuid: attack_uuid.to_string(),
                    response: Some(response),
                }),
            })
            .await;
        match result {
            Ok(_) => self.notify.notify_waiters(),
            Err(err) => {
                error!("Couldn't write backlog entry: {err}");
            }
        }
    }
}

const KRAKEN_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const DB_QUERY_PAUSE: Duration = Duration::from_secs(10);
const DB_QUERY_LIMIT: usize = 1000;

/// Starts the backlog upload server
pub async fn start_backlog(db: Database) -> Result<Backlog, Box<dyn Error>> {
    let secret = LEECH_SECRET.parse()?;
    let endpoint = kraken_endpoint(&*KRAKEN)?;
    let notify = Arc::new(Notify::new());

    tokio::spawn({
        let db = db.clone();
        let notify = notify.clone();
        async move {
            if let Err(err) = run_backlog(db, notify, secret, endpoint).await {
                error!("Backlog error: {err}");
            }
        }
    });

    Ok(Backlog { db, notify })
}

async fn run_backlog(
    db: Database,
    notify: Arc<Notify>,
    secret: AsciiMetadataValue,
    kraken_endpoint: Endpoint,
) -> Result<(), Box<dyn Error>> {
    loop {
        // Connect to kraken
        let mut kraken = loop {
            match kraken_endpoint.connect().await {
                Ok(chan) => {
                    let secret = secret.clone();
                    info!("connected to kraken @ {}", kraken_endpoint.uri());
                    break BacklogServiceClient::with_interceptor(
                        chan,
                        move |mut req: Request<()>| {
                            req.metadata_mut().insert("x-leech-secret", secret.clone());
                            Ok(req)
                        },
                    );
                }
                Err(_) => {
                    debug!(
                        "could not connect to kraken, retrying in {} minutes",
                        KRAKEN_RETRY_INTERVAL.as_secs() / 60
                    );
                    tokio::time::sleep(KRAKEN_RETRY_INTERVAL).await;
                    continue;
                }
            }
        };

        // Use transaction to lock the BacklogEntry table
        let mut tx = db.start_transaction().await?;

        // Send the entire BacklogEntry table to the kraken in chunks
        {
            let mut chunks = query!(&mut tx, (BacklogEntry::F.response,))
                .stream()
                .map_ok(|x| x.0 .0)
                .try_chunks(DB_QUERY_LIMIT);
            while let Some(responses) = chunks.try_next().await? {
                // TODO handle error
                let _ = kraken.submit_backlog(BacklogRequest { responses }).await;
            }
        }

        // Clear and unlock the BacklogEntry table
        delete!(&mut tx, BacklogEntry).all().await?;
        tx.commit().await?;

        // Wait for new entries
        notify.notified().await;

        // After receiving the first notification, wait a short duration
        // because it is likely for multiple backlog entries to be stored in a short period of time.
        tokio::time::sleep(DB_QUERY_PAUSE).await;
    }
}
