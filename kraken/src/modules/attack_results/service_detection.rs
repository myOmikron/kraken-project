use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::models::{
    AggregationSource, AggregationTable, AttackType, Host, HostCertainty, Port, PortCertainty,
    PortProtocol, Service, ServiceCertainty, ServiceDetectionName, ServiceDetectionResultInsert,
};

/// Store a service detection's result and update the aggregated hosts, ports and services
pub async fn store_service_detection_result(
    executor: impl Executor<'_>,
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    service_names: &[String],
    certainty: ServiceCertainty,
    host: IpNetwork,
    port: u16,
) -> Result<(), rorm::Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tx = guard.get_transaction();

    let result_uuid = insert!(&mut *tx, ServiceDetectionResultInsert)
        .return_primary_key()
        .single(&ServiceDetectionResultInsert {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
            certainty,
            host,
            port: i16::from_ne_bytes(port.to_ne_bytes()),
        })
        .await?;
    insert!(&mut *tx, ServiceDetectionName)
        .return_nothing()
        .bulk(service_names.iter().map(|x| ServiceDetectionName {
            uuid: Uuid::new_v4(),
            name: x.to_string(),
            result: ForeignModelByField::Key(result_uuid),
        }))
        .await?;

    let host_uuid =
        Host::aggregate(&mut *tx, workspace_uuid, host, HostCertainty::Verified).await?;
    let port_uuid = Port::aggregate(
        &mut *tx,
        workspace_uuid,
        host_uuid,
        port,
        PortProtocol::Tcp,
        PortCertainty::Verified,
    )
    .await?;

    let mut service_uuids = Vec::new();
    for service in service_names {
        service_uuids.push(
            Service::aggregate(
                &mut *tx,
                workspace_uuid,
                host_uuid,
                Some(port_uuid),
                service,
                certainty,
            )
            .await?,
        );
    }

    insert!(&mut *tx, AggregationSource)
        .return_nothing()
        .bulk(
            [
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace_uuid),
                    result_type: AttackType::ServiceDetection,
                    result_uuid,
                    aggregated_table: AggregationTable::Host,
                    aggregated_uuid: host_uuid,
                },
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace_uuid),
                    result_type: AttackType::ServiceDetection,
                    result_uuid,
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
                        workspace: ForeignModelByField::Key(workspace_uuid),
                        result_type: AttackType::ServiceDetection,
                        result_uuid,
                        aggregated_table: AggregationTable::Service,
                        aggregated_uuid: service_uuid,
                    }),
            ),
        )
        .await?;

    guard.commit().await
}
