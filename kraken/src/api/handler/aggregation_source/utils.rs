use std::collections::HashMap;

use futures::TryStreamExt;
use log::error;
use rorm::and;
use rorm::conditions::BoxedCondition;
use rorm::conditions::Condition;
use rorm::conditions::DynamicCollection;
use rorm::db::transaction::Transaction;
use rorm::fields::traits::FieldEq;
use rorm::internal::field::Field;
use rorm::internal::field::FieldProxy;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::aggregation_source::schema::FullAggregationSource;
use crate::api::handler::aggregation_source::schema::ManualInsert;
use crate::api::handler::aggregation_source::schema::SimpleAggregationSource;
use crate::api::handler::aggregation_source::schema::SourceAttack;
use crate::api::handler::aggregation_source::schema::SourceAttackResult;
use crate::api::handler::attack_results::schema::DnsTxtScanEntry;
use crate::api::handler::attack_results::schema::FullDnsTxtScanResult;
use crate::api::handler::attack_results::schema::FullOsDetectionResult;
use crate::api::handler::attack_results::schema::FullQueryCertificateTransparencyResult;
use crate::api::handler::attack_results::schema::FullServiceDetectionResult;
use crate::api::handler::attack_results::schema::FullUdpServiceDetectionResult;
use crate::api::handler::attack_results::schema::SimpleBruteforceSubdomainsResult;
use crate::api::handler::attack_results::schema::SimpleDnsResolutionResult;
use crate::api::handler::attack_results::schema::SimpleHostAliveResult;
use crate::api::handler::attack_results::schema::SimpleQueryUnhashedResult;
use crate::api::handler::users::schema::SimpleUser;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::Attack;
use crate::models::AttackType;
use crate::models::BruteforceSubdomainsResult;
use crate::models::CertificateTransparencyResult;
use crate::models::CertificateTransparencyValueName;
use crate::models::DehashedQueryResult;
use crate::models::DnsResolutionResult;
use crate::models::DnsTxtScanAttackResult;
use crate::models::DnsTxtScanServiceHintEntry;
use crate::models::DnsTxtScanSpfEntry;
use crate::models::HostAliveResult;
use crate::models::ManualDomain;
use crate::models::ManualHost;
use crate::models::ManualPort;
use crate::models::ManualService;
use crate::models::OsDetectionResult;
use crate::models::ServiceDetectionName;
use crate::models::ServiceDetectionResult;
use crate::models::SourceType;
use crate::models::UdpServiceDetectionName;
use crate::models::UdpServiceDetectionResult;

fn field_in<'a, T, F, P, Any>(
    field_proxy: FieldProxy<F, P>,
    values: impl IntoIterator<Item = T>,
) -> BoxedCondition<'a>
where
    T: 'a,
    F: Field,
    F::Type: FieldEq<'a, T, Any>,
    P: rorm::internal::relation_path::Path,
{
    let mut values = values.into_iter();
    if let Some(first_value) = values.next() {
        if let Some(second_value) = values.next() {
            DynamicCollection::or(
                [first_value, second_value]
                    .into_iter()
                    .chain(values)
                    .map(|value| field_proxy.equals(value))
                    .collect(),
            )
            .boxed()
        } else {
            field_proxy.equals(first_value).boxed()
        }
    } else {
        rorm::conditions::Value::Bool(false).boxed()
    }
}

impl SimpleAggregationSource {
    /// Queries the [`SimpleAggregationSource`] for a list of aggregated models
    pub async fn query(
        tx: &mut Transaction,
        workspace: Uuid,
        aggregated_table: AggregationTable,
        aggregated_uuids: impl IntoIterator<Item = Uuid>,
    ) -> Result<HashMap<Uuid, Self>, rorm::Error> {
        let aggregated_uuids: Vec<_> = aggregated_uuids
            .into_iter()
            .map(|uuid| AggregationSource::F.aggregated_uuid.equals(uuid))
            .collect();

        if aggregated_uuids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut stream = query!(
            tx,
            (
                AggregationSource::F.aggregated_uuid,
                AggregationSource::F.source_type
            )
        )
        .condition(and![
            AggregationSource::F.workspace.equals(workspace),
            AggregationSource::F
                .aggregated_table
                .equals(aggregated_table),
            DynamicCollection::or(aggregated_uuids)
        ])
        .stream();

        let mut sources: HashMap<Uuid, SimpleAggregationSource> = HashMap::new();
        while let Some((uuid, source_type)) = stream.try_next().await? {
            sources.entry(uuid).or_default().add(source_type);
        }
        Ok(sources)
    }

