use std::net::IpAddr;

use ipnetwork::IpNetwork;
use kraken_proto::{shared, PortOrRange, TcpPortScanRequest, TcpPortScanResponse};
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::{
    AggregationSource, AggregationTable, HostCertainty, PortCertainty, PortProtocol, SourceType,
    TcpPortScanResultInsert,
};
use crate::modules::attacks::{
    AttackContext, AttackError, DomainOrNetwork, HandleAttackResponse, TcpPortScanParams,
};

impl AttackContext {
    /// Executes the "tcp port scan" attack
    pub async fn tcp_port_scan(
        &self,
        mut leech: LeechClient,
        params: TcpPortScanParams,
    ) -> Result<(), AttackError> {
        let targets =
            DomainOrNetwork::resolve(self.workspace.uuid, self.user.uuid, &leech, &params.targets)
                .await?;
        let request = TcpPortScanRequest {
            attack_uuid: self.attack_uuid.to_string(),
            targets: targets
                .into_iter()
                .map(shared::NetOrAddress::from)
                .collect(),
            ports: params.ports.into_iter().map(PortOrRange::from).collect(),
            timeout: params.timeout,
            concurrent_limit: params.concurrent_limit,
            max_retries: params.max_retries,
            retry_interval: params.retry_interval,
            skip_icmp_check: params.skip_icmp_check,
        };
        self.handle_streamed_response(leech.run_tcp_port_scan(request))
            .await
    }
}
impl HandleAttackResponse<TcpPortScanResponse> for AttackContext {
    async fn handle_response(&self, response: TcpPortScanResponse) -> Result<(), AttackError> {
        let TcpPortScanResponse {
            address: Some(address),
            port,
        } = response
        else {
            return Err(AttackError::Malformed("Missing `address`"));
        };

        let address = IpAddr::try_from(address)?;

        self.send_ws(WsMessage::ScanTcpPortsResult {
            attack_uuid: self.attack_uuid,
            address: address.to_string(),
            port: port as u16,
        })
        .await;

        let mut tx = GLOBAL.db.start_transaction().await?;

        let address = IpNetwork::from(address);

        let result_uuid = insert!(&mut tx, TcpPortScanResultInsert)
            .return_primary_key()
            .single(&TcpPortScanResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                address,
                port: port as i32,
            })
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(self.workspace.uuid, address, HostCertainty::Verified)
            .await?;

        let port_uuid = GLOBAL
            .aggregator
            .aggregate_port(
                self.workspace.uuid,
                host_uuid,
                port as u16,
                PortProtocol::Tcp,
                PortCertainty::Verified,
            )
            .await?;

        insert!(&mut tx, AggregationSource)
            .return_nothing()
            .bulk(&[
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                    source_type: SourceType::TcpPortScan,
                    source_uuid: result_uuid,
                    aggregated_table: AggregationTable::Host,
                    aggregated_uuid: host_uuid,
                },
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                    source_type: SourceType::TcpPortScan,
                    source_uuid: result_uuid,
                    aggregated_table: AggregationTable::Port,
                    aggregated_uuid: port_uuid,
                },
            ])
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
