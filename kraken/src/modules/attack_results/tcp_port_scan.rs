use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::models::{
    AggregationSource, AggregationTable, Host, HostCertainty, Port, PortCertainty, PortProtocol,
    ResultType, TcpPortScanResultInsert,
};

/// Store a tcp port scan's result and update the aggregated hosts and ports
pub async fn store_tcp_port_scan_result(
    executor: impl Executor<'_>,
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    ip_addr: IpNetwork,
    port_num: u16,
) -> Result<(), rorm::Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tx = guard.get_transaction();

    let result_uuid = insert!(&mut *tx, TcpPortScanResultInsert)
        .return_primary_key()
        .single(&TcpPortScanResultInsert {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
            address: ip_addr,
            port: port_num as i32,
        })
        .await?;

    let host_uuid =
        Host::aggregate(&mut *tx, workspace_uuid, ip_addr, HostCertainty::Verified).await?;

    let port_uuid = Port::aggregate(
        &mut *tx,
        workspace_uuid,
        host_uuid,
        port_num,
        PortProtocol::Tcp,
        PortCertainty::Verified,
    )
    .await?;

    insert!(&mut *tx, AggregationSource)
        .return_nothing()
        .bulk(&[
            AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace_uuid),
                result_type: ResultType::TcpPortScan,
                result_uuid,
                aggregated_table: AggregationTable::Host,
                aggregated_uuid: host_uuid,
            },
            AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace_uuid),
                result_type: ResultType::TcpPortScan,
                result_uuid,
                aggregated_table: AggregationTable::Port,
                aggregated_uuid: port_uuid,
            },
        ])
        .await?;

    guard.commit().await
}
