use std::net::IpAddr;

use ipnetwork::IpNetwork;
use kraken_proto::shared;
use kraken_proto::shared::OperatingSystem;
use kraken_proto::OsDetectionRequest;
use kraken_proto::OsDetectionResponse;
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
use crate::models::OsDetectionResultInsert;
use crate::models::OsType;
use crate::models::SourceType;
use crate::modules::attacks::AttackContext;
use crate::modules::attacks::AttackError;
use crate::modules::attacks::HandleAttackResponse;
use crate::modules::attacks::OsDetectionParams;

impl AttackContext {
    pub async fn os_detection(
        &self,
        mut leech: LeechClient,
        params: OsDetectionParams,
    ) -> Result<(), AttackError> {
        let targets =
            DomainOrNetwork::resolve(self.workspace.uuid, self.user.uuid, &leech, &params.targets)
                .await?;
        self.handle_streamed_response(
            leech.os_detection(OsDetectionRequest {
                targets: targets
                    .into_iter()
                    .map(shared::NetOrAddress::from)
                    .collect(),
                attack_uuid: self.attack_uuid.to_string(),
                fingerprint_port: params.fingerprint_port,
                ssh_port: params.ssh_port,
                fingerprint_timeout: params.fingerprint_timeout,
                ssh_connect_timeout: params.ssh_connect_timeout,
                ssh_timeout: params.ssh_timeout,
                port_ack_timeout: params.port_ack_timeout,
                port_parallel_syns: params.port_parallel_syns,
                concurrent_limit: params.concurrent_limit,
            }),
        )
        .await
    }
}

impl HandleAttackResponse<OsDetectionResponse> for AttackContext {
    async fn handle_response(&self, response: OsDetectionResponse) -> Result<(), AttackError> {
        let OsDetectionResponse {
            host: Some(host),
            hints,
            versions: version,
            os,
        } = response
        else {
            return Err(AttackError::Malformed("missing `host`"));
        };
        let os = shared::OperatingSystem::try_from(os)
            .map_err(|_| AttackError::Malformed("invalid `os`"))?;
        let host = IpAddr::try_from(host)?;
        let network = IpNetwork::from(host);

        // TODO: map what we find better
        let os = match os {
            OperatingSystem::Unknown => OsType::Unknown,
            OperatingSystem::Linux => OsType::Linux,
            OperatingSystem::Bsd => OsType::FreeBSD,
            OperatingSystem::Android => OsType::Android,
            OperatingSystem::Osx => OsType::Apple,
            OperatingSystem::Ios => OsType::Apple,
            OperatingSystem::Windows => OsType::Windows,
        };

        let hints = hints.join("\n");
        let version = version.join(" OR ");

        self.send_ws(WsMessage::OsDetectionResult {
            os,
            host,
            hints: hints.clone(),
            version: version.clone(),
        })
        .await;

        let mut tx = GLOBAL.db.start_transaction().await?;

        let result_uuid = insert!(&mut tx, OsDetectionResultInsert)
            .return_primary_key()
            .single(&OsDetectionResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                host: network,
                os,
                hints: hints.clone(),
                version: version.clone(),
            })
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host_os(self.workspace.uuid, network, HostCertainty::Verified, os)
            .await?;

        insert!(&mut tx, AggregationSource)
            .return_nothing()
            .single(&AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(self.workspace.uuid),
                source_type: SourceType::OSDetection,
                source_uuid: result_uuid,
                aggregated_table: AggregationTable::Host,
                aggregated_uuid: host_uuid,
            })
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
