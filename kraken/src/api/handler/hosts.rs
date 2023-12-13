//! The aggregated data of hosts

use std::collections::HashMap;

use actix_web::web::{Json, Path};
use actix_web::{get, post, put, HttpResponse};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use ipnetwork::IpNetwork;
use rorm::conditions::DynamicCollection;
use rorm::db::sql::value::Value;
use rorm::model::PatchSelector;
use rorm::prelude::*;
use rorm::{and, insert, query, update};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::aggregation_source::{FullAggregationSource, SimpleAggregationSource};
use crate::api::handler::{
    get_page_params, ApiError, ApiResult, HostResultsPage, PageParams, PathUuid, SimpleTag,
    TagType, UuidResponse,
};
use crate::chan::{AggregationType, WsMessage, GLOBAL};
use crate::models::{
    AggregationSource, AggregationTable, GlobalTag, Host, HostGlobalTag, HostWorkspaceTag,
    ManualHost, ManualHostCertainty, OsType, Workspace, WorkspaceTag,
};
use crate::modules::filter::{GlobalAST, HostAST};
use crate::modules::raw_query::RawQueryBuilder;
use crate::query_tags;

/// Query parameters for filtering the hosts to get
#[derive(Deserialize, ToSchema)]
pub struct GetAllHostsQuery {
    /// The parameters controlling the page to query
    #[serde(flatten)]
    pub page: PageParams,

    /// An optional general filter to apply
    pub global_filter: Option<String>,

    /// An optional host specific filter to apply
    pub host_filter: Option<String>,
}