    fn add(&mut self, source_type: SourceType) {
        match source_type {
            SourceType::BruteforceSubdomains => self.bruteforce_subdomains += 1,
            SourceType::QueryCertificateTransparency => self.query_certificate_transparency += 1,
            SourceType::QueryDehashed => self.query_dehashed += 1,
            SourceType::HostAlive => self.host_alive += 1,
            SourceType::ServiceDetection => self.service_detection += 1,
            SourceType::UdpServiceDetection => self.udp_service_detection += 1,
            SourceType::DnsResolution => self.dns_resolution += 1,
            SourceType::DnsTxtScan => self.dns_txt_scan += 1,
            SourceType::UdpPortScan => self.udp_port_scan += 1,
            SourceType::ForcedBrowsing => self.forced_browsing += 1,
            SourceType::OSDetection => self.os_detection += 1,
            SourceType::VersionDetection => self.version_detection += 1,
            SourceType::AntiPortScanningDetection => self.anti_port_scanning_detection += 1,
            SourceType::ManualDomain
            | SourceType::ManualHost
            | SourceType::ManualPort
            | SourceType::ManualService => self.manual = true,
            SourceType::TcpPortScan => {}
        }
    }
}

impl Extend<SourceType> for SimpleAggregationSource {
    fn extend<T: IntoIterator<Item = SourceType>>(&mut self, iter: T) {
        for result_type in iter {
            self.add(result_type)
        }
    }
}

