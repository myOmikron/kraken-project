use std::net::IpAddr;
use std::num::NonZeroU16;

use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::api::handler::services::schema::ServiceProtocols;
use crate::chan::global::GLOBAL;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::DomainCertainty;
use crate::models::HostCertainty;
use crate::models::ManualDomain;
use crate::models::ManualHost;
use crate::models::ManualHostCertainty;
use crate::models::ManualHttpService;
use crate::models::ManualPort;
use crate::models::ManualPortCertainty;
use crate::models::ManualService;
use crate::models::ManualServiceCertainty;
use crate::models::OsType;
use crate::models::PortCertainty;
use crate::models::PortProtocol;
use crate::models::ServiceCertainty;
use crate::models::SourceType;
use crate::models::User;
use crate::models::Workspace;

#[derive(Patch)]
#[rorm(model = "ManualDomain")]
struct InsertManualDomain {
    uuid: Uuid,
    domain: String,
    user: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
}

impl ManualDomain {
    /// Manually insert a domain
    ///
    /// This function will store the raw data given by the user
    /// and add it to the aggregations.
    ///
    /// The [`Domain`]'s uuid will be returned.
    pub async fn insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
        domain: String,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let source_uuid = insert!(&mut *tx, ManualDomain)
            .return_primary_key()
            .single(&InsertManualDomain {
                uuid: Uuid::new_v4(),
                domain: domain.clone(),
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
            })
            .await?;

        let domain_uuid = GLOBAL
            .aggregator
            .aggregate_domain(workspace, &domain, DomainCertainty::Unverified, user)
            .await?;

        insert!(&mut *tx, AggregationSource)
            .single(&AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace),
                source_type: SourceType::ManualDomain,
                source_uuid,
                aggregated_table: AggregationTable::Domain,
                aggregated_uuid: domain_uuid,
            })
            .await?;

        guard.commit().await?;
        Ok(domain_uuid)
    }
}

#[derive(Patch)]
#[rorm(model = "ManualHost")]
struct InsertManualHost {
    uuid: Uuid,
    ip_addr: IpNetwork,
    os_type: OsType,
    certainty: ManualHostCertainty,
    user: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
}

impl ManualHost {
    /// Manually insert a host
    ///
    /// This function will store the raw data given by the user
    /// and add it to the aggregations.
    ///
    /// The [`Host`]'s uuid will be returned.
    pub async fn insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
        ip_addr: IpNetwork,
        certainty: ManualHostCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let source_uuid = insert!(&mut *tx, ManualHost)
            .return_primary_key()
            .single(&InsertManualHost {
                uuid: Uuid::new_v4(),
                ip_addr,
                os_type: OsType::Unknown,
                certainty,
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
            })
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(
                workspace,
                ip_addr,
                match certainty {
                    ManualHostCertainty::Historical => HostCertainty::Historical,
                    ManualHostCertainty::SupposedTo => HostCertainty::SupposedTo,
                },
            )
            .await?;

        insert!(&mut *tx, AggregationSource)
            .single(&AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace),
                source_type: SourceType::ManualHost,
                source_uuid,
                aggregated_table: AggregationTable::Host,
                aggregated_uuid: host_uuid,
            })
            .await?;

        guard.commit().await?;
        Ok(host_uuid)
    }
}

#[derive(Patch)]
#[rorm(model = "ManualPort")]
pub struct InsertManualPort {
    uuid: Uuid,
    port: i32,
    protocol: PortProtocol,
    certainty: ManualPortCertainty,
    host: IpNetwork,
    user: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
}

