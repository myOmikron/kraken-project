use std::collections::HashMap;

use actix_web::get;
use actix_web::web::{Json, Path};
use chrono::Utc;
use futures::TryStreamExt;
use rorm::{and, query, FieldAccess, Model};
use uuid::Uuid;

use crate::api::extractors::BearerToken;
use crate::api::handler::common::error::{ApiError, ApiResult};
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::data_export::schema::{
    AggregatedDomain, AggregatedHost, AggregatedPort, AggregatedRelation, AggregatedService,
    AggregatedWorkspace,
};
use crate::chan::global::GLOBAL;
use crate::models::{
    Domain, DomainDomainRelation, DomainGlobalTag, DomainHostRelation, DomainWorkspaceTag, Host,
    HostGlobalTag, HostWorkspaceTag, Port, PortGlobalTag, PortWorkspaceTag, Service,
    ServiceGlobalTag, ServiceWorkspaceTag, WorkspaceAccessToken,
};

#[utoipa::path(
    tag = "Data Export",
    context_path = "/api/v1/export",
    responses(
        (status = 200, description = "All hosts in the workspace", body = AggregatedWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("bearer_token" = []))
)]
#[get("/workspace/{uuid}")]
pub(crate) async fn export_workspace(
    path: Path<PathUuid>,
    token: BearerToken,
) -> ApiResult<Json<AggregatedWorkspace>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    // Check access
    query!(&mut tx, (WorkspaceAccessToken::F.id,))
        .condition(and![
            WorkspaceAccessToken::F.token.equals(token.as_str()),
            WorkspaceAccessToken::F.workspace.equals(path.uuid),
            WorkspaceAccessToken::F.expires_at.greater_than(Utc::now()),
        ])
        .optional()
        .await?
        .ok_or(ApiError::MissingPrivileges)?;

    // Query all models without joins
    let mut hosts: HashMap<Uuid, AggregatedHost> = query!(&mut tx, Host)
        .condition(Host::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|host| (host.uuid, host.into()))
        .try_collect()
        .await?;
    let mut ports: HashMap<Uuid, AggregatedPort> = query!(&mut tx, Port)
        .condition(Port::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|port| (port.uuid, port.into()))
        .try_collect()
        .await?;
    let mut services: HashMap<Uuid, AggregatedService> = query!(&mut tx, Service)
        .condition(Service::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|service| (service.uuid, convert_service(service, &ports)))
        .try_collect()
        .await?;
    let mut domains: HashMap<Uuid, AggregatedDomain> = query!(&mut tx, Domain)
        .condition(Domain::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|domain| (domain.uuid, domain.into()))
        .try_collect()
        .await?;
    let mut relations = HashMap::new();

    query!(&mut tx, DomainDomainRelation)
        .condition(DomainDomainRelation::F.workspace.equals(path.uuid))
        .stream()
        .try_for_each(|x| {
            relations.insert(
                x.uuid,
                AggregatedRelation::DomainDomain {
                    source: *x.source.key(),
                    destination: *x.destination.key(),
                },
            );
            if let Some(domain) = domains.get_mut(x.source.key()) {
                domain.destinations.push(x.uuid);
            }
            if let Some(domain) = domains.get_mut(x.destination.key()) {
                domain.sources.push(x.uuid);
            }
            async { Ok(()) }
        })
        .await?;

    query!(&mut tx, DomainHostRelation)
        .condition(DomainHostRelation::F.workspace.equals(path.uuid))
        .stream()
        .try_for_each(|x| {
            relations.insert(
                x.uuid,
                AggregatedRelation::DomainHost {
                    domain: *x.domain.key(),
                    host: *x.host.key(),
                    is_direct: x.is_direct,
                },
            );
            if let Some(host) = hosts.get_mut(x.host.key()) {
                host.domains.push(x.uuid);
            }
            if let Some(domain) = domains.get_mut(x.domain.key()) {
                domain.hosts.push(x.uuid);
            }
            async { Ok(()) }
        })
        .await?;

    // Resolve BackRefs manually
    for service in services.values() {
        if let Some(host) = hosts.get_mut(&service.host) {
            host.services.push(service.uuid);
        }
        if let Some(port) = service.port.as_ref() {
            if let Some(port) = ports.get_mut(port) {
                port.services.push(service.uuid);
            }
        }
    }
    for port in ports.values() {
        if let Some(host) = hosts.get_mut(&port.host) {
            host.ports.push(port.uuid);
        }
    }

    // Query all tags
    macro_rules! query_tags {
        ($owner:ident, $owner_set:ident, $GlobalTag:ident, $WorkspaceTag:ident) => {
            let mut stream = query!(
                &mut tx,
                ($GlobalTag::F.$owner.uuid, $GlobalTag::F.global_tag.name)
            )
            .condition($GlobalTag::F.$owner.workspace.equals(path.uuid))
            .stream();
            while let Some((owner_uuid, name)) = stream.try_next().await? {
                if let Some(owner) = $owner_set.get_mut(&owner_uuid) {
                    owner.tags.global_tags.push(name);
                }
            }
            drop(stream);
            let mut stream = query!(
                &mut tx,
                ($WorkspaceTag::F.$owner, $WorkspaceTag::F.workspace_tag.name)
            )
            .condition($WorkspaceTag::F.workspace_tag.workspace.equals(path.uuid))
            .stream();
            while let Some((owner_uuid, name)) = stream.try_next().await? {
                if let Some(owner) = $owner_set.get_mut(owner_uuid.key()) {
                    owner.tags.local_tags.push(name);
                }
            }
            drop(stream);
        };
    }
    query_tags!(host, hosts, HostGlobalTag, HostWorkspaceTag);
    query_tags!(port, ports, PortGlobalTag, PortWorkspaceTag);
    query_tags!(service, services, ServiceGlobalTag, ServiceWorkspaceTag);
    query_tags!(domain, domains, DomainGlobalTag, DomainWorkspaceTag);

    tx.commit().await?;
    Ok(Json(AggregatedWorkspace {
        hosts,
        ports,
        services,
        domains,
        relations,
    }))
}

