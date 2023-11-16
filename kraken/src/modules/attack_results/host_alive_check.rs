use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::models::{
    AggregationSource, AggregationTable, Host, HostAliveResultInsert, HostCertainty, ResultType,
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

    let host_uuid =
        Host::aggregate(&mut *tx, workspace_uuid, ip_addr, HostCertainty::Verified).await?;

    insert!(&mut *tx, AggregationSource)
        .return_nothing()
        .single(&AggregationSource {
            uuid: Uuid::new_v4(),
            workspace: ForeignModelByField::Key(workspace_uuid),
            result_type: ResultType::HostAlive,
            result_uuid,
            aggregated_table: AggregationTable::Host,
            aggregated_uuid: host_uuid,
        })
        .await?;

    guard.commit().await
}
