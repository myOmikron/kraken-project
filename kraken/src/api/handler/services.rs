//! This module holds the aggregated data of services

use std::collections::HashMap;

use actix_toolbox::tb_middleware::Session;
use actix_web::get;
use actix_web::web::{Data, Json, Path, Query};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use rorm::conditions::{BoxedCondition, Condition, DynamicCollection};
use rorm::{and, query, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::hosts::SimpleHost;
use crate::api::handler::ports::SimplePort;
use crate::api::handler::{
    get_page_params, ApiError, ApiResult, PageParams, PathUuid, ServiceResultsPage, SimpleTag,
    TagType,
};
use crate::models::{
    GlobalTag, Host, Port, Service, ServiceCertainty, ServiceGlobalTag, ServiceWorkspaceTag,
    Workspace, WorkspaceTag,
};
use crate::query_tags;

/// Query parameters for filtering the services to get
#[derive(Deserialize, IntoParams)]
pub struct GetAllServicesQuery {
    /// Only get services associated with a specific host
    pub host: Option<Uuid>,
}

/// A simple representation of a service
#[derive(Serialize, ToSchema)]
pub struct SimpleService {
    pub(crate) uuid: Uuid,
    #[schema(example = "postgresql")]
    pub(crate) name: String,
    #[schema(example = "13.0.1")]
    pub(crate) version: Option<String>,
    pub(crate) host: Uuid,
    pub(crate) port: Option<Uuid>,
    #[schema(example = "Holds all relevant information")]
    pub(crate) comment: String,
    pub(crate) workspace: Uuid,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// A full representation of a service
#[derive(Serialize, ToSchema)]
pub struct FullService {
    uuid: Uuid,
    #[schema(example = "postgresql")]
    name: String,
    #[schema(example = "13.0.1")]
    version: Option<String>,
    certainty: ServiceCertainty,
    host: SimpleHost,
    port: Option<SimplePort>,
    #[schema(example = "Holds all relevant information")]
    comment: String,
    workspace: Uuid,
    tags: Vec<SimpleTag>,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// List the services of a workspace
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all services of a workspace", body = ServiceResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams, GetAllServicesQuery),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/services")]
pub async fn get_all_services(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    filter_params: Query<GetAllServicesQuery>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<ServiceResultsPage>> {
    let path = path.into_inner();
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (limit, offset) = get_page_params(page_params).await?;

    fn build_condition(workspace: Uuid, filter_params: &GetAllServicesQuery) -> BoxedCondition<'_> {
        match filter_params {
            GetAllServicesQuery { host: Some(host) } => and![
                Service::F.workspace.equals(workspace),
                Service::F.host.equals(*host)
            ]
            .boxed(),
            GetAllServicesQuery { host: None } => Service::F.workspace.equals(workspace).boxed(),
        }
    }

    let (total,) = query!(&mut tx, (Service::F.uuid.count()))
        .condition(build_condition(path.uuid, &filter_params))
        .one()
        .await?;

    let services = query!(
        &mut tx,
        (
            Service::F.uuid,
            Service::F.name,
            Service::F.version,
            Service::F.certainty,
            Service::F.comment,
            Service::F.created_at,
            Service::F.host as Host,
            Service::F.port,
            Service::F.workspace,
        )
    )
    .condition(build_condition(path.uuid, &filter_params))
    .order_desc(Service::F.created_at)
    .limit(limit)
    .offset(offset)
    .all()
    .await?;

    let mut ports = HashMap::new();
    let p: Vec<_> = services
        .iter()
        .filter_map(|x| x.7.as_ref().map(|y| Port::F.uuid.equals(*y.key())))
        .collect();

    if !p.is_empty() {
        let mut port_stream = query!(&mut tx, Port)
            .condition(DynamicCollection::or(p))
            .stream();

        while let Some(port) = port_stream.try_next().await? {
            ports.insert(
                port.uuid,
                SimplePort {
                    uuid: port.uuid,
                    port: u16::from_ne_bytes(port.port.to_ne_bytes()),
                    protocol: port.protocol,
                    comment: port.comment,
                    created_at: port.created_at,
                    workspace: *port.workspace.key(),
                    host: *port.host.key(),
                },
            );
        }
    }

    let mut tags = HashMap::new();

    query_tags!(
        tags,
        tx,
        (
            ServiceWorkspaceTag::F.workspace_tag as WorkspaceTag,
            ServiceWorkspaceTag::F.service
        ),
        ServiceWorkspaceTag::F.service,
        (
            ServiceGlobalTag::F.global_tag as GlobalTag,
            ServiceGlobalTag::F.service
        ),
        ServiceGlobalTag::F.service,
        services.iter().map(|x| x.0)
    );

    tx.commit().await?;

    let items = services
        .into_iter()
        .map(
            |(uuid, name, version, certainty, comment, created_at, host, port, workspace)| {
                FullService {
                    uuid,
                    name,
                    version,
                    certainty,
                    comment,
                    host: SimpleHost {
                        uuid: host.uuid,
                        ip_addr: host.ip_addr.to_string(),
                        os_type: host.os_type,
                        comment: host.comment,
                        workspace: *host.workspace.key(),
                        created_at: host.created_at,
                    },
                    port: port.map(|y| ports.remove(y.key()).unwrap()),
                    workspace: *workspace.key(),
                    tags: tags.remove(&uuid).unwrap_or_default(),
                    created_at,
                }
            },
        )
        .collect();

    Ok(Json(ServiceResultsPage {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}
