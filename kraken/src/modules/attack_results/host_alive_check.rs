use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{and, insert, query, update};
use uuid::Uuid;

use crate::models::{
    AggregationSource, AggregationTable, AttackType, Host, HostAliveResultInsert, HostCertainty,
    HostInsert, OsType,
};

/// Store a host alive check's result and update the aggregated hosts
pub async fn store_host_alive_check_result(
    executor: impl Executor<'_>,
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    ip_addr: IpNetwork,
) -> Result<(), rorm::Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tx = guard.get_transaction();

    let result_uuid = insert!(&mut *tx, HostAliveResultInsert)
        .return_primary_key()
        .single(&HostAliveResultInsert {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
            host: ip_addr,
        })
        .await?;

    let aggregated_uuid = if let Some((host_uuid, certainty)) =
        query!(&mut *tx, (Host::F.uuid, Host::F.certainty))
            .condition(and!(
                Host::F.ip_addr.equals(ip_addr),
                Host::F.workspace.equals(workspace_uuid)
            ))
            .optional()
            .await?
    {
        if !matches!(certainty, HostCertainty::Verified) {
            update!(&mut *tx, Host)
                .set(Host::F.certainty, HostCertainty::Verified)
                .condition(Host::F.uuid.equals(host_uuid))
                .await?;
        }
        host_uuid
    } else {
        let host_uuid = Uuid::new_v4();
        insert!(&mut *tx, HostInsert)
            .return_nothing()
            .single(&HostInsert {
                uuid: host_uuid,
                ip_addr,
                os_type: OsType::Unknown,
                response_time: None,
                comment: String::new(),
                certainty: HostCertainty::Verified,
                workspace: ForeignModelByField::Key(workspace_uuid),
            })
            .await?;
        host_uuid
    };

    insert!(&mut *tx, AggregationSource)
        .return_nothing()
        .single(&AggregationSource {
            uuid: Uuid::new_v4(),
            workspace: ForeignModelByField::Key(workspace_uuid),
            result_type: AttackType::HostAlive,
            result_uuid,
            aggregated_table: AggregationTable::Host,
            aggregated_uuid,
        })
        .await?;

    guard.commit().await
}
