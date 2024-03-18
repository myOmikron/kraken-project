use std::net::IpAddr;

use ipnetwork::IpNetwork;
use kraken_proto::shared;
use kraken_proto::PortOrRange;
use kraken_proto::UdpServiceDetectionRequest;
use kraken_proto::UdpServiceDetectionResponse;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::api::handler::attacks::schema::DomainOrNetwork;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::HostCertainty;
use crate::models::PortCertainty;
use crate::models::PortProtocol;
use crate::models::ServiceCertainty;
use crate::models::ServiceProtocols;
use crate::models::SourceType;
use crate::models::UdpServiceDetectionName;
use crate::models::UdpServiceDetectionResultInsert;
use crate::modules::attacks::AttackContext;
use crate::modules::attacks::AttackError;
use crate::modules::attacks::HandleAttackResponse;
use crate::modules::attacks::UdpServiceDetectionParams;

impl AttackContext {
    /// Executes the "service detection" attack
    pub async fn udp_service_detection(
        &self,
        mut leech: LeechClient,
        params: UdpServiceDetectionParams,
    ) -> Result<(), AttackError> {
        let targets =
            DomainOrNetwork::resolve(self.workspace.uuid, self.user.uuid, &leech, &params.targets)
                .await?;
        let request = UdpServiceDetectionRequest {
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
        };
        self.handle_streamed_response(leech.udp_service_detection(request))
            .await
    }
}

impl HandleAttackResponse<UdpServiceDetectionResponse> for AttackContext {
    async fn handle_response(
        &self,
        response: UdpServiceDetectionResponse,
    ) -> Result<(), AttackError> {
        let UdpServiceDetectionResponse {
            address: Some(address),
            services,
            certainty,
            port,
        } = response
        else {
            return Err(AttackError::Malformed("Missing `address`"));
        };

        let address = IpAddr::try_from(address)?;
        let host = IpNetwork::from(address);

        let certainty = match certainty {
            0 => ServiceCertainty::UnknownService,
            1 => ServiceCertainty::MaybeVerified,
            2 => ServiceCertainty::DefinitelyVerified,
            _ => {
                return Err(AttackError::Custom("Received unknown certainty".into()));
            }
        };

        self.send_ws(WsMessage::UdpServiceDetectionResult {
            attack_uuid: self.attack_uuid,
            address: address.to_string(),
            port: port as u16,
            services: services.iter().map(|x| x.name.to_string()).collect(),
        })
        .await;

        let mut tx = GLOBAL.db.start_transaction().await?;

        let result_uuid = insert!(&mut tx, UdpServiceDetectionResultInsert)
            .return_primary_key()
            .single(&UdpServiceDetectionResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                certainty,
                host,
                port: port as i32,
            })
            .await?;

        if !services.is_empty() {
            insert!(&mut tx, UdpServiceDetectionName)
                .return_nothing()
                .bulk(services.iter().map(|x| UdpServiceDetectionName {
                    uuid: Uuid::new_v4(),
                    name: x.name.to_string(),
                    result: ForeignModelByField::Key(result_uuid),
                }))
                .await?;
        }

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(self.workspace.uuid, host, HostCertainty::Verified)
            .await?;
        let port_uuid = GLOBAL
            .aggregator
            .aggregate_port(
                self.workspace.uuid,
                host_uuid,
                port as u16,
                PortProtocol::Udp,
                PortCertainty::Verified,
            )
            .await?;

        let mut service_uuids = Vec::new();
        for service in services {
            service_uuids.push(
                GLOBAL
                    .aggregator
                    .aggregate_service(
                        self.workspace.uuid,
                        host_uuid,
                        Some(port_uuid),
                        Some(ServiceProtocols::Udp { raw: service.udp }),
                        &service.name,
                        certainty,
                    )
                    .await?,
            );
        }

        insert!(&mut tx, AggregationSource)
            .return_nothing()
            .bulk(
                [
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(self.workspace.uuid),
                        source_type: SourceType::UdpServiceDetection,
                        source_uuid: result_uuid,
                        aggregated_table: AggregationTable::Host,
                        aggregated_uuid: host_uuid,
                    },
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(self.workspace.uuid),
                        source_type: SourceType::UdpServiceDetection,
                        source_uuid: result_uuid,
                        aggregated_table: AggregationTable::Port,
                        aggregated_uuid: port_uuid,
                    },
                ]
                .into_iter()
                .chain(
                    service_uuids
                        .into_iter()
                        .map(|service_uuid| AggregationSource {
                            uuid: Uuid::new_v4(),
                            workspace: ForeignModelByField::Key(self.workspace.uuid),
                            source_type: SourceType::UdpServiceDetection,
                            source_uuid: result_uuid,
                            aggregated_table: AggregationTable::Service,
                            aggregated_uuid: service_uuid,
                        }),
                ),
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