impl ManualPort {
    /// Manually insert a port
    ///
    /// This function will store the raw data given by the user
    /// and add it to the aggregations.
    ///
    /// The [`Port`]'s uuid will be returned.
    pub async fn insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
        ip_addr: IpNetwork,
        port: u16,
        certainty: ManualPortCertainty,
        protocol: PortProtocol,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let source_uuid = insert!(&mut *tx, ManualPort)
            .return_primary_key()
            .single(&InsertManualPort {
                uuid: Uuid::new_v4(),
                port: port as i32,
                protocol,
                certainty,
                host: ip_addr,
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
            })
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(
                workspace,
                ip_addr,
                match certainty {
                    ManualPortCertainty::Historical => HostCertainty::Historical,
                    ManualPortCertainty::SupposedTo => HostCertainty::SupposedTo,
                },
            )
            .await?;

        let port_uuid = GLOBAL
            .aggregator
            .aggregate_port(
                workspace,
                host_uuid,
                port,
                protocol,
                match certainty {
                    ManualPortCertainty::Historical => PortCertainty::Historical,
                    ManualPortCertainty::SupposedTo => PortCertainty::SupposedTo,
                },
            )
            .await?;

        insert!(&mut *tx, AggregationSource)
            .bulk([
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace),
                    source_type: SourceType::ManualPort,
                    source_uuid,
                    aggregated_table: AggregationTable::Host,
                    aggregated_uuid: host_uuid,
                },
                AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace),
                    source_type: SourceType::ManualPort,
                    source_uuid,
                    aggregated_table: AggregationTable::Port,
                    aggregated_uuid: port_uuid,
                },
            ])
            .await?;

        guard.commit().await?;
        Ok(port_uuid)
    }
}

#[derive(Patch)]
#[rorm(model = "ManualService")]
struct InsertManualService {
    uuid: Uuid,
    name: String,
    version: Option<String>,
    certainty: ManualServiceCertainty,
    host: IpNetwork,
    port: Option<i32>,
    protocol: PortProtocol,
    protocols: i16,
    user: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
}

impl ManualService {
    /// Manually insert a service
    ///
    /// This function will store the raw data given by the user
    /// and add it to the aggregations.
    ///
    /// The [`Service`]'s uuid will be returned.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
        name: String,
        host: IpNetwork,
        port: Option<(u16, ServiceProtocols)>,
        certainty: ManualServiceCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let port_protocol = match port {
            Some((_, ServiceProtocols::Tcp { .. })) => PortProtocol::Tcp,
            Some((_, ServiceProtocols::Udp { .. })) => PortProtocol::Udp,
            Some((_, ServiceProtocols::Sctp { .. })) => PortProtocol::Sctp,
            Some((_, ServiceProtocols::Unknown { .. })) | None => PortProtocol::Unknown,
        };

        let source_uuid = insert!(&mut *tx, ManualService)
            .return_primary_key()
            .single(&InsertManualService {
                uuid: Uuid::new_v4(),
                name: name.clone(),
                version: None,
                certainty,
                host,
                port: port.map(|(x, _)| x as i32),
                protocol: port_protocol,
                protocols: port.map(|(_, x)| x.encode()).unwrap_or(0),
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
            })
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(
                workspace,
                host,
                match certainty {
                    ManualServiceCertainty::Historical => HostCertainty::Historical,
                    ManualServiceCertainty::SupposedTo => HostCertainty::SupposedTo,
                },
            )
            .await?;

        let port_uuid = if let Some((port, _)) = port {
            Some(
                GLOBAL
                    .aggregator
                    .aggregate_port(
                        workspace,
                        host_uuid,
                        port,
                        port_protocol,
                        match certainty {
                            ManualServiceCertainty::Historical => PortCertainty::Historical,
                            ManualServiceCertainty::SupposedTo => PortCertainty::SupposedTo,
                        },
                    )
                    .await?,
            )
        } else {
            None
        };

        let service_uuid = GLOBAL
            .aggregator
            .aggregate_service(
                workspace,
                host_uuid,
                port_uuid,
                port.map(|(_, x)| x),
                &name,
                match certainty {
                    ManualServiceCertainty::Historical => ServiceCertainty::Historical,
                    ManualServiceCertainty::SupposedTo => ServiceCertainty::SupposedTo,
                },
            )
            .await?;

