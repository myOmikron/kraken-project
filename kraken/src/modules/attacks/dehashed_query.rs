use dehashed_rs::DehashedError;
use dehashed_rs::ScheduledRequest;
use dehashed_rs::SearchResult;
use ipnetwork::IpNetwork;
use rorm::insert;
use rorm::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::models::DehashedQueryResultInsert;
use crate::modules::attacks::AttackContext;
use crate::modules::attacks::AttackError;
use crate::modules::attacks::DehashedQueryParams;

impl AttackContext {
    /// Executes the "dehashed query" attack
    pub async fn dehashed_query(
        &mut self,
        sender: mpsc::Sender<ScheduledRequest>,
        params: DehashedQueryParams,
    ) -> Result<(), AttackError> {
        let (tx, rx) = oneshot::channel::<Result<SearchResult, DehashedError>>();

        sender
            .send(ScheduledRequest::new(params.query, tx))
            .await
            .map_err(|_| AttackError::Custom("Couldn't send to dehashed scheduler".into()))?;

        let response = rx
            .await
            .map_err(|err| AttackError::Custom(err.into()))?
            .map_err(|err| AttackError::Custom(err.into()))?;

        let entries = response
            .entries
            .into_iter()
            .map(|x| DehashedQueryResultInsert {
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

        insert!(&GLOBAL.db, DehashedQueryResultInsert)
            .return_nothing()
            .bulk(entries)
            .await?;
        Ok(())
    }
}
