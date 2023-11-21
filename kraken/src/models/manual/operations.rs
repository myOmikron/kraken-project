use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::models::{
    AggregationSource, AggregationTable, Domain, DomainCertainty, Host, HostCertainty,
    ManualDomain, ManualHost, ManualHostCertainty, ManualPort, ManualPortCertainty, ManualService,
    ManualServiceCertainty, OsType, Port, PortCertainty, PortProtocol, Service, ServiceCertainty,
    SourceType, User, Workspace,
};

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

        let domain_uuid = Domain::aggregate(
            &mut *tx,
            workspace,
            &domain,
            DomainCertainty::Unverified,
            user,
        )
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

        let host_uuid = Host::aggregate(
            &mut *tx,
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
    port: i16,
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
                port: i16::from_ne_bytes(port.to_ne_bytes()),
                protocol,
                certainty,
                host: ip_addr,
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
            })
            .await?;

        let host_uuid = Host::aggregate(
            &mut *tx,
            workspace,
            ip_addr,
            match certainty {
                ManualPortCertainty::Historical => HostCertainty::Historical,
                ManualPortCertainty::SupposedTo => HostCertainty::SupposedTo,
            },
        )
        .await?;

        let port_uuid = Port::aggregate(
            &mut *tx,
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
    port: Option<i16>,
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
    pub async fn insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
        name: String,
        host: IpNetwork,
        port: Option<u16>,
        certainty: ManualServiceCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let source_uuid = insert!(&mut *tx, ManualService)
            .return_primary_key()
            .single(&InsertManualService {
                uuid: Uuid::new_v4(),
                name: name.clone(),
                version: None,
                certainty,
                host,
                port: port.map(u16::to_ne_bytes).map(i16::from_ne_bytes),
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
            })
            .await?;

        let host_uuid = Host::aggregate(
            &mut *tx,
            workspace,
            host,
            match certainty {
                ManualServiceCertainty::Historical => HostCertainty::Historical,
                ManualServiceCertainty::SupposedTo => HostCertainty::SupposedTo,
            },
        )
        .await?;

        let port_uuid = if let Some(port) = port {
            Some(
                Port::aggregate(
                    &mut *tx,
                    workspace,
                    host_uuid,
                    port,
                    PortProtocol::Tcp,
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

        let service_uuid = Service::aggregate(
            &mut *tx,
            workspace,
            host_uuid,
            port_uuid,
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
