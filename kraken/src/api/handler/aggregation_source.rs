//! Declaration of [`SimpleAggregationSource`], [`FullAggregationSource`] and the implementation to query them

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use log::error;
use rorm::conditions::{BoxedCondition, Condition, DynamicCollection};
use rorm::db::transaction::Transaction;
use rorm::fields::traits::FieldEq;
use rorm::internal::field::{Field, FieldProxy};
use rorm::prelude::*;
use rorm::{and, query};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::attack_results::{
    FullQueryCertificateTransparencyResult, FullServiceDetectionResult,
    SimpleBruteforceSubdomainsResult, SimpleDnsResolutionResult, SimpleHostAliveResult,
    SimpleQueryUnhashedResult, SimpleTcpPortScanResult,
};
use crate::api::handler::users::SimpleUser;
use crate::models::{
    AggregationSource, AggregationTable, Attack, AttackType, BruteforceSubdomainsResult,
    CertificateTransparencyResult, CertificateTransparencyValueName, DehashedQueryResult,
    DnsResolutionResult, HostAliveResult, ManualDomain, ManualHost, ManualHostCertainty,
    ManualPort, ManualPortCertainty, ManualService, ManualServiceCertainty, OsType, PortProtocol,
    ServiceDetectionName, ServiceDetectionResult, SourceType, TcpPortScanResult,
};
/// Numbers how many attacks of a certain kind found an aggregated model
#[derive(Copy, Clone, Serialize, ToSchema, Debug, Default)]
pub struct SimpleAggregationSource {
    /// Bruteforce subdomains via DNS requests
    bruteforce_subdomains: usize,
    /// Scan tcp ports
    tcp_port_scan: usize,
    /// Query certificate transparency
    query_certificate_transparency: usize,
    /// Query the dehashed API
    query_dehashed: usize,
    /// Check if a host is reachable via icmp
    host_alive: usize,
    /// Detect the service that is running on a port
    service_detection: usize,
    /// Resolve domain names
    dns_resolution: usize,
    /// Perform forced browsing
    forced_browsing: usize,
    /// Detect the OS of the target
    os_detection: usize,
    /// Detect if anti-port scanning techniques are in place
    anti_port_scanning_detection: usize,
    /// Scan udp ports
    udp_port_scan: usize,
    /// Perform version detection
    version_detection: usize,
    /// Manually inserted
    manual: bool,
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
            SourceType::TcpPortScan => self.tcp_port_scan += 1,
            SourceType::QueryCertificateTransparency => self.query_certificate_transparency += 1,
            SourceType::QueryDehashed => self.query_dehashed += 1,
            SourceType::HostAlive => self.host_alive += 1,
            SourceType::ServiceDetection => self.service_detection += 1,
            SourceType::DnsResolution => self.dns_resolution += 1,
            SourceType::UdpPortScan => self.udp_port_scan += 1,
            SourceType::ForcedBrowsing => self.forced_browsing += 1,
            SourceType::OSDetection => self.os_detection += 1,
            SourceType::VersionDetection => self.version_detection += 1,
            SourceType::AntiPortScanningDetection => self.anti_port_scanning_detection += 1,
            SourceType::ManualDomain
            | SourceType::ManualHost
            | SourceType::ManualPort
            | SourceType::ManualService => self.manual = true,
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

/// All data sources which contributed to an aggregated model
#[derive(Serialize, ToSchema)]
pub struct FullAggregationSource {
    /// All attack which contributed to an aggregated model
    attacks: Vec<SourceAttack>,

