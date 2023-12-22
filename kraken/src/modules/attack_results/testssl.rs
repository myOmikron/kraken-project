use std::str::FromStr;

use ipnetwork::IpNetwork;
use log::error;
use rorm::prelude::{ForeignModel, ForeignModelByField};
use rorm::{insert, Patch};
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::models::{
    AggregationSource, AggregationTable, Attack, HostCertainty, PortCertainty, PortProtocol,
    ServiceCertainty, SourceType, TestSSLResult,
};
use crate::rpc::rpc_definitions::TestSslScanResult;

/// Store a query certificate transparency's result and update the aggregated domains and hosts
pub async fn store_testssl_result(
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    result: TestSslScanResult,
) -> Result<(), rorm::Error> {
    // TODO

    let mut tx = GLOBAL.db.start_transaction().await?;

    let TestSslScanResult {
        target_host,
        ip,
        port,
        rdns,
        service,
        pretest,
        protocols,
        grease,
        ciphers,
        pfs,
        server_preferences,
        server_defaults,
        header_response,
        vulnerabilities,
        cipher_tests,
        browser_simulations,
    } = result;

    let ip = match IpNetwork::from_str(&ip) {
        Ok(ip) => ip,
        Err(err) => {
            error!("Testssl didn't return a valid ip: {err}");
            return Ok(());
        }
    };

    let port = match u16::from_str(&port) {
        Ok(port) => port,
        Err(err) => {
            error!("Testssl didn't return a valid port: {err}");
            return Ok(());
        }
    };

    let host_uuid = GLOBAL
        .aggregator
        .aggregate_host(workspace_uuid, ip, HostCertainty::Verified)
        .await?;

    let port_uuid = GLOBAL
        .aggregator
        .aggregate_port(
            workspace_uuid,
            host_uuid,
            port,
            PortProtocol::Tcp,
            PortCertainty::Verified,
        )
        .await?;

    let service_uuid = GLOBAL
        .aggregator
        .aggregate_service(
            workspace_uuid,
            host_uuid,
            Some(port_uuid),
            &service,
            ServiceCertainty::MaybeVerified, // TODO might be DefinitelyVerified?
        )
        .await?;

    let source_uuid = insert!(&mut tx, TestSSLResult)
        .return_primary_key()
        .single(&InsertTestSSLResult {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
        })
        .await?;

    insert!(&mut tx, AggregationSource)
        .return_nothing()
        .bulk([
            AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace_uuid),
                source_type: SourceType::TestSSL,
                source_uuid,
                aggregated_table: AggregationTable::Host,
                aggregated_uuid: host_uuid,
            },
            AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace_uuid),
                source_type: SourceType::TestSSL,
                source_uuid,
                aggregated_table: AggregationTable::Port,
                aggregated_uuid: port_uuid,
            },
            AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace_uuid),
                source_type: SourceType::TestSSL,
                source_uuid,
                aggregated_table: AggregationTable::Service,
                aggregated_uuid: service_uuid,
            },
        ])
        .await?;

    tx.commit().await
}

#[derive(Patch)]
#[rorm(model = "TestSSLResult")]
struct InsertTestSSLResult {
    uuid: Uuid,
    attack: ForeignModel<Attack>,
}
