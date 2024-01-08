use ipnetwork::IpNetwork;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::models::{
    AggregationSource, AggregationTable, HostCertainty, PortCertainty, PortProtocol,
    ServiceCertainty, ServiceDetectionName, ServiceDetectionResultInsert, SourceType,
};
use crate::modules::attacks::{AttackContext, AttackError, ServiceDetectionParams};
use crate::rpc::rpc_definitions::{ServiceDetectionRequest, ServiceDetectionResponse};

impl AttackContext {
    /// Executes the "service detection" attack
    pub async fn service_detection(
        &self,
        mut leech: LeechClient,
        params: ServiceDetectionParams,
    ) -> Result<(), AttackError> {
        let request = ServiceDetectionRequest {
            attack_uuid: self.attack_uuid.to_string(),
            address: Some(params.target.into()),
            port: params.port as u32,
            timeout: params.timeout,
        };
        let ServiceDetectionResponse {
            services,
            response_type,
        } = leech.service_detection(request).await?.into_inner();

        let certainty = match response_type {
            1 => ServiceCertainty::MaybeVerified,
            2 => ServiceCertainty::DefinitelyVerified,
            _ => {
                return Err(AttackError::Custom("Retrieved certainty Unknown".into()));
            }
        };

        let mut tx = GLOBAL.db.start_transaction().await?;

        let result_uuid = insert!(&mut tx, ServiceDetectionResultInsert)
            .return_primary_key()
            .single(&ServiceDetectionResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                certainty,
                host: IpNetwork::from(params.target),
                port: params.port as i32,
            })
            .await?;
        insert!(&mut tx, ServiceDetectionName)
            .return_nothing()
            .bulk(services.iter().map(|x| ServiceDetectionName {
                uuid: Uuid::new_v4(),
                name: x.to_string(),
                result: ForeignModelByField::Key(result_uuid),
            }))
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(
                self.workspace.uuid,
                IpNetwork::from(params.target),
                HostCertainty::Verified,
            )
            .await?;
        let port_uuid = GLOBAL
            .aggregator
            .aggregate_port(
                self.workspace.uuid,
                host_uuid,
                params.port as u16,
                PortProtocol::Tcp,
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
                        &service,
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
                        source_type: SourceType::ServiceDetection,
                        source_uuid: result_uuid,
                        aggregated_table: AggregationTable::Host,
                        aggregated_uuid: host_uuid,
                    },
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(self.workspace.uuid),
                        source_type: SourceType::ServiceDetection,
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
                            source_type: SourceType::ServiceDetection,
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