impl From<Host> for AggregatedHost {
    fn from(value: Host) -> Self {
        let Host {
            uuid,
            ip_addr,
            os_type,
            response_time,
            ports: _,
            services: _,
            domains: _,
            comment,
            workspace: _,
            workspace_tags: _,
            global_tags: _,
            created_at,
            certainty,
        } = value;
        // DON'T just ignore new fields with `: _`
        // Make sure you export the field in some other way!

        Self {
            uuid,
            ip_addr,
            os_type,
            response_time,
            certainty,
            ports: Vec::new(),
            services: Vec::new(),
            domains: Vec::new(),
            comment,
            tags: Default::default(),
            created_at,
        }
    }
}
impl From<Port> for AggregatedPort {
    fn from(value: Port) -> Self {
        let Port {
            uuid,
            port,
            protocol,
            host,
            services: _,
            comment,
            workspace: _,
            global_tags: _,
            workspace_tags: _,
            created_at,
            certainty,
        } = value;
        // DON'T just ignore new fields with `: _`
        // Make sure you export the field in some other way!

        Self {
            uuid,
            port: port as u16,
            protocol,
            host: *host.key(),
            services: Vec::new(),
            certainty,
            comment,
            tags: Default::default(),
            created_at,
        }
    }
}

pub fn convert_service(
    service: Service,
    ports: &HashMap<Uuid, AggregatedPort>,
) -> AggregatedService {
    let Service {
        uuid,
        name,
        version,
        host,
        port,
        protocols,
        comment,
        certainty,
        workspace: _,
        workspace_tags: _,
        global_tags: _,
        created_at,
    } = service;
    // DON'T just ignore new fields with `: _`
    // Make sure you export the field in some other way!

    AggregatedService {
        uuid,
        name,
        version,
        host: *host.key(),
        port: port.as_ref().map(|port| *port.key()),
        protocols: port
            .and_then(|port| ports.get(port.key()))
            .map(|port| port.protocol.decode_service(protocols)),
        comment,
        certainty,
        tags: Default::default(),
        created_at,
    }
}
impl From<Domain> for AggregatedDomain {
    fn from(value: Domain) -> Self {
        let Domain {
            uuid,
            domain,
            comment,
            hosts: _,
            sources: _,
            destinations: _,
            workspace: _,
            workspace_tags: _,
            global_tags: _,
            created_at,
            certainty,
        } = value;
        // DON'T just ignore new fields with `: _`
        // Make sure you export the field in some other way!

        Self {
            uuid,
            domain,
            hosts: Vec::new(),
            sources: Vec::new(),
            destinations: Vec::new(),
            certainty,
            comment,
            tags: Default::default(),
            created_at,
        }
    }
}
