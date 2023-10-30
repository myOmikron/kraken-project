//! This module holds the aggregated data of ports

use std::collections::HashMap;

use actix_web::get;
use actix_web::web::{Data, Json, Path, Query};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use rorm::conditions::{BoxedCondition, Condition, DynamicCollection};
use rorm::{and, query, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::hosts::SimpleHost;
use crate::api::handler::{
    get_page_params, ApiError, ApiResult, PageParams, PathUuid, PortResultsPage, SimpleTag, TagType,
};
use crate::models::{
    GlobalTag, Host, Port, PortGlobalTag, PortProtocol, PortWorkspaceTag, Workspace, WorkspaceTag,
};
use crate::query_tags;

/// Query parameters for filtering the ports to get
#[derive(Deserialize, IntoParams)]
pub struct GetAllPortsQuery {
    /// Only get ports associated with a specific host
    pub host: Option<Uuid>,
}

/// The simple representation of a port
#[derive(Serialize, ToSchema)]
pub struct SimplePort {
    /// Uuid of the port
    pub uuid: Uuid,
    /// Port number
    #[schema(example = 1337)]
    pub port: u16,
    /// Port protocol
    pub protocol: PortProtocol,
    /// The host this port is assigned to
    pub host: Uuid,
    /// A comment to the port
    pub comment: String,
    /// The workspace this port is linked to
    pub workspace: Uuid,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// The full representation of a port
#[derive(Serialize, ToSchema)]
pub struct FullPort {
    /// Uuid of the port
    pub uuid: Uuid,
    /// Port number
    #[schema(example = 1337)]
    pub port: u16,
    /// Port protocol
    pub protocol: PortProtocol,
    /// The host this port is assigned to
    pub host: SimpleHost,
    /// A comment to the port
    pub comment: String,
    /// The tags this port is linked to
    pub tags: Vec<SimpleTag>,
    /// The workspace this port is linked to
    pub workspace: Uuid,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// List the ports of a workspace
#[utoipa::path(
    tag = "Ports",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all ports of a workspace", body = PortResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams, GetAllPortsQuery),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/ports")]
pub async fn get_all_ports(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    filter_params: Query<GetAllPortsQuery>,
    db: Data<Database>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<PortResultsPage>> {
    let path = path.into_inner();

    let mut tx = db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (limit, offset) = get_page_params(page_params).await?;

    fn build_condition(workspace: Uuid, filter_params: &GetAllPortsQuery) -> BoxedCondition<'_> {
        match filter_params {
            GetAllPortsQuery { host: Some(host) } => and![
                Port::F.workspace.equals(workspace),
                Port::F.host.equals(*host)
            ]
            .boxed(),
            GetAllPortsQuery { host: None } => Port::F.workspace.equals(workspace).boxed(),
        }
    }

    let (total,) = query!(&mut tx, (Port::F.uuid.count(),))
        .condition(build_condition(path.uuid, &filter_params))
        .one()
        .await?;

    let ports: Vec<_> = query!(
        &mut tx,
        (
            Port::F.uuid,
            Port::F.port,
            Port::F.protocol,
            Port::F.comment,
            Port::F.created_at,
            Port::F.host as Host,
            Port::F.workspace
        )
    )
    .condition(build_condition(path.uuid, &filter_params))
    .order_desc(Port::F.created_at)
    .limit(limit)
    .offset(offset)
    .all()
    .await?;

    let mut tags = HashMap::new();

    query_tags!(
        tags,
        tx,
        (
            PortWorkspaceTag::F.workspace_tag as WorkspaceTag,
            PortWorkspaceTag::F.port
        ),
        PortWorkspaceTag::F.port,
        (
            PortGlobalTag::F.global_tag as GlobalTag,
            PortGlobalTag::F.port
        ),
        PortGlobalTag::F.port,
        ports.iter().map(|x| x.0)
    );

    let items = ports
        .into_iter()
        .map(
            |(uuid, port, protocol, comment, created_at, host, workspace)| FullPort {
                uuid,
                port: u16::from_ne_bytes(port.to_ne_bytes()),
                protocol,
                comment,
                host: SimpleHost {
                    uuid: host.uuid,
                    ip_addr: host.ip_addr.to_string(),
                    os_type: host.os_type,
                    workspace: *host.workspace.key(),
                    comment: host.comment,
                    created_at: host.created_at,
                },
                workspace: *workspace.key(),
                tags: tags.remove(&uuid).unwrap_or_default(),
                created_at,
            },
        )
        .collect();

    tx.commit().await?;

    Ok(Json(PortResultsPage {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// The path parameter of a port
#[derive(Deserialize, IntoParams)]
pub struct PathPort {
    /// The workspace's uuid
    pub w_uuid: Uuid,
    /// The port's uuid
    pub p_uuid: Uuid,
}

/// Retrieve all information about a single port
#[utoipa::path(
    tag = "Ports",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved the selected port", body = FullPort),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathPort),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/ports/{p_uuid}")]
pub async fn get_port(
    path: Path<PathPort>,
    db: Data<Database>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullPort>> {
    let mut tx = db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges)?;
    }

    let port = query!(&mut tx, Port)
        .condition(and!(
            Port::F.workspace.equals(path.w_uuid),
            Port::F.uuid.equals(path.p_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let host = query!(&mut tx, Host)
        .condition(Host::F.uuid.equals(*port.host.key()))
        .one()
        .await?;

    let mut tags: Vec<_> = query!(&mut tx, (PortGlobalTag::F.global_tag as GlobalTag,))
        .condition(PortGlobalTag::F.port.equals(path.p_uuid))
        .stream()
        .map_ok(|(x,)| SimpleTag {
            uuid: x.uuid,
            name: x.name,
            color: x.color.into(),
            tag_type: TagType::Global,
        })
        .try_collect()
        .await?;

    let global_tags: Vec<_> = query!(
        &mut tx,
        (PortWorkspaceTag::F.workspace_tag as WorkspaceTag,)
    )
    .condition(PortWorkspaceTag::F.port.equals(path.p_uuid))
    .stream()
    .map_ok(|(x,)| SimpleTag {
        uuid: x.uuid,
        name: x.name,
        color: x.color.into(),
        tag_type: TagType::Workspace,
    })
    .try_collect()
    .await?;

    tags.extend(global_tags);

    tx.commit().await?;

    Ok(Json(FullPort {
        uuid: port.uuid,
        port: u16::from_ne_bytes(port.port.to_ne_bytes()),
        protocol: port.protocol,
        host: SimpleHost {
            uuid: host.uuid,
            ip_addr: host.ip_addr.to_string(),
            os_type: host.os_type,
            comment: host.comment,
            workspace: path.w_uuid,
            created_at: host.created_at,
        },
        comment: port.comment,
        tags,
        workspace: path.w_uuid,
        created_at: port.created_at,
    }))
}
