use std::iter;
use std::net::IpAddr;

use ipnetwork::IpNetwork;
use kraken_proto::shared;
use kraken_proto::PortOrRange;
use kraken_proto::ServiceDetectionRequest;
use kraken_proto::ServiceDetectionResponse;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::api::handler::attacks::schema::DomainOrNetwork;
use crate::api::handler::services::schema::ServiceProtocols;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::HostCertainty;
use crate::models::PortCertainty;
use crate::models::PortProtocol;
use crate::models::ServiceCertainty;
use crate::models::ServiceDetectionName;
use crate::models::ServiceDetectionResultInsert;
use crate::models::SourceType;
use crate::modules::attacks::AttackContext;
use crate::modules::attacks::AttackError;
use crate::modules::attacks::HandleAttackResponse;
use crate::modules::attacks::ServiceDetectionParams;
use crate::modules::finding_factory::schema::FindingFactoryIdentifier;

impl AttackContext {
    /// Executes the "service detection" attack
    pub async fn service_detection(
        &mut self,
        mut leech: LeechClient,
        params: ServiceDetectionParams,
    ) -> Result<(), AttackError> {
        let targets =
            DomainOrNetwork::resolve(self.workspace.uuid, self.user.uuid, &leech, &params.targets)
                .await?;
        let request = ServiceDetectionRequest {
            attack_uuid: self.attack_uuid.to_string(),
            targets: targets
                .into_iter()
                .map(shared::NetOrAddress::from)
                .collect(),
            ports: params.ports.into_iter().map(PortOrRange::from).collect(),
            connect_timeout: params.connect_timeout,
            receive_timeout: params.receive_timeout,
            concurrent_limit: params.concurrent_limit,
            max_retries: params.max_retries,
            retry_interval: params.retry_interval,
            skip_icmp_check: params.skip_icmp_check,
        };
        self.handle_streamed_response(leech.service_detection(request))
            .await
    }
}

impl HandleAttackResponse<ServiceDetectionResponse> for AttackContext {
    async fn handle_response(
        &mut self,
        response: ServiceDetectionResponse,
    ) -> Result<(), AttackError> {
        let ServiceDetectionResponse {
            address: Some(address),
            port,
            tcp_certainty,
            tcp_services,
            is_tls,
            tls_certainty,
            tls_services,
        } = response
        else {
            return Err(AttackError::Malformed("Missing `address`"));
        };

        // Basic conversions
        let address = IpAddr::try_from(address)?;
        let host = IpNetwork::from(address);
        let tcp_certainty = parse_service_certainty(tcp_certainty)?;
        let tls_certainty = parse_service_certainty(tls_certainty)?;

        // Preprocess list of services:
        // Combine `tcp_services` and `tls_services` into one list while merging matches between the two
        // The uuid is populated later when the aggregations are updated
        let mut services = Vec::new();
        for name in tcp_services.iter().cloned() {
            services.push((Uuid::nil(), name, true, false, tcp_certainty));
        }
        if is_tls {
            if tls_certainty == ServiceCertainty::UnknownService {
                services.push((
                    Uuid::nil(),
                    "unknown".to_string(),
                    false,
                    true,
                    ServiceCertainty::MaybeVerified,
                ));
            } else if tls_certainty == tcp_certainty {
                for name in tls_services.iter() {
                    if let Some(service) = services
                        .iter_mut()
                        .find(|(_, tcp_name, _, _, _)| tcp_name == name)
                    {
                        service.2 = true;
                    } else {
                        services.push((Uuid::nil(), name.clone(), false, true, tls_certainty));
                    }
                }
            } else {
                for name in tls_services.iter().cloned() {
                    services.push((Uuid::nil(), name, false, true, tls_certainty));
                }
            }
        }

        let mut tx = GLOBAL.db.start_transaction().await?;

        // Insert the two raw results
        let mut tcp_result_uuid = Uuid::nil();
        let mut tls_result_uuid = Uuid::nil();
        for (certainty, services, result_uuid) in
            iter::once((tcp_certainty, tcp_services, &mut tcp_result_uuid))
                .chain(is_tls.then_some((tls_certainty, tls_services, &mut tls_result_uuid)))
        {
            *result_uuid = insert!(&mut tx, ServiceDetectionResultInsert)
                .return_primary_key()
                .single(&ServiceDetectionResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(self.attack_uuid),
                    certainty,
                    host,
                    port: port as i32,
                })
                .await?;
            insert!(&mut tx, ServiceDetectionName)
                .return_nothing()
                .bulk(services.into_iter().map(|name| ServiceDetectionName {
                    uuid: Uuid::new_v4(),
                    name,
                    result: ForeignModelByField::Key(*result_uuid),
                }))
                .await?;
        }