/// The simple representation of a host
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleHost {
    /// The primary key of the host
    pub uuid: Uuid,
    /// The ip address of the host
    #[schema(example = "172.0.0.1")]
    pub ip_addr: String,
    /// The type of OS
    pub os_type: OsType,
    /// A comment
    pub comment: String,
    /// The workspace this host is in
    pub workspace: Uuid,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// The full representation of a host
#[derive(Serialize, Debug, ToSchema)]
pub struct FullHost {
    /// The primary key of the host
    pub uuid: Uuid,
    /// The ip address of the host
    #[schema(example = "172.0.0.1")]
    pub ip_addr: String,
    /// The type of OS
    pub os_type: OsType,
    /// A comment
    pub comment: String,
    /// The workspace this host is in
    pub workspace: Uuid,
    /// The list of tags this host has attached to
    pub tags: Vec<SimpleTag>,
    /// The number of attacks which found this host
    pub sources: SimpleAggregationSource,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// Retrieve all hosts.
///
/// Hosts are created out of aggregating data or by user input.
/// They represent a single host and can be created by providing an IP address
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "All hosts in the workspace", body = HostResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = GetAllHostsQuery,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/hosts/all")]
pub(crate) async fn get_all_hosts(
    path: Path<PathUuid>,
    params: Json<GetAllHostsQuery>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<HostResultsPage>> {
    let path = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (limit, offset) = get_page_params(params.page).await?;

    let global_filter = params
        .global_filter
        .as_deref()
        .map(GlobalAST::parse)
        .transpose()?
        .unwrap_or_default();

    let host_filter = params
        .host_filter
        .as_deref()
        .map(HostAST::parse)
        .transpose()?
        .unwrap_or_default();

    let mut count_query = RawQueryBuilder::new((Host::F.uuid.count(),));
    let mut select_query = RawQueryBuilder::new(PatchSelector::<Host>::new());

    host_filter.apply_to_query(&global_filter, &mut count_query);
    host_filter.apply_to_query(&global_filter, &mut select_query);

    count_query.append_eq_condition(Host::F.workspace, Value::Uuid(path.uuid));
    select_query.append_eq_condition(Host::F.workspace, Value::Uuid(path.uuid));

    select_query.order_desc(Host::F.created_at);
    select_query.limit_offset(limit, offset);

    let (total,) = count_query.one(&mut tx).await?;
    let hosts: Vec<_> = select_query.stream(&mut tx).try_collect().await?;

    let mut tags = HashMap::new();
    query_tags!(
        tags,
        tx,
        (
            HostWorkspaceTag::F.workspace_tag as WorkspaceTag,
            HostWorkspaceTag::F.host
        ),
        HostWorkspaceTag::F.host,
        (
            HostGlobalTag::F.global_tag as GlobalTag,
            HostGlobalTag::F.host
        ),
        HostGlobalTag::F.host,
        hosts.iter().map(|x| x.uuid)
    );

    let mut sources = SimpleAggregationSource::query(
        &mut tx,
        path.uuid,
        AggregationTable::Host,
        hosts.iter().map(|x| x.uuid),
    )
    .await?;

    tx.commit().await?;

    Ok(Json(HostResultsPage {
        items: hosts
            .into_iter()
            .map(|x| FullHost {
                uuid: x.uuid,
                ip_addr: x.ip_addr.ip().to_string(),
                comment: x.comment,
                os_type: x.os_type,
                workspace: *x.workspace.key(),
                tags: tags.remove(&x.uuid).unwrap_or_default(),
                sources: sources.remove(&x.uuid).unwrap_or_default(),
                created_at: x.created_at,
            })
            .collect(),
        limit,
        offset,
        total: total as u64,
    }))
}

/// The path parameter of a host
#[derive(Deserialize, IntoParams)]
pub struct PathHost {
    w_uuid: Uuid,
    h_uuid: Uuid,
}

/// Retrieve all information about a single host
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved the selected host", body = FullHost),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHost),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/hosts/{h_uuid}")]
pub async fn get_host(
    path: Path<PathHost>,

    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullHost>> {
    let path = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges)?;
    }

    let host = query!(&mut tx, Host)
        .condition(and!(
            Host::F.workspace.equals(path.w_uuid),
            Host::F.uuid.equals(path.h_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let mut tags: Vec<_> = query!(&mut tx, (HostGlobalTag::F.global_tag as GlobalTag,))
        .condition(HostGlobalTag::F.host.equals(host.uuid))
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
        (HostWorkspaceTag::F.workspace_tag as WorkspaceTag,)
    )
    .condition(HostWorkspaceTag::F.host.equals(host.uuid))
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

    let sources = query!(&mut tx, (AggregationSource::F.source_type,))
        .condition(AggregationSource::F.aggregated_uuid.equals(host.uuid))
        .stream()
        .map_ok(|(x,)| x)
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(FullHost {
        uuid: host.uuid,
        ip_addr: host.ip_addr.ip().to_string(),
        workspace: *host.workspace.key(),
        os_type: host.os_type,
        comment: host.comment,
        tags,
        sources,
        created_at: host.created_at,
    }))
}

/// The request to manually add a host
#[derive(Deserialize, ToSchema)]
pub struct CreateHostRequest {
    /// The host's ip address
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip_addr: IpNetwork,

    /// Whether the host should exist right now or existed at some point
    pub certainty: ManualHostCertainty,
}

/// Manually add a host
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Host was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateHostRequest,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/hosts")]
pub async fn create_host(
    req: Json<CreateHostRequest>,
    path: Path<PathUuid>,

    SessionUser(user): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let CreateHostRequest { ip_addr, certainty } = req.into_inner();
    let PathUuid { uuid: workspace } = path.into_inner();
    Ok(Json(UuidResponse {
        uuid: ManualHost::insert(&GLOBAL.db, workspace, user, ip_addr, certainty).await?,
    }))
}

/// The request to update a host
#[derive(Deserialize, ToSchema)]
pub struct UpdateHostRequest {
    comment: Option<String>,
    global_tags: Option<Vec<Uuid>>,
    workspace_tags: Option<Vec<Uuid>>,
}

/// Update a host
///
/// You must include at least on parameter
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Host was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateHostRequest,
    params(PathHost),
    security(("api_key" = []))
)]
#[put("/workspaces/{w_uuid}/hosts/{h_uuid}")]
pub async fn update_host(
    req: Json<UpdateHostRequest>,
    path: Path<PathHost>,

    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let path = path.into_inner();
    let req = req.into_inner();

    if req.workspace_tags.is_none() && req.global_tags.is_none() && req.comment.is_none() {
        return Err(ApiError::EmptyJson);
    }

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (Host::F.uuid,))
        .condition(Host::F.uuid.equals(path.h_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(global_tags) = &req.global_tags {
        GlobalTag::exist_all(&mut tx, global_tags.iter().copied())
            .await?
            .ok_or(ApiError::InvalidUuid)?;

        rorm::delete!(&mut tx, HostGlobalTag)
            .condition(HostGlobalTag::F.host.equals(path.h_uuid))
            .await?;

        if !global_tags.is_empty() {
            insert!(&mut tx, HostGlobalTag)
                .return_nothing()
                .bulk(
                    &global_tags
                        .into_iter()
                        .map(|x| HostGlobalTag {
                            uuid: Uuid::new_v4(),
                            host: ForeignModelByField::Key(path.h_uuid),
                            global_tag: ForeignModelByField::Key(*x),
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;
        }
    }

    if let Some(workspace_tags) = &req.workspace_tags {
        WorkspaceTag::exist_all(&mut tx, workspace_tags.iter().copied())
            .await?
            .ok_or(ApiError::InvalidUuid)?;

        rorm::delete!(&mut tx, HostWorkspaceTag)
            .condition(HostWorkspaceTag::F.host.equals(path.h_uuid))
            .await?;

        if !workspace_tags.is_empty() {
            insert!(&mut tx, HostWorkspaceTag)
                .return_nothing()
                .bulk(
                    &workspace_tags
                        .into_iter()
                        .map(|x| HostWorkspaceTag {
                            uuid: Uuid::new_v4(),
                            host: ForeignModelByField::Key(path.h_uuid),
                            workspace_tag: ForeignModelByField::Key(*x),
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;
        }
    }

    if let Some(comment) = req.comment {
        update!(&mut tx, Host)
            .condition(Host::F.uuid.equals(path.h_uuid))
            .set(Host::F.comment, comment)
            .exec()
            .await?;
    }

    tx.commit().await?;

    // Send WS messages
    if let Some(tags) = req.workspace_tags {
        GLOBAL
            .ws
            .message_workspace(
                path.w_uuid,
                WsMessage::UpdatedWorkspaceTags {
                    uuid: path.h_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::Host,
                    tags,
                },
            )
            .await;
    }
    if let Some(tags) = req.global_tags {
        GLOBAL
            .ws
            .message_workspace(
                path.w_uuid,
                WsMessage::UpdatedGlobalTags {
                    uuid: path.h_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::Host,
                    tags,
                },
            )
            .await;
    }

    Ok(HttpResponse::Ok().finish())
}

/// Get all data sources which referenced this host
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The host's sources", body = FullAggregationSource),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHost),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/hosts/{h_uuid}/sources")]
pub async fn get_host_sources(
    path: Path<PathHost>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullAggregationSource>> {
    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }
    let source =
        FullAggregationSource::query(&mut tx, path.w_uuid, AggregationTable::Host, path.h_uuid)
            .await?;
    tx.commit().await?;
    Ok(Json(source))
}
