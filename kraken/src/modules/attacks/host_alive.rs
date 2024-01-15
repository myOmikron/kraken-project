use std::net::IpAddr;

use ipnetwork::IpNetwork;
use kraken_proto::{shared, HostsAliveRequest, HostsAliveResponse};
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::{
    AggregationSource, AggregationTable, HostAliveResultInsert, HostCertainty, SourceType,
};
use crate::modules::attacks::{
    AttackContext, AttackError, DomainOrNetwork, HandleAttackResponse, HostAliveParams,
};

impl AttackContext {
    /// Executes the "host alive" attack
    pub async fn host_alive(
        &self,
        mut leech: LeechClient,
        params: HostAliveParams,
    ) -> Result<(), AttackError> {
        let targets =
            DomainOrNetwork::resolve(self.workspace.uuid, self.user.uuid, &leech, &params.targets)
                .await?;
        let request = HostsAliveRequest {
            attack_uuid: self.attack_uuid.to_string(),
            targets: targets
                .into_iter()
                .map(shared::NetOrAddress::from)
                .collect(),
            timeout: params.timeout,
            concurrent_limit: params.concurrent_limit,
        };
        self.handle_streamed_response(leech.hosts_alive_check(request))
            .await
    }
}
impl HandleAttackResponse<HostsAliveResponse> for AttackContext {
    async fn handle_response(&self, response: HostsAliveResponse) -> Result<(), AttackError> {
        let HostsAliveResponse { host: Some(host) } = response else {
            return Err(AttackError::Malformed("Missing `host`"));
        };

        let host = IpAddr::try_from(host)?;
        self.send_ws(WsMessage::HostsAliveCheck {
            host,
            attack_uuid: self.attack_uuid,
        })
        .await;
        let host = IpNetwork::from(host);

        let mut tx = GLOBAL.db.start_transaction().await?;

        let result_uuid = insert!(&mut tx, HostAliveResultInsert)
            .return_primary_key()
            .single(&HostAliveResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                host,
            })
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(self.workspace.uuid, host, HostCertainty::Verified)
            .await?;

        insert!(&mut tx, AggregationSource)
            .return_nothing()
            .single(&AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(self.workspace.uuid),
                source_type: SourceType::HostAlive,
                source_uuid: result_uuid,
                aggregated_table: AggregationTable::Host,
                aggregated_uuid: host_uuid,
            })
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
