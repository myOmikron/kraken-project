use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::ErrorKind;

use actix_web::get;
use actix_web::post;
use actix_web::web::Json;
use actix_web::web::Path;
use futures::TryStreamExt;
use log::error;
use rorm::db::Executor;
use rorm::imr::Annotation;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::files::utils::media_file_path;
use crate::api::handler::files::utils::media_thumbnail_path;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::CloneWorkspaceRequest;
use crate::api::handler::workspaces::schema::FullWorkspace;
use crate::api::handler::workspaces::schema::ListWorkspaces;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::api::handler::workspaces::utils::get_workspace_unchecked;
use crate::api::handler::workspaces::utils::insert_models;
use crate::chan::global::GLOBAL;
use crate::models::Domain;
use crate::models::DomainDomainRelation;
use crate::models::DomainHostRelation;
use crate::models::Finding;
use crate::models::FindingAffected;
use crate::models::FindingDetails;
use crate::models::Host;
use crate::models::HttpService;
use crate::models::MediaFile;
use crate::models::Port;
use crate::models::Service;
use crate::models::Workspace;

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the workspace with the given id", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}")]
pub async fn get_workspace_admin(req: Path<PathUuid>) -> ApiResult<Json<FullWorkspace>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspace = get_workspace_unchecked(req.uuid, &mut tx).await;

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// Retrieve all workspaces
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all workspaces", body = ListWorkspaces),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces_admin() -> ApiResult<Json<ListWorkspaces>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspaces = query!(
        &mut tx,
        (
            Workspace::F.uuid,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.created_at,
            Workspace::F.archived,
            Workspace::F.owner.uuid,
            Workspace::F.owner.username,
            Workspace::F.owner.display_name
        )
    )
    .all()
    .await?;

    tx.commit().await?;

    Ok(Json(ListWorkspaces {
        workspaces: workspaces
            .into_iter()
            .map(
                |(
                    uuid,
                    name,
                    description,
                    created_at,
                    archived,
                    by_uuid,
                    username,
                    display_name,
                )| {
                    SimpleWorkspace {
                        uuid,
                        name,
                        description,
                        owner: SimpleUser {
                            uuid: by_uuid,
                            username,
                            display_name,
                        },
                        created_at,
                        archived,
                    }
                },
            )
            .collect(),
    }))
}

