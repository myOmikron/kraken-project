//! This module holds the aggregated data of ports

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
    session: Session,
) -> ApiResult<Json<PortResultsPage>> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
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