        insert!(&mut *tx, AggregationSource)
            .bulk(
                [
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(workspace),
                        source_type: SourceType::ManualService,
                        source_uuid,
                        aggregated_table: AggregationTable::Host,
                        aggregated_uuid: host_uuid,
                    },
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(workspace),
                        source_type: SourceType::ManualService,
                        source_uuid,
                        aggregated_table: AggregationTable::Service,
                        aggregated_uuid: service_uuid,
                    },
                ]
                .into_iter()
                .chain(port_uuid.map(|port_uuid| AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace),
                    source_type: SourceType::ManualService,
                    source_uuid,
                    aggregated_table: AggregationTable::Port,
                    aggregated_uuid: port_uuid,
                })),
            )
            .await?;

        guard.commit().await?;
        Ok(service_uuid)
    }
}

#[derive(Patch)]
#[rorm(model = "ManualHttpService")]
struct InsertManualHttpService {
    uuid: Uuid,
    name: String,
    domain: Option<String>,
    ip_addr: IpNetwork,
    port: i32,
    port_protocol: PortProtocol,
    base_path: String,
    tls: bool,
    sni_required: bool,
    user: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
}

impl ManualHttpService {
    /// Manually insert a http service
    ///
    /// This function will store the raw data given by the user
    /// and add it to the aggregations.
    ///
    /// The [`HttpService`]'s uuid will be returned.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
        name: String,
        domain: Option<String>,
        ip_addr: IpAddr,
        port: NonZeroU16,
        port_protocol: PortProtocol,
        base_path: String,
        tls: bool,
        sni_required: bool,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;

        let source_uuid = insert!(guard.get_transaction(), ManualHttpService)
            .return_primary_key()
            .single(&InsertManualHttpService {
                uuid: Uuid::new_v4(),
                name: name.clone(),
                domain: domain.clone(),
                ip_addr: IpNetwork::from(ip_addr),
                port: port.get() as i32,
                port_protocol,
                base_path: base_path.clone(),
                tls,
                sni_required,
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
            })
            .await?;

        let host_uuid = GLOBAL
            .aggregator
            .aggregate_host(
                workspace,
                IpNetwork::from(ip_addr),
                HostCertainty::SupposedTo, // TODO
            )
            .await?;

        let port_uuid = GLOBAL
            .aggregator
            .aggregate_port(
                workspace,
                host_uuid,
                port.get(),
                port_protocol,
                PortCertainty::SupposedTo, // TODO
            )
            .await?;

        let domain_uuid = if let Some(domain) = domain {
            Some(
                GLOBAL
                    .aggregator
                    .aggregate_domain(workspace, &domain, DomainCertainty::Unverified, user) // TODO
                    .await?,
            )
        } else {
            None
        };

        let http_service_uuid = GLOBAL
            .aggregator
            .aggregate_http_service(
                workspace,
                name,
                host_uuid,
                port_uuid,
                domain_uuid,
                base_path,
                tls,
                sni_required,
            )
            .await?;

        insert!(guard.get_transaction(), AggregationSource)
            .bulk(
                [
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(workspace),
                        source_type: SourceType::ManualHttpService,
                        source_uuid,
                        aggregated_table: AggregationTable::Host,
                        aggregated_uuid: host_uuid,
                    },
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(workspace),
                        source_type: SourceType::ManualHttpService,
                        source_uuid,
                        aggregated_table: AggregationTable::Port,
                        aggregated_uuid: port_uuid,
                    },
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(workspace),
                        source_type: SourceType::ManualHttpService,
                        source_uuid,
                        aggregated_table: AggregationTable::HttpService,
                        aggregated_uuid: http_service_uuid,
                    },
                ]
                .into_iter()
                .chain(domain_uuid.map(|domain_uuid| AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace),
                    source_type: SourceType::ManualHttpService,
                    source_uuid,
                    aggregated_table: AggregationTable::Domain,
                    aggregated_uuid: domain_uuid,
                })),
            )
            .await?;

        guard.commit().await?;
        Ok(http_service_uuid)
    }
}
