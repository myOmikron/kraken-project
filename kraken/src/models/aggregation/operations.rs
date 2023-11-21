use ipnetwork::IpNetwork;
use log::warn;
use rorm::db::Executor;
use rorm::fields::traits::FieldType;
use rorm::prelude::*;
use rorm::{and, insert, query, update};
use uuid::Uuid;

use crate::chan::GLOBAL;
use crate::models::{
    Attack, AttackType, Domain, DomainCertainty, DomainDomainRelation, DomainHostRelation, Host,
    HostCertainty, InsertAttackError, OsType, Port, PortCertainty, PortProtocol, Service,
    ServiceCertainty, Workspace,
};
use crate::modules::attacks::{AttackContext, LeechAttackContext};
use crate::rpc::rpc_definitions::DnsResolutionRequest;

#[derive(Patch)]
#[rorm(model = "Host")]
pub(crate) struct HostInsert {
    pub(crate) uuid: Uuid,
    pub(crate) ip_addr: IpNetwork,
    pub(crate) os_type: OsType,
    pub(crate) response_time: Option<i32>,
    pub(crate) comment: String,
    pub(crate) certainty: HostCertainty,
    pub(crate) workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Service")]
pub(crate) struct ServiceInsert {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) host: ForeignModel<Host>,
    pub(crate) port: Option<ForeignModel<Port>>,
    pub(crate) certainty: ServiceCertainty,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Port")]
pub(crate) struct PortInsert {
    pub(crate) uuid: Uuid,
    pub(crate) port: i16,
    pub(crate) protocol: PortProtocol,
    pub(crate) certainty: PortCertainty,
    pub(crate) host: ForeignModel<Host>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}
#[derive(Patch)]
#[rorm(model = "Domain")]
pub(crate) struct DomainInsert {
    pub(crate) uuid: Uuid,
    pub(crate) domain: String,
    pub(crate) certainty: DomainCertainty,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}

impl Service {
    /// Insert an aggregated service if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate(
        executor: impl Executor<'_>,
        workspace: Uuid,
        host: Uuid,
        port: Option<Uuid>,
        name: &str,
        certainty: ServiceCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let service_uuid = if let Some((service_uuid, old_certainty)) =
            query!(&mut *tx, (Service::F.uuid, Service::F.certainty))
                .condition(and![
                    Service::F.workspace.equals(workspace),
                    Service::F.name.equals(name),
                    Service::F.host.equals(host),
                    // Sadly rorm fails at `Service::F.port.equals(port)`
                    rorm::conditions::Binary {
                        operator: rorm::conditions::BinaryOperator::Equals,
                        fst_arg: rorm::conditions::Column(Service::F.port),
                        snd_arg: port.into_values()[0].clone(),
                    },
                ])
                .optional()
                .await?
        {
            if old_certainty < certainty {
                update!(&mut *tx, Service)
                    .set(Service::F.certainty, certainty)
                    .condition(Service::F.uuid.equals(service_uuid))
                    .await?;
            }
            service_uuid
        } else {
            insert!(tx, Service)
                .return_primary_key()
                .single(&ServiceInsert {
                    uuid: Uuid::new_v4(),
                    name: name.to_string(),
                    version: None,
                    host: ForeignModelByField::Key(host),
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(workspace),
                    port: port.map(ForeignModelByField::Key),
                    certainty,
                })
                .await?
        };

        guard.commit().await?;
        Ok(service_uuid)
    }
}

impl Domain {
    /// Insert an aggregated domain if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    ///
    /// The `user` is required to start dns resolution attacks implicitly.
    pub async fn aggregate(
        executor: impl Executor<'_>,
        workspace: Uuid,
        domain: &str,
        certainty: DomainCertainty,
        user: Uuid,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let uuid = if let Some((uuid, old_certainty)) =
            query!(&mut *tx, (Domain::F.uuid, Domain::F.certainty))
                .condition(and![
                    Domain::F.workspace.equals(workspace),
                    Domain::F.domain.equals(domain)
                ])
                .optional()
                .await?
        {
            if old_certainty < certainty {
                update!(&mut *tx, Domain)
                    .set(Domain::F.certainty, certainty)
                    .condition(Domain::F.uuid.equals(uuid))
                    .await?;
            }
            uuid
        } else {
            let domain_uuid = insert!(&mut *tx, Domain)
                .return_primary_key()
                .single(&DomainInsert {
                    uuid: Uuid::new_v4(),
                    domain: domain.to_string(),
                    certainty,
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(workspace),
                })
                .await?;

            if let Ok(leech) = GLOBAL.leeches.random_leech() {
                let attack_uuid =
                    Attack::insert(&mut *tx, AttackType::DnsResolution, user, workspace)
                        .await
                        .map_err(|err| match err {
                            InsertAttackError::DatabaseError(err) => err,
                            InsertAttackError::WorkspaceInvalid => {
                                unreachable!("Workspace already used above")
                            }
                        })?;
                tokio::spawn(
                    LeechAttackContext {
                        common: AttackContext {
                            user_uuid: user,
                            workspace_uuid: workspace,
                            attack_uuid,
                        },
                        leech,
                    }
                    .dns_resolution(DnsResolutionRequest {
                        attack_uuid: attack_uuid.to_string(),
                        targets: vec![domain.to_string()],
                        concurrent_limit: 1,
                    }),
                );
            } else {
                warn!("Couldn't resolve new domain \"{domain}\" automatically: No leech");
            }
            domain_uuid
        };

        guard.commit().await?;
        Ok(uuid)
    }
}