    /// All manual inserts which contributed to an aggregated model
    manual_insert: Vec<ManualInsert>,
}
/// Copy of [`SimpleAttack`](crate::api::handler::attacks::SimpleAttack) with an added `results` field
#[derive(Serialize, ToSchema)]
pub struct SourceAttack {
    /// The identifier of the attack
    pub uuid: Uuid,
    /// The workspace this attack is attached to
    pub workspace_uuid: Uuid,
    /// The user that has started the attack
    pub started_by: SimpleUser,
    /// If this is None, the attack is still running
    pub finished_at: Option<DateTime<Utc>>,
    /// If this field is set, the attack has finished with an error
    pub error: Option<String>,
    /// The point in time this attack was started
    pub created_at: DateTime<Utc>,
    /// Flattened enum storing the `attack_type` next to the `results`
    #[serde(flatten)]
    pub results: SourceAttackResult,
}

/// The different types of attack and their results
#[derive(Serialize, ToSchema)]
#[serde(tag = "attack_type", content = "results")]
pub enum SourceAttackResult {
    /// The [`AttackType::BruteforceSubdomains`] and its results
    BruteforceSubdomains(Vec<SimpleBruteforceSubdomainsResult>),
    /// The [`AttackType::TcpPortScan`] and its results
    TcpPortScan(Vec<SimpleTcpPortScanResult>),
    /// The [`AttackType::QueryCertificateTransparency`] and its results
    QueryCertificateTransparency(Vec<FullQueryCertificateTransparencyResult>),
    /// The [`AttackType::QueryUnhashed`] and its results
    QueryDehashed(Vec<SimpleQueryUnhashedResult>),
    /// The [`AttackType::HostAlive`] and its results
    HostAlive(Vec<SimpleHostAliveResult>),
    /// The [`AttackType::ServiceDetection`] and its results
    ServiceDetection(Vec<FullServiceDetectionResult>),
    /// The [`AttackType::DnsResolution`] and its results
    DnsResolution(Vec<SimpleDnsResolutionResult>),
}

/// The different types of manual inserts
#[derive(Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum ManualInsert {
    /// A manually inserted domain
    Domain {
        /// The inserted domain
        domain: String,
        /// The user which inserted the domain
        user: SimpleUser,
        /// The workspace the domain was inserted to
        workspace: Uuid,
        /// The point in time, the domain was inserted
        created_at: DateTime<Utc>,
    },
    /// A manually inserted host
    Host {
        /// The host's ip address
        #[schema(example = "172.0.0.1")]
        ip_addr: String,
        /// The host's os type
        os_type: OsType,
        /// The inserted data's certainty
        certainty: ManualHostCertainty,
        /// The user which inserted the host
        user: SimpleUser,
        /// The workspace the host was inserted to
        workspace: Uuid,
        /// The point in time, the host was inserted
        created_at: DateTime<Utc>,
    },
    /// A manually inserted port
    Port {
        /// The inserted port
        port: u16,
        /// The port's protocol
        protocol: PortProtocol,
        /// The inserted data's certainty
        certainty: ManualPortCertainty,
        /// The host's ip address
        #[schema(example = "172.0.0.1")]
        host: String,
        /// The user which inserted the port
        user: SimpleUser,
        /// The workspace the port was inserted to
        workspace: Uuid,
        /// The point in time, the port was inserted
        created_at: DateTime<Utc>,
    },
    /// A manually inserted service
    Service {
        /// The inserted service
        name: String,
        /// The service's version
        version: Option<String>,
        /// The inserted data's certainty
        certainty: ManualServiceCertainty,
        /// The service's port
        port: Option<u16>,
        /// The host's ip address
        #[schema(example = "172.0.0.1")]
        host: String,
        /// The user which inserted the service
        user: SimpleUser,
        /// The workspace the service was inserted to
        workspace: Uuid,
        /// The point in time, the service was inserted
        created_at: DateTime<Utc>,
    },
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
        let mut tcp_port_scan: Results<SimpleTcpPortScanResult> = Results::new();
        let mut certificate_transparency: Results<FullQueryCertificateTransparencyResult> =
            Results::new();
        let mut query_dehashed: Results<SimpleQueryUnhashedResult> = Results::new();
        let mut host_alive: Results<SimpleHostAliveResult> = Results::new();
        let mut service_detection: Results<FullServiceDetectionResult> = Results::new();
        let mut dns_resolution: Results<SimpleDnsResolutionResult> = Results::new();
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
                SourceType::TcpPortScan => {
                    let mut stream = query!(&mut *tx, TcpPortScanResult)
                        .condition(field_in(TcpPortScanResult::F.uuid, uuids))
                        .stream();
                    while let Some(result) = stream.try_next().await? {
                        tcp_port_scan.entry(*result.attack.key()).or_default().push(
                            SimpleTcpPortScanResult {
                                uuid: result.uuid,
                                attack: *result.attack.key(),
                                created_at: result.created_at,
                                address: result.address,
                                port: result.port as u16,
                            },
                        );
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
                                host: result.host,
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
                                host: result.host,
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
                SourceType::UdpPortScan
                | SourceType::ForcedBrowsing
                | SourceType::OSDetection
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
                            ip_addr: ip_addr.ip().to_string(),
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
                            host: host.ip().to_string(),
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
                            host: host.ip().to_string(),
                            user,
                            workspace: *workspace.key(),
                            created_at,
                            version: None,
                        });
                    }
                }
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
                    .chain(tcp_port_scan.keys())
                    .chain(certificate_transparency.keys())
                    .chain(query_dehashed.keys())
                    .chain(host_alive.keys())
                    .chain(service_detection.keys())
                    .chain(dns_resolution.keys())
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
                    AttackType::TcpPortScan => SourceAttackResult::TcpPortScan(
                        tcp_port_scan.remove(&uuid).unwrap_or_default(),
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
                    AttackType::DnsResolution => SourceAttackResult::DnsResolution(
                        dns_resolution.remove(&uuid).unwrap_or_default(),
                    ),
                    AttackType::UdpPortScan
                    | AttackType::ForcedBrowsing
                    | AttackType::OSDetection
                    | AttackType::VersionDetection
                    | AttackType::AntiPortScanningDetection => {
                        error!("An `{attack_type:?}` isn't implemented yet");
                        continue;
                    }
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
