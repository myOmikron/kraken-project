use std::collections::HashMap;
use std::future::ready;

use actix_web::get;
use actix_web::web::Json;
use actix_web::web::Path;
use chrono::Utc;
use futures::TryStreamExt;
use log::error;
use rorm::and;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::extractors::BearerToken;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::data_export::schema::AggregatedDomain;
use crate::api::handler::data_export::schema::AggregatedFinding;
use crate::api::handler::data_export::schema::AggregatedFindingAffected;
use crate::api::handler::data_export::schema::AggregatedHost;
use crate::api::handler::data_export::schema::AggregatedHttpService;
use crate::api::handler::data_export::schema::AggregatedPort;
use crate::api::handler::data_export::schema::AggregatedRelation;
use crate::api::handler::data_export::schema::AggregatedService;
use crate::api::handler::data_export::schema::AggregatedWorkspace;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::AggregationType;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;
use crate::models::Domain;
use crate::models::DomainDomainRelation;
use crate::models::DomainGlobalTag;
use crate::models::DomainHostRelation;
use crate::models::DomainWorkspaceTag;
use crate::models::Finding;
use crate::models::FindingAffected;
use crate::models::FindingFindingCategoryRelation;
use crate::models::Host;
use crate::models::HostGlobalTag;
use crate::models::HostWorkspaceTag;
use crate::models::HttpService;
use crate::models::HttpServiceGlobalTag;
use crate::models::HttpServiceWorkspaceTag;
use crate::models::Port;
use crate::models::PortGlobalTag;
use crate::models::PortWorkspaceTag;
use crate::models::Service;
use crate::models::ServiceGlobalTag;
use crate::models::ServiceWorkspaceTag;
use crate::models::WorkspaceAccessToken;

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
    let mut http_services: HashMap<Uuid, AggregatedHttpService> = query!(&mut tx, HttpService)
        .condition(HttpService::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|http_service| (http_service.uuid, http_service.into()))
        .try_collect()
        .await?;
    let mut domains: HashMap<Uuid, AggregatedDomain> = query!(&mut tx, Domain)
        .condition(Domain::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|domain| (domain.uuid, domain.into()))
        .try_collect()
        .await?;
    let mut relations = HashMap::new();

    // Resolve M2M relations
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
    for http_service in http_services.values() {
        if let Some(host) = hosts.get_mut(&http_service.host) {
            host.http_services.push(http_service.uuid);
        }
        if let Some(port) = ports.get_mut(&http_service.port) {
            port.http_services.push(http_service.uuid);
        }
        if let Some(domain) = http_service.domain.as_ref() {
            if let Some(domain) = domains.get_mut(domain) {
                domain.http_services.push(http_service.uuid);
            }
        }
    }
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
    query_tags!(
        http_service,
        http_services,
        HttpServiceGlobalTag,
        HttpServiceWorkspaceTag
    );
    query_tags!(domain, domains, DomainGlobalTag, DomainWorkspaceTag);

    // Query findings
    let mut categories = query!(
        &mut tx,
        (
            FindingFindingCategoryRelation::F.finding.uuid,
            FindingFindingCategoryRelation::F.category.name,
        )
    )
    .condition(
        FindingFindingCategoryRelation::F
            .finding
            .workspace
            .equals(path.uuid),
    )
    .stream()
    .try_fold(
        HashMap::<Uuid, Vec<String>>::new(),
        |mut map, (uuid, name)| {
            map.entry(uuid).or_default().push(name);
            ready(Ok(map))
        },
    )
    .await?;
    let mut affected = query!(&mut tx, FindingAffected)
        .condition(FindingAffected::F.workspace.equals(path.uuid))
        .stream()
        .err_into::<ApiError>()
        .and_then(|x| {
            let (aggr_uuid, aggr_type) = match &x {
                FindingAffected {
                    domain: Some(fm),
                    host: None,
                    port: None,
                    service: None,
                    http_service: None,
                    ..
                } => (*fm.key(), AggregationType::Domain),
                FindingAffected {
                    domain: None,
                    host: Some(fm),
                    port: None,
                    service: None,
                    http_service: None,
                    ..
                } => (*fm.key(), AggregationType::Host),
                FindingAffected {
                    domain: None,
                    host: None,
                    port: Some(fm),
                    service: None,
                    http_service: None,
                    ..
                } => (*fm.key(), AggregationType::Port),
                FindingAffected {
                    domain: None,
                    host: None,
                    port: None,
                    service: Some(fm),
                    http_service: None,
                    ..
                } => (*fm.key(), AggregationType::Service),
                FindingAffected {
                    domain: None,
                    host: None,
                    port: None,
                    service: None,
                    http_service: Some(fm),
                    ..
                } => (*fm.key(), AggregationType::HttpService),
                FindingAffected {uuid, ..} => {
                    error!("Invalid \"findingaffected\": {uuid}. This means a) invalid db state or b) programmer forgot a match arm.");
                    return ready(Err(ApiError::InternalServerError));
                }
            };
            ready(Ok((*x.finding.key(), aggr_uuid, aggr_type)))
        })
        .try_fold(
            HashMap::<Uuid, HashMap<Uuid, AggregatedFindingAffected>>::new(),
            |mut map, (finding, aggr_uuid, aggr_type)| async move {
                let (details, _) = GLOBAL.editor_cache.finding_affected_export_details.get((finding,aggr_uuid)).await?.unwrap_or_default();
                map.entry(finding).or_default().insert(aggr_uuid, AggregatedFindingAffected {
                    uuid: aggr_uuid,
                    r#type: aggr_type,
                    details,
                });
                Ok(map)
            }
        )
        .await?;
    let findings = query!(
        &mut tx,
        (
            Finding::F.uuid,
            Finding::F.definition.name,
            Finding::F.definition.cve,
            Finding::F.severity,
            Finding::F.created_at,
        )
    )
    .condition(Finding::F.workspace.equals(path.uuid))
    .stream()
    .and_then(|(uuid, name, cve, severity, created_at)| {
        let affected = affected.remove(&uuid).unwrap_or_default();
        let categories = categories.remove(&uuid).unwrap_or_default();
        async move {
            let (details, _) = GLOBAL
                .editor_cache
                .finding_export_details
                .get(uuid)
                .await?
                .unwrap_or_default();
            Ok((
                uuid,
                AggregatedFinding {
                    uuid,
                    name,
                    cve,
                    severity: FromDb::from_db(severity),
                    details,
                    affected,
                    created_at,
                    categories,
                },
            ))
        }
    })
    .try_collect()
    .await?;

    tx.commit().await?;
    Ok(Json(AggregatedWorkspace {
        hosts,
        ports,
        services,
        http_services,
        domains,
        relations,
        findings,
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
            http_services: _,
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
            os_type: FromDb::from_db(os_type),
            response_time,
            certainty: FromDb::from_db(certainty),
            ports: Vec::new(),
            services: Vec::new(),
            http_services: Vec::new(),
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
            http_services: _,
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
            protocol: FromDb::from_db(protocol),
            host: *host.key(),
            services: Vec::new(),
            http_services: Vec::new(),
            certainty: FromDb::from_db(certainty),
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
            .map(|port| port.protocol.into_db().decode_service(protocols)),
        comment,
        certainty: FromDb::from_db(certainty),
        tags: Default::default(),
        created_at,
    }
}
impl From<HttpService> for AggregatedHttpService {
    fn from(value: HttpService) -> Self {
        let HttpService {
            uuid,
            name,
            version,
            base_path,
            tls,
            sni_required,
            domain,
            host,
            port,
            comment,
            certainty,
            workspace_tags: _,
            global_tags: _,
            workspace: _,
            created_at,
        } = value;

        Self {
            uuid,
            name,
            version,
            domain: domain.map(|fm| *fm.key()),
            host: *host.key(),
            port: *port.key(),
            base_path,
            tls,
            sni_required,
            comment,
            certainty: FromDb::from_db(certainty),
            tags: Default::default(),
            created_at,
        }
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
            http_services: _,
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
            http_services: Vec::new(),
            hosts: Vec::new(),
            sources: Vec::new(),
            destinations: Vec::new(),
            certainty: FromDb::from_db(certainty),
            comment,
            tags: Default::default(),
            created_at,
        }
    }
}
