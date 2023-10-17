use ipnetwork::IpNetwork;
use rorm::conditions::{Condition, DynamicCollection};
use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{and, insert, query, update};
use uuid::Uuid;

use crate::models::{
    Certainty, Domain, DomainDomainRelation, DomainHostRelation, Host, OsType, Port, PortProtocol,
    Service, Workspace,
};

#[derive(Patch)]
#[rorm(model = "Host")]
pub(crate) struct HostInsert {
    pub(crate) uuid: Uuid,
    pub(crate) ip_addr: IpNetwork,
    pub(crate) os_type: OsType,
    pub(crate) response_time: Option<i32>,
    pub(crate) comment: String,
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
    pub(crate) certainty: Certainty,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Port")]
pub(crate) struct PortInsert {
    pub(crate) uuid: Uuid,
    pub(crate) port: i16,
    pub(crate) protocol: PortProtocol,
    pub(crate) host: ForeignModel<Host>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}
#[derive(Patch)]
#[rorm(model = "Domain")]
pub(crate) struct DomainInsert {
    pub(crate) uuid: Uuid,
    pub(crate) domain: String,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}

impl Service {
    /// Insert an aggregated service if it doesn't exist yet.
    ///
    /// Returns whether the service was inserted or not.
    pub async fn update_or_insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        name: &str,
        host: IpNetwork,
        port: Option<i16>,
        certainty: Certainty,
    ) -> Result<bool, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let mut c = vec![
            Service::F.workspace.equals(workspace).boxed(),
            Service::F.name.equals(name).boxed(),
            Service::F.host.ip_addr.equals(host).boxed(),
        ];
        let cond = if let Some(port) = port {
            c.push(Service::F.port.port.equals(port).boxed());
            DynamicCollection::and(c)
        } else {
            DynamicCollection::and(c)
        };

        let service = query!(&mut *tx, Service).condition(cond).optional().await?;

        let res = if let Some(service) = service {
            if service.certainty != certainty && certainty == Certainty::Definitely {
                update!(&mut *tx, Service)
                    .condition(Service::F.uuid.equals(service.uuid))
                    .set(Service::F.certainty, Certainty::Definitely)
                    .exec()
                    .await?;
            }
            false
        } else {
            // Check if host is already been created
            let host_uuid = query!(&mut *tx, (Host::F.uuid,))
                .condition(and!(
                    Host::F.workspace.equals(workspace),
                    Host::F.ip_addr.equals(host)
                ))
                .optional()
                .await?;

            let host_uuid = if let Some((host_uuid,)) = host_uuid {
                host_uuid
            } else {
                insert!(&mut *tx, HostInsert)
                    .return_primary_key()
                    .single(&HostInsert {
                        uuid: Uuid::new_v4(),
                        ip_addr: host,
                        os_type: OsType::Unknown,
                        response_time: None,
                        comment: String::new(),
                        workspace: ForeignModelByField::Key(workspace),
                    })
                    .await?
            };

            let mut port_uuid = None;
            if let Some(port) = port {
                // Check if port already exists
                let p = query!(&mut *tx, (Port::F.uuid,))
                    .condition(and!(
                        Port::F.workspace.equals(workspace),
                        Port::F.host.equals(host_uuid),
                        Port::F.port.equals(port)
                    ))
                    .optional()
                    .await?;

                port_uuid = Some(if let Some((port_uuid,)) = p {
                    port_uuid
                } else {
                    insert!(&mut *tx, PortInsert)
                        .return_primary_key()
                        .single(&PortInsert {
                            uuid: Uuid::new_v4(),
                            port: 0,
                            protocol: PortProtocol::Unknown,
                            host: ForeignModelByField::Key(host_uuid),
                            comment: "".to_string(),
                            workspace: ForeignModelByField::Key(workspace),
                        })
                        .await?
                });
            }

            insert!(tx, ServiceInsert)
                .single(&ServiceInsert {
                    uuid: Uuid::new_v4(),
                    name: name.to_string(),
                    version: None,
                    host: ForeignModelByField::Key(host_uuid),
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(workspace),
                    port: port_uuid.map(ForeignModelByField::Key),
                    certainty,
                })
                .await?;
            true
        };

        guard.commit().await?;
        Ok(res)
    }
}

impl Domain {
    /// Insert an aggregated domain if it doesn't exist yet.
    ///
    /// Returns the domain's primary key.
    pub async fn get_or_create(
        executor: impl Executor<'_>,
        workspace: Uuid,
        domain: &str,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let uuid = if let Some((uuid,)) = query!(&mut *tx, (Domain::F.uuid,))
            .condition(and![
                Domain::F.workspace.equals(workspace),
                Domain::F.domain.equals(domain)
            ])
            .optional()
            .await?
        {
            uuid
        } else {
            insert!(tx, Domain)
                .return_primary_key()
                .single(&DomainInsert {
                    uuid: Uuid::new_v4(),
                    domain: domain.to_string(),
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(workspace),
                })
                .await?
        };

        guard.commit().await?;
        Ok(uuid)
    }
}

impl Host {
    /// Insert an aggregated host if it doesn't exist yet.
    ///
    /// Returns the host's primary key.
    pub async fn get_or_create(
        executor: impl Executor<'_>,
        workspace: Uuid,
        ip_addr: IpNetwork,
        os_type: OsType,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let uuid = if let Some((uuid,)) = query!(&mut *tx, (Host::F.uuid,))
            .condition(and![
                Host::F.workspace.equals(workspace),
                Host::F.ip_addr.equals(ip_addr),
            ])
            .optional()
            .await?
        {
            uuid
        } else {
            insert!(&mut *tx, Host)
                .return_primary_key()
                .single(&HostInsert {
                    uuid: Uuid::new_v4(),
                    ip_addr,
                    os_type,
                    response_time: None,
                    comment: "".to_string(),
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
