use dehashed_rs::{DehashedError, Query, ScheduledRequest, SearchResult};
use ipnetwork::IpNetwork;
use log::error;
use rorm::insert;
use rorm::prelude::*;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::models::DehashedQueryResultInsert;
use crate::modules::attacks::AttackContext;

impl AttackContext {
    /// Query the [dehashed](https://dehashed.com/) API.
    ///
    /// See [`handler::attacks::query_dehashed`] for more information.
    pub async fn query_dehashed(self, sender: mpsc::Sender<ScheduledRequest>, query: Query) {
        let (tx, rx) = oneshot::channel::<Result<SearchResult, DehashedError>>();

        if let Err(err) = sender.send(ScheduledRequest::new(query, tx)).await {
            error!("Couldn't send to dehashed scheduler: {err}");
            return;
        }

        let res = match rx.await {
            Err(err) => {
                error!("Error waiting for result: {err}");
                return;
            }
            Ok(Err(err)) => {
                error!("Error while using dehashed: {err}");
                return;
            }
            Ok(Ok(res)) => res,
        };

        let entries = res.entries.into_iter().map(|x| DehashedQueryResultInsert {
            uuid: Uuid::new_v4(),
            dehashed_id: x.id as i64,
            username: x.username,
            name: x.name,
            email: x.email,
            password: x.password,
            hashed_password: x.hashed_password,
            database_name: x.database_name,
            address: x.address,
            phone: x.phone,
            vin: x.vin,
            ip_address: x.ip_address.map(IpNetwork::from),
            attack: ForeignModelByField::Key(self.attack_uuid),
        });

        if let Err(err) = insert!(&self.db, DehashedQueryResultInsert)
            .bulk(entries)
            .await
        {
            error!("Database error: {err}");
            return;
        }

        self.set_finished(true).await;
    }
}