impl Port {
    /// Insert an aggregated port if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate(
        executor: impl Executor<'_>,
        workspace: Uuid,
        host: Uuid,
        port: u16,
        protocol: PortProtocol,
        certainty: PortCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let port_uuid = if let Some((port_uuid, old_certainty)) =
            query!(&mut *tx, (Port::F.uuid, Port::F.certainty))
                .condition(and![
                    Port::F.port.equals(i16::from_ne_bytes(port.to_ne_bytes())),
                    Port::F.protocol.equals(protocol),
                    Port::F.host.equals(host),
                    Port::F.workspace.equals(workspace),
                ])
                .optional()
                .await?
        {
            if old_certainty < certainty {
                update!(&mut *tx, Port)
                    .set(Port::F.certainty, certainty)
                    .condition(Port::F.uuid.equals(host))
                    .await?;
            }
            port_uuid
        } else {
            insert!(&mut *tx, Port)
                .return_primary_key()
                .single(&PortInsert {
                    uuid: Uuid::new_v4(),
                    port: i16::from_ne_bytes(port.to_ne_bytes()),
                    protocol,
                    certainty,
                    host: ForeignModelByField::Key(host),
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(workspace),
                })
                .await?
        };

        guard.commit().await?;
        Ok(port_uuid)
    }
}

impl Host {
    /// Insert an aggregated host if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate(
        executor: impl Executor<'_>,
        workspace: Uuid,
        ip_addr: IpNetwork,
        certainty: HostCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let uuid = if let Some((uuid, old_certainty)) =
            query!(&mut *tx, (Host::F.uuid, Host::F.certainty))
                .condition(and![
                    Host::F.ip_addr.equals(ip_addr),
                    Host::F.workspace.equals(workspace)
                ])
                .optional()
                .await?
        {
            if old_certainty < certainty {
                update!(&mut *tx, Host)
                    .set(Host::F.certainty, certainty)
                    .condition(Host::F.uuid.equals(uuid))
                    .await?;
            }
            uuid
        } else {
            insert!(&mut *tx, HostInsert)
                .return_primary_key()
                .single(&HostInsert {
                    uuid: Uuid::new_v4(),
                    ip_addr,
                    os_type: OsType::Unknown,
                    response_time: None,
                    comment: String::new(),
                    certainty: HostCertainty::Verified,
                    workspace: ForeignModelByField::Key(workspace),
                })
                .await?
        };

        guard.commit().await?;
        Ok(uuid)
    }
}

impl DomainDomainRelation {
    /// Insert a [`CnameRelation`] if it doesn't exist yet.
    pub async fn insert_if_missing(
        executor: impl Executor<'_>,
        workspace: Uuid,
        source: Uuid,
        destination: Uuid,
    ) -> Result<(), rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        if query!(&mut *tx, (DomainDomainRelation::F.uuid,))
            .condition(and![
                DomainDomainRelation::F.source.equals(source),
                DomainDomainRelation::F.destination.equals(destination)
            ])
            .optional()
            .await?
            .is_none()
        {
            insert!(&mut *tx, DomainDomainRelation)
                .return_nothing()
                .single(&DomainDomainRelation {
                    uuid: Uuid::new_v4(),
                    source: ForeignModelByField::Key(source),
                    destination: ForeignModelByField::Key(destination),
                    workspace: ForeignModelByField::Key(workspace),
                })
                .await?;

            // Create direct domain -> host relations
            for (host,) in query!(&mut *tx, (DomainHostRelation::F.host,))
                .condition(DomainHostRelation::F.domain.equals(destination))
                .all()
                .await?
            {
                DomainHostRelation::insert_if_missing(
                    &mut *tx,
                    workspace,
                    source,
                    *host.key(),
                    false,
                )
                .await?;
            }
        }

        guard.commit().await?;
        Ok(())
    }
}

impl DomainHostRelation {
    /// Insert a [`DomainHostRelation`] if it doesn't exist yet.
    ///
    /// Indirect relations are created implicitly by [`CnameRelation::insert_if_missing`].
    pub async fn insert_if_missing(
        executor: impl Executor<'_>,
        workspace: Uuid,
        domain: Uuid,
        host: Uuid,
        is_direct: bool,
    ) -> Result<(), rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        match query!(
            &mut *tx,
            (DomainHostRelation::F.uuid, DomainHostRelation::F.is_direct)
        )
        .condition(and![
            DomainHostRelation::F.domain.equals(domain),
            DomainHostRelation::F.host.equals(host)
        ])
        .optional()
        .await?
        {
            None => {
                insert!(&mut *tx, DomainHostRelation)
                    .return_nothing()
                    .single(&DomainHostRelation {
                        uuid: Uuid::new_v4(),
                        domain: ForeignModelByField::Key(domain),
                        host: ForeignModelByField::Key(host),
                        workspace: ForeignModelByField::Key(workspace),
                        is_direct: true,
                    })
                    .await?;
            }
            Some((uuid, false)) if is_direct => {
                update!(&mut *tx, DomainHostRelation)
                    .set(DomainHostRelation::F.is_direct, true)
                    .condition(DomainHostRelation::F.uuid.equals(uuid))
                    .await?;
            }
            _ => {}
        }

        guard.commit().await?;
        Ok(())
    }
}