        // Update the aggregations
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
                PortProtocol::Tcp,
                PortCertainty::Verified,
            )
            .await?;
        let mut tcp_service_uuids = Vec::new();
        let mut tls_service_uuids = Vec::new();
        for (uuid, name, raw, tls, certainty) in &mut services {
            *uuid = GLOBAL
                .aggregator
                .aggregate_service(
                    self.workspace.uuid,
                    host_uuid,
                    Some(port_uuid),
                    Some(ServiceProtocols::Tcp {
                        raw: *raw,
                        tls: *tls,
                    }),
                    name,
                    *certainty,
                )
                .await?;
            if *raw {
                tcp_service_uuids.push(*uuid);
            }
            if *tls {
                tls_service_uuids.push(*uuid);
            }
        }

        // Link raw results with aggregations
        let mut sources = Vec::with_capacity(
            2 + is_tls as usize * 2 + tcp_service_uuids.len() + tls_service_uuids.len(),
        );
        for source_uuid in iter::once(tcp_result_uuid).chain(is_tls.then_some(tls_result_uuid)) {
            sources.extend([
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                    source_type: SourceType::ServiceDetection,
                    source_uuid,
                    aggregated_table: AggregationTable::Host,
                    aggregated_uuid: host_uuid,
                },
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                    source_type: SourceType::ServiceDetection,
                    source_uuid,
                    aggregated_table: AggregationTable::Port,
                    aggregated_uuid: port_uuid,
                },
            ]);
        }
        for (aggregated_uuids, source_uuid) in [
            (tcp_service_uuids, tcp_result_uuid),
            (tls_service_uuids, tls_result_uuid),
        ] {
            for aggregated_uuid in aggregated_uuids {
                sources.push(AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                    source_type: SourceType::ServiceDetection,
                    source_uuid,
                    aggregated_table: AggregationTable::Service,
                    aggregated_uuid,
                });
            }
        }
        insert!(&mut tx, AggregationSource)
            .return_nothing()
            .bulk(sources)
            .await?;

        tx.commit().await?;

        // Find issues in the detected services
        for (uuid, name, _raw, _tls, _certainty) in services {
            match name.as_str() {
                "postgres" => self
                    .finding_factory
                    .add_service(uuid, FindingFactoryIdentifier::ServiceDetectionPostgres),
                "mariadb" => self
                    .finding_factory
                    .add_service(uuid, FindingFactoryIdentifier::ServiceDetectionMariaDb),
                "ssh" => self
                    .finding_factory
                    .add_service(uuid, FindingFactoryIdentifier::ServiceDetectionSsh),
                "snmp" => self
                    .finding_factory
                    .add_service(uuid, FindingFactoryIdentifier::ServiceDetectionSnmp),
                _ => {}
            }
        }

        Ok(())
    }
}

fn parse_service_certainty(data: i32) -> Result<ServiceCertainty, AttackError> {
    const D: i32 = kraken_proto::ServiceCertainty::Definitely as i32;
    const M: i32 = kraken_proto::ServiceCertainty::Maybe as i32;
    const U: i32 = kraken_proto::ServiceCertainty::Unknown as i32;
    match data {
        D => Ok(ServiceCertainty::DefinitelyVerified),
        M => Ok(ServiceCertainty::MaybeVerified),
        U => Ok(ServiceCertainty::UnknownService),
        _ => Err(AttackError::Malformed(
            "Got invalid value for ServiceCertainty",
        )),
    }
}
