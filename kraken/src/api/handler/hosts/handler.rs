use std::collections::HashMap;

use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use rorm::and;
use rorm::conditions::DynamicCollection;
use rorm::db::sql::value::Value;
use rorm::insert;
use rorm::model::PatchSelector;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::aggregation_source::schema::FullAggregationSource;
use crate::api::handler::aggregation_source::schema::SimpleAggregationSource;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::HostResultsPage;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::common::schema::TagType;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::common::utils::get_page_params;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::findings::schema::ListFindings;
use crate::api::handler::hosts::schema::CreateHostRequest;
use crate::api::handler::hosts::schema::FullHost;
use crate::api::handler::hosts::schema::GetAllHostsQuery;
use crate::api::handler::hosts::schema::HostRelations;
use crate::api::handler::hosts::schema::PathHost;
use crate::api::handler::hosts::schema::UpdateHostRequest;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::SimpleService;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::AggregationType;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::Domain;
use crate::models::DomainHostRelation;
use crate::models::FindingAffected;
use crate::models::GlobalTag;
use crate::models::Host;
use crate::models::HostGlobalTag;
use crate::models::HostWorkspaceTag;
use crate::models::ManualHost;
use crate::models::Port;
use crate::models::Service;
use crate::models::Workspace;
use crate::models::WorkspaceTag;
use crate::modules::filter::GlobalAST;
use crate::modules::filter::HostAST;
use crate::modules::raw_query::RawQueryBuilder;
use crate::query_tags;

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
                ip_addr: x.ip_addr.ip(),
                comment: x.comment,
                response_time: x.response_time,
                certainty: x.certainty,
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
        ip_addr: host.ip_addr.ip(),
        workspace: *host.workspace.key(),
        os_type: host.os_type,
        comment: host.comment,
        response_time: host.response_time,
        certainty: host.certainty,
        tags,
        sources,
        created_at: host.created_at,
    }))
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
                        .iter()
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
                        .iter()
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

/// Delete the host
///
/// This only deletes the aggregation. The raw results are still in place
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Host was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHost),
    security(("api_key" = []))
)]
#[delete("/workspaces/{w_uuid}/hosts/{h_uuid}")]
pub async fn delete_host(
    path: Path<PathHost>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let PathHost { w_uuid, h_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (Host::F.uuid,))
        .condition(and!(
            Host::F.uuid.equals(h_uuid),
            Host::F.workspace.equals(w_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, Host)
        // We can omit the check if the workspace is the same as we have already checked it in the query before
        .condition(Host::F.uuid.equals(h_uuid))
        .await?;

    tx.commit().await?;

    let msg = WsMessage::DeletedHost {
        workspace: w_uuid,
        host: h_uuid,
    };
    GLOBAL.ws.message_workspace(w_uuid, msg).await;

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

/// Get a host's direct relations
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The host's relations", body = HostRelations),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHost),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/hosts/{h_uuid}/relations")]
pub async fn get_host_relations(path: Path<PathHost>) -> ApiResult<Json<HostRelations>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let ports = query!(&mut tx, Port)
        .condition(Port::F.host.equals(path.h_uuid))
        .stream()
        .map_ok(|p| SimplePort {
            uuid: p.uuid,
            port: p.port as u16,
            protocol: p.protocol,
            certainty: p.certainty,
            host: *p.host.key(),
            comment: p.comment,
            workspace: *p.workspace.key(),
            created_at: p.created_at,
        })
        .try_collect()
        .await?;

    let services = query!(&mut tx, Service)
        .condition(Service::F.host.equals(path.h_uuid))
        .stream()
        .map_ok(|s| SimpleService {
            uuid: s.uuid,
            name: s.name,
            version: s.version,
            certainty: s.certainty,
            host: *s.host.key(),
            port: s.port.map(|x| *x.key()),
            comment: s.comment,
            workspace: *s.workspace.key(),
            created_at: s.created_at,
        })
        .try_collect()
        .await?;

    let mut direct_domains = Vec::new();
    let mut indirect_domains = Vec::new();
    {
        let mut stream = query!(
            &mut tx,
            (
                DomainHostRelation::F.domain as Domain,
                DomainHostRelation::F.is_direct,
            )
        )
        .condition(DomainHostRelation::F.host.equals(path.h_uuid))
        .stream();
        while let Some((d, is_direct)) = stream.try_next().await? {
            (if is_direct {
                &mut direct_domains
            } else {
                &mut indirect_domains
            })
            .push(SimpleDomain {
                uuid: d.uuid,
                domain: d.domain,
                comment: d.comment,
                certainty: d.certainty,
                workspace: *d.workspace.key(),
                created_at: d.created_at,
            });
        }
    }

    tx.commit().await?;

    Ok(Json(HostRelations {
        ports,
        services,
        direct_domains,
        indirect_domains,
    }))
}

/// Get a host's findings
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The host's findings", body = ListFindings),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHost),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/hosts/{h_uuid}/findings")]
pub async fn get_host_findings(
    path: Path<PathHost>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<Json<ListFindings>> {
    let PathHost { w_uuid, h_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let findings = ListFindings::query_through_affected(
        &mut tx,
        w_uuid,
        FindingAffected::F.host.equals(h_uuid),
    )
    .await?;

    tx.commit().await?;
    Ok(Json(findings))
}