/// Clones a workspace by id
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the cloned workspace", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/clone")]
pub async fn clone_workspace(
    req: Path<PathUuid>,
    Json(request): Json<CloneWorkspaceRequest>,
) -> ApiResult<Json<FullWorkspace>> {
    let old_workspace_uuid = req.uuid;
    let mut tx = GLOBAL.db.start_transaction().await?;

    let new_workspace_uuid = {
        let (name, description, old_owner) = query!(
            &mut tx,
            (
                Workspace::F.name,
                Workspace::F.description,
                Workspace::F.owner,
            )
        )
        .condition(Workspace::F.uuid.equals(old_workspace_uuid))
        .optional()
        .await?
        .ok_or(ApiError::NotFound)?;

        Workspace::insert(
            &mut tx,
            format!("{name} (Copy)"),
            description,
            request.new_owner.unwrap_or(*old_owner.key()),
        )
        .await?
    };

    let mut hosts = query!(&mut tx, Host)
        .condition(Host::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    let mut hosts_lookup = HashMap::with_capacity(hosts.len());
    for host in &mut hosts {
        let new_uuid = Uuid::new_v4();
        hosts_lookup.insert(host.uuid, new_uuid);
        host.uuid = new_uuid;

        host.workspace = ForeignModelByField::Key(new_workspace_uuid);
    }
    insert_models(&mut tx, hosts).await?;

    let mut domains = query!(&mut tx, Domain)
        .condition(Domain::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    let mut domains_lookup = HashMap::with_capacity(domains.len());
    for domain in &mut domains {
        let new_uuid = Uuid::new_v4();
        domains_lookup.insert(domain.uuid, new_uuid);
        domain.uuid = new_uuid;

        domain.workspace = ForeignModelByField::Key(new_workspace_uuid);
    }
    insert_models(&mut tx, domains).await?;

    let mut ports = query!(&mut tx, Port)
        .condition(Port::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    let mut ports_lookup = HashMap::with_capacity(ports.len());
    for port in &mut ports {
        let new_uuid = Uuid::new_v4();
        ports_lookup.insert(port.uuid, new_uuid);
        port.uuid = new_uuid;

        port.workspace = ForeignModelByField::Key(new_workspace_uuid);

        port.host = ForeignModelByField::Key(hosts_lookup[port.host.key()]);
    }
    insert_models(&mut tx, ports).await?;

    let mut services = query!(&mut tx, Service)
        .condition(Service::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    let mut services_lookup = HashMap::with_capacity(services.len());
    for service in &mut services {
        let new_uuid = Uuid::new_v4();
        services_lookup.insert(service.uuid, new_uuid);
        service.uuid = new_uuid;

        service.workspace = ForeignModelByField::Key(new_workspace_uuid);

        service.host = ForeignModelByField::Key(hosts_lookup[service.host.key()]);
        service.port = service
            .port
            .as_ref()
            .map(|port_fm| ForeignModelByField::Key(ports_lookup[port_fm.key()]));
    }
    insert_models(&mut tx, services).await?;

    let mut http_services = query!(&mut tx, HttpService)
        .condition(HttpService::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    let mut http_services_lookup = HashMap::with_capacity(http_services.len());
    for http_service in &mut http_services {
        let new_uuid = Uuid::new_v4();
        http_services_lookup.insert(http_service.uuid, new_uuid);
        http_service.uuid = new_uuid;

        http_service.workspace = ForeignModelByField::Key(new_workspace_uuid);

        http_service.host = ForeignModelByField::Key(hosts_lookup[http_service.host.key()]);
        http_service.port = ForeignModelByField::Key(ports_lookup[http_service.port.key()]);
        http_service.domain = http_service
            .domain
            .as_ref()
            .map(|domain_fm| ForeignModelByField::Key(domains_lookup[domain_fm.key()]));
    }
    insert_models(&mut tx, http_services).await?;

    let mut relations = query!(&mut tx, DomainDomainRelation)
        .condition(DomainDomainRelation::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    for relation in &mut relations {
        relation.uuid = Uuid::new_v4();

        relation.workspace = ForeignModelByField::Key(new_workspace_uuid);

        relation.source = ForeignModelByField::Key(domains_lookup[relation.source.key()]);
        relation.destination = ForeignModelByField::Key(domains_lookup[relation.destination.key()]);
    }
    insert_models(&mut tx, relations).await?;

    let mut relations = query!(&mut tx, DomainHostRelation)
        .condition(DomainHostRelation::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    for relation in &mut relations {
        relation.uuid = Uuid::new_v4();

        relation.workspace = ForeignModelByField::Key(new_workspace_uuid);

        relation.domain = ForeignModelByField::Key(domains_lookup[relation.domain.key()]);
        relation.host = ForeignModelByField::Key(hosts_lookup[relation.host.key()]);
    }
    insert_models(&mut tx, relations).await?;

    let mut files = query!(&mut tx, MediaFile)
        .condition(MediaFile::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    let mut files_lookup = HashMap::with_capacity(files.len());
    for file in &mut files {
        let new_uuid = Uuid::new_v4();
        files_lookup.insert(file.uuid, new_uuid);
        file.uuid = new_uuid;

        file.workspace = Some(ForeignModelByField::Key(new_workspace_uuid));
    }
    insert_models(&mut tx, files).await?;

    let mut finding_details = Vec::new();
    query!(&mut tx, (Finding::F.details as FindingDetails,))
        .condition(Finding::F.workspace.equals(old_workspace_uuid))
        .stream()
        .try_for_each(|(details,)| {
            finding_details.push(details);
            async move { Ok(()) }
        })
        .await?;
    query!(&mut tx, (FindingAffected::F.details as FindingDetails,))
        .condition(FindingAffected::F.workspace.equals(old_workspace_uuid))
        .stream()
        .try_for_each(|(details,)| {
            finding_details.push(details);
            async move { Ok(()) }
        })
        .await?;
    let mut finding_details_lookup = HashMap::with_capacity(finding_details.len());
    for finding_detail in &mut finding_details {
        let new_uuid = Uuid::new_v4();
        finding_details_lookup.insert(finding_detail.uuid, new_uuid);
        finding_detail.uuid = new_uuid;

        finding_detail.screenshot = finding_detail
            .screenshot
            .as_ref()
            .map(|file_fm| ForeignModelByField::Key(files_lookup[file_fm.key()]));
        finding_detail.log_file = finding_detail
            .log_file
            .as_ref()
            .map(|file_fm| ForeignModelByField::Key(files_lookup[file_fm.key()]));
    }
    insert_models(&mut tx, finding_details).await?;

    let mut findings = query!(&mut tx, Finding)
        .condition(Finding::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    let mut findings_lookup = HashMap::with_capacity(findings.len());
    for finding in &mut findings {
        let new_uuid = Uuid::new_v4();
        findings_lookup.insert(finding.uuid, new_uuid);
        finding.uuid = new_uuid;

        finding.workspace = ForeignModelByField::Key(new_workspace_uuid);

        finding.details = ForeignModelByField::Key(finding_details_lookup[finding.details.key()]);
    }
    insert_models(&mut tx, findings).await?;

    let mut finding_affecteds = query!(&mut tx, FindingAffected)
        .condition(FindingAffected::F.workspace.equals(old_workspace_uuid))
        .all()
        .await?;
    for finding_affected in &mut finding_affecteds {
        finding_affected.uuid = Uuid::new_v4();

        finding_affected.workspace = ForeignModelByField::Key(new_workspace_uuid);

        finding_affected.finding =
            ForeignModelByField::Key(findings_lookup[finding_affected.finding.key()]);
        finding_affected.details = finding_affected
            .details
            .as_ref()
            .map(|details_fm| ForeignModelByField::Key(finding_details_lookup[details_fm.key()]));
        finding_affected.domain = finding_affected
            .domain
            .as_ref()
            .map(|domain_fm| ForeignModelByField::Key(domains_lookup[domain_fm.key()]));
        finding_affected.host = finding_affected
            .host
            .as_ref()
            .map(|host_fm| ForeignModelByField::Key(hosts_lookup[host_fm.key()]));
        finding_affected.port = finding_affected
            .port
            .as_ref()
            .map(|port_fm| ForeignModelByField::Key(ports_lookup[port_fm.key()]));
        finding_affected.service = finding_affected
            .service
            .as_ref()
            .map(|service_fm| ForeignModelByField::Key(services_lookup[service_fm.key()]));
        finding_affected.http_service =
            finding_affected
                .http_service
                .as_ref()
                .map(|http_service_fm| {
                    ForeignModelByField::Key(http_services_lookup[http_service_fm.key()])
                });
    }
    insert_models(&mut tx, finding_affecteds).await?;

    let workspace = get_workspace_unchecked(req.uuid, &mut tx).await?;

    tokio::task::spawn_blocking(move || {
        for (old_uuid, new_uuid) in files_lookup {
            fs::copy(media_file_path(old_uuid), media_file_path(new_uuid))?;
            fs::copy(
                media_thumbnail_path(old_uuid),
                media_thumbnail_path(new_uuid),
            )
            .or_else(|error| match error.kind() {
                ErrorKind::NotFound => Ok(0),
                _ => Err(error),
            })?;
        }
        io::Result::Ok(())
    })
    .await
    .map_err(|_| {
        error!("Failed to join blocking task");
        ApiError::InternalServerError
    })?
    .map_err(|io_error| {
        error!("Failed to copy files: {io_error}");
        ApiError::InternalServerError
    })?;

    tx.commit().await?;
    Ok(Json(workspace))
}