impl FullAggregationSource {
    /// Queries the [`FullAggregationSource`] for an aggregated model
    pub async fn query(
        tx: &mut Transaction,
        workspace: Uuid,
        aggregated_table: AggregationTable,
        aggregated_uuid: Uuid,
    ) -> Result<Self, rorm::Error> {
        let mut sources: HashMap<SourceType, Vec<Uuid>> = HashMap::new();
        {
            let mut stream = query!(
                &mut *tx,
                (
                    AggregationSource::F.source_uuid,
                    AggregationSource::F.source_type
                )
            )
            .condition(and![
                AggregationSource::F.workspace.equals(workspace),
                AggregationSource::F.aggregated_uuid.equals(aggregated_uuid),
                AggregationSource::F
                    .aggregated_table
                    .equals(aggregated_table)
            ])
            .stream();
            while let Some((source_uuid, source_type)) = stream.try_next().await? {
                sources.entry(source_type).or_default().push(source_uuid);
            }
        }

        type Results<T> = HashMap<Uuid, Vec<T>>;
        let mut bruteforce_subdomains: Results<SimpleBruteforceSubdomainsResult> = Results::new();
        let mut certificate_transparency: Results<FullQueryCertificateTransparencyResult> =
            Results::new();
        let mut query_dehashed: Results<SimpleQueryUnhashedResult> = Results::new();
        let mut host_alive: Results<SimpleHostAliveResult> = Results::new();
        let mut service_detection: Results<FullServiceDetectionResult> = Results::new();
        let mut udp_service_detection: Results<FullUdpServiceDetectionResult> = Results::new();
        let mut dns_resolution: Results<SimpleDnsResolutionResult> = Results::new();
        let mut dns_txt_scan: Results<FullDnsTxtScanResult> = Results::new();
        let mut os_detection: Results<FullOsDetectionResult> = Results::new();
        let mut manual_insert = Vec::new();
        for (source_type, uuids) in sources {
            if uuids.is_empty() {
                continue;
            }
            match source_type {
                SourceType::BruteforceSubdomains => {
                    let mut stream = query!(&mut *tx, BruteforceSubdomainsResult)
                        .condition(field_in(BruteforceSubdomainsResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        bruteforce_subdomains
                            .entry(*result.attack.key())
                            .or_default()
                            .push(SimpleBruteforceSubdomainsResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                source: result.source,
                                destination: result.destination,
                                dns_record_type: result.dns_record_type,
                            });
                    }
                }
                SourceType::QueryCertificateTransparency => {
                    let mut values: HashMap<Uuid, Vec<String>> = HashMap::new();
                    {
                        let mut stream = query!(
                            &mut *tx,
                            (
                                CertificateTransparencyValueName::F.ct_result,
                                CertificateTransparencyValueName::F.value_name
                            )
                        )
                        .condition(field_in(
                            CertificateTransparencyValueName::F.ct_result,
                            uuids.iter().copied(),
                        ))
                        .stream();
                        while let Some((uuid, value)) = stream.try_next().await? {
                            values.entry(*uuid.key()).or_default().push(value);
                        }
                    }

                    let mut stream = query!(&mut *tx, CertificateTransparencyResult)
                        .condition(field_in(CertificateTransparencyResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        certificate_transparency
                            .entry(*result.attack.key())
                            .or_default()
                            .push(FullQueryCertificateTransparencyResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                issuer_name: result.issuer_name,
                                common_name: result.common_name,
                                value_names: values.remove(&result.uuid).unwrap_or_default(),
                                not_before: result.not_before,
                                not_after: result.not_after,
                                serial_number: result.serial_number,
                            });
                    }
                }
                SourceType::QueryDehashed => {
                    let mut stream = query!(&mut *tx, DehashedQueryResult)
                        .condition(field_in(DehashedQueryResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        query_dehashed
                            .entry(*result.attack.key())
                            .or_default()
                            .push(SimpleQueryUnhashedResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                dehashed_id: result.dehashed_id,
                                email: result.email,
                                username: result.username,
                                password: result.password,
                                hashed_password: result.hashed_password,
                                ip_address: result.ip_address,
                                name: result.name,
                                vin: result.vin,
                                address: result.address,
                                phone: result.phone,
                                database_name: result.database_name,
                            });
                    }
                }
                SourceType::HostAlive => {
                    let mut stream = query!(&mut *tx, HostAliveResult)
                        .condition(field_in(HostAliveResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        host_alive.entry(*result.attack.key()).or_default().push(
                            SimpleHostAliveResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                host: result.host.ip(),
                            },
                        );
                    }
                }
                SourceType::ServiceDetection => {
                    let mut services: HashMap<Uuid, Vec<String>> = HashMap::new();
                    {
                        let mut stream = query!(
                            &mut *tx,
                            (ServiceDetectionName::F.result, ServiceDetectionName::F.name)
                        )
                        .condition(field_in(
                            ServiceDetectionName::F.result,
                            uuids.iter().copied(),
                        ))
                        .stream();
                        while let Some((uuid, value)) = stream.try_next().await? {
                            services.entry(*uuid.key()).or_default().push(value);
                        }
                    }

                    let mut stream = query!(&mut *tx, ServiceDetectionResult)
                        .condition(field_in(ServiceDetectionResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        service_detection
                            .entry(*result.attack.key())
                            .or_default()
                            .push(FullServiceDetectionResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                certainty: result.certainty,
                                service_names: services.remove(&result.uuid).unwrap_or_default(),
                                host: result.host.ip(),
                                port: result.port as u16,
                            });
                    }
                }
                SourceType::UdpServiceDetection => {
                    let mut services: HashMap<Uuid, Vec<String>> = HashMap::new();
                    {
                        let mut stream = query!(
                            &mut *tx,
                            (
                                UdpServiceDetectionName::F.result,
                                UdpServiceDetectionName::F.name
                            )
                        )
                        .condition(field_in(
                            UdpServiceDetectionName::F.result,
                            uuids.iter().copied(),
                        ))
                        .stream();
                        while let Some((uuid, value)) = stream.try_next().await? {
                            services.entry(*uuid.key()).or_default().push(value);
                        }
                    }

                    let mut stream = query!(&mut *tx, UdpServiceDetectionResult)
                        .condition(field_in(UdpServiceDetectionResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        udp_service_detection
                            .entry(*result.attack.key())
                            .or_default()
                            .push(FullUdpServiceDetectionResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                certainty: result.certainty,
                                service_names: services.remove(&result.uuid).unwrap_or_default(),
                                host: result.host.ip(),
                                port: result.port as u16,
                            });
                    }
                }
                SourceType::DnsResolution => {
                    let mut stream = query!(&mut *tx, DnsResolutionResult)
                        .condition(field_in(DnsResolutionResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        dns_resolution
                            .entry(*result.attack.key())
                            .or_default()
                            .push(SimpleDnsResolutionResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                source: result.source,
                                destination: result.destination,
                                dns_record_type: result.dns_record_type,
                            });
                    }
                }
                SourceType::DnsTxtScan => {
                    let service_hints = query!(&mut *tx, DnsTxtScanServiceHintEntry)
                        .condition(field_in(
                            DnsTxtScanServiceHintEntry::F.collection,
                            uuids.clone(),
                        ))
                        .all()
                        .await?;
                    let spf = query!(&mut *tx, DnsTxtScanSpfEntry)
                        .condition(field_in(DnsTxtScanSpfEntry::F.collection, uuids.clone()))
                        .all()
                        .await?;
                    let mut stream = query!(&mut *tx, DnsTxtScanAttackResult)
                        .condition(field_in(DnsTxtScanAttackResult::F.uuid, uuids.clone()))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        dns_txt_scan.entry(*result.attack.key()).or_default().push(
                            FullDnsTxtScanResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                domain: result.domain,
                                created_at: result.created_at,
                                collection_type: result.collection_type,
                                entries: service_hints
                                    .iter()
                                    .filter(|s| *s.collection.key() == result.uuid)
                                    .map(|s| DnsTxtScanEntry::ServiceHint {
                                        created_at: s.created_at,
                                        uuid: s.uuid,
                                        txt_type: s.txt_type,
                                        rule: s.rule.clone(),
                                    })
                                    .chain(
                                        spf.iter()
                                            .filter(|s| *s.collection.key() == result.uuid)
                                            .map(|s| DnsTxtScanEntry::Spf {
                                                created_at: s.created_at,
                                                uuid: s.uuid,
                                                rule: s.rule.clone(),
                                                spf_type: s.spf_type,
                                                spf_ip: s.spf_ip,
                                                spf_domain: s.spf_domain.clone(),
                                                spf_domain_ipv4_cidr: s.spf_domain_ipv4_cidr,
                                                spf_domain_ipv6_cidr: s.spf_domain_ipv6_cidr,
                                            }),
                                    )
                                    .collect(),
                            },
                        );
                    }
                }
                SourceType::OSDetection => {
                    let mut stream = query!(&mut *tx, OsDetectionResult)
                        .condition(field_in(OsDetectionResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        os_detection.entry(*result.attack.key()).or_default().push(
                            FullOsDetectionResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                host: result.host.ip(),
                                hints: result.hints,
                                version: result.version,
                                os: result.os,
                            },
                        );
                    }
                }
                SourceType::UdpPortScan
                | SourceType::ForcedBrowsing
                | SourceType::VersionDetection
                | SourceType::AntiPortScanningDetection => {
                    error!("source type unimplemented: {source_type:?}")
                }
                SourceType::ManualDomain => {
                    let mut stream = query!(
                        &mut *tx,
                        (
                            ManualDomain::F.domain,
                            ManualDomain::F.user as SimpleUser,
                            ManualDomain::F.workspace,
                            ManualDomain::F.created_at,
                        )
                    )
                    .condition(field_in(ManualDomain::F.uuid, uuids))
                    .stream();
                    while let Some((domain, user, workspace, created_at)) =
                        stream.try_next().await?
                    {
                        manual_insert.push(ManualInsert::Domain {
                            domain,
                            user,
                            workspace: *workspace.key(),
                            created_at,
                        });
                    }
                }
                SourceType::ManualHost => {
                    let mut stream = query!(
                        &mut *tx,
                        (
                            ManualHost::F.ip_addr,
                            ManualHost::F.os_type,
                            ManualHost::F.certainty,
                            ManualHost::F.user as SimpleUser,
                            ManualHost::F.workspace,
                            ManualHost::F.created_at,
                        )
                    )
                    .condition(field_in(ManualHost::F.uuid, uuids))
                    .stream();
                    while let Some((ip_addr, os_type, certainty, user, workspace, created_at)) =
                        stream.try_next().await?
                    {
                        manual_insert.push(ManualInsert::Host {
                            ip_addr: ip_addr.ip(),
                            os_type,
                            certainty,
                            user,
                            workspace: *workspace.key(),
                            created_at,
                        });
                    }
                }
                SourceType::ManualPort => {
                    let mut stream = query!(
                        &mut *tx,
                        (
                            ManualPort::F.port,
                            ManualPort::F.protocol,
                            ManualPort::F.certainty,
                            ManualPort::F.host,
                            ManualPort::F.user as SimpleUser,
                            ManualPort::F.workspace,
                            ManualPort::F.created_at,
                        )
                    )
                    .condition(field_in(ManualPort::F.uuid, uuids))
                    .stream();
                    while let Some((port, protocol, certainty, host, user, workspace, created_at)) =
                        stream.try_next().await?
                    {
                        manual_insert.push(ManualInsert::Port {
                            port: port as u16,
                            protocol,
                            certainty,
                            host: host.ip(),
                            user,
                            workspace: *workspace.key(),
                            created_at,
                        });
                    }
                }
                SourceType::ManualService => {
                    let mut stream = query!(
                        &mut *tx,
                        (
                            ManualService::F.name,
                            ManualService::F.port,
                            ManualService::F.certainty,
                            ManualService::F.host,
                            ManualService::F.user as SimpleUser,
                            ManualService::F.workspace,
                            ManualService::F.created_at,
                        )
                    )
                    .condition(field_in(ManualService::F.uuid, uuids))
                    .stream();
                    while let Some((name, port, certainty, host, user, workspace, created_at)) =
                        stream.try_next().await?
                    {
                        manual_insert.push(ManualInsert::Service {
                            name,
                            port: port.map(|p| p as u16),
                            certainty,
                            host: host.ip(),
                            user,
                            workspace: *workspace.key(),
                            created_at,
                            version: None,
                        });
                    }
                }
                SourceType::TcpPortScan => {}
            }
        }

        let mut attacks = Vec::new();
        {
            let mut stream = query!(
                &mut *tx,
                (
                    Attack::F.uuid,
                    Attack::F.workspace,
                    Attack::F.attack_type,
                    Attack::F.finished_at,
                    Attack::F.created_at,
                    Attack::F.started_by as SimpleUser,
                    Attack::F.error,
                )
            )
            .condition(field_in(
                Attack::F.uuid,
                bruteforce_subdomains
                    .keys()
                    .chain(certificate_transparency.keys())
                    .chain(query_dehashed.keys())
                    .chain(host_alive.keys())
                    .chain(service_detection.keys())
                    .chain(udp_service_detection.keys())
                    .chain(dns_resolution.keys())
                    .chain(dns_txt_scan.keys())
                    .chain(os_detection.keys())
                    .copied(),
            ))
            .stream();
            while let Some((
                uuid,
                workspace,
                attack_type,
                finished_at,
                created_at,
                started_by,
                error,
            )) = stream.try_next().await?
            {
                let results = match attack_type {
                    AttackType::Undefined => {
                        error!("An `AttackType::Undefined` shouldn't have been queried");
                        continue;
                    }
                    AttackType::BruteforceSubdomains => SourceAttackResult::BruteforceSubdomains(
                        bruteforce_subdomains.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::QueryCertificateTransparency => {
                        SourceAttackResult::QueryCertificateTransparency(
                            certificate_transparency.remove(&uuid).unwrap_or_default(),
                        )
                    }
                    AttackType::QueryUnhashed => SourceAttackResult::QueryDehashed(
                        query_dehashed.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::HostAlive => {
                        SourceAttackResult::HostAlive(host_alive.remove(&uuid).unwrap_or_default())
                    }
                    AttackType::ServiceDetection => SourceAttackResult::ServiceDetection(
                        service_detection.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::UdpServiceDetection => SourceAttackResult::UdpServiceDetection(
                        udp_service_detection.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::DnsResolution => SourceAttackResult::DnsResolution(
                        dns_resolution.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::DnsTxtScan => SourceAttackResult::DnsTxtScan(
                        dns_txt_scan.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::OSDetection => SourceAttackResult::OsDetection(
                        os_detection.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::UdpPortScan
                    | AttackType::ForcedBrowsing
                    | AttackType::VersionDetection
                    | AttackType::AntiPortScanningDetection => {
                        error!("An `{attack_type:?}` isn't implemented yet");
                        continue;
                    }
                    AttackType::TcpPortScan => continue,
                };
                attacks.push(SourceAttack {
                    uuid,
                    workspace_uuid: *workspace.key(),
                    started_by,
                    finished_at,
                    error,
                    created_at,
                    results,
                });
            }
        }

        Ok(Self {
            attacks,
            manual_insert,
        })
    }
}
