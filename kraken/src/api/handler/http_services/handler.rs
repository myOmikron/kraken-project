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
use crate::api::handler::common::schema::HttpServiceResultsPage;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::common::utils::get_page_params;
use crate::api::handler::common::utils::query_many_severities;
use crate::api::handler::common::utils::query_single_severity;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::findings::schema::ListFindings;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::http_services::schema::CreateHttpServiceRequest;
use crate::api::handler::http_services::schema::FullHttpService;
use crate::api::handler::http_services::schema::GetAllHttpServicesQuery;
use crate::api::handler::http_services::schema::HttpServiceRelations;
use crate::api::handler::http_services::schema::PathHttpService;
use crate::api::handler::http_services::schema::UpdateHttpServiceRequest;
use crate::api::handler::ports::schema::SimplePort;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::AggregationType;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::Domain;
use crate::models::FindingAffected;
use crate::models::GlobalTag;
use crate::models::Host;
use crate::models::HttpService;
use crate::models::HttpServiceGlobalTag;
use crate::models::HttpServiceWorkspaceTag;
use crate::models::ManualHttpService;
use crate::models::Port;
use crate::models::Workspace;
use crate::models::WorkspaceTag;
use crate::modules::filter::GlobalAST;
use crate::modules::filter::HttpServiceAST;
use crate::modules::raw_query::RawQueryBuilder;
use crate::query_tags;

/// List the http services of a workspace
#[utoipa::path(
    tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all http services of a workspace", body = HttpServiceResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = GetAllHttpServicesQuery,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/httpServices/all")]
pub async fn get_all_http_services(
    path: Path<PathUuid>,
    params: Json<GetAllHttpServicesQuery>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<HttpServiceResultsPage>> {
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

    let http_service_filter = params
        .http_service_filter
        .as_deref()
        .map(HttpServiceAST::parse)
        .transpose()?
        .unwrap_or_default();

    // Count host's uuid instead of directly service's to force the implicit join required by the conditions
    let mut count_query = RawQueryBuilder::new((HttpService::F.host.uuid.count(),));
    let mut select_query = RawQueryBuilder::new((
        HttpService::F.uuid,
        HttpService::F.name,
        HttpService::F.version,
        HttpService::F.domain,
        HttpService::F.host.select_as::<Host>(),
        HttpService::F.port.select_as::<Port>(),
        HttpService::F.base_path,
        HttpService::F.tls,
        HttpService::F.sni_required,
        HttpService::F.comment,
        HttpService::F.certainty,
        HttpService::F.created_at,
    ));

    http_service_filter.apply_to_query(&global_filter, &mut count_query);
    http_service_filter.apply_to_query(&global_filter, &mut select_query);

    count_query.append_eq_condition(HttpService::F.workspace, Value::Uuid(path.uuid));
    select_query.append_eq_condition(HttpService::F.workspace, Value::Uuid(path.uuid));

    if let Some(host_uuid) = params.host {
        count_query.append_eq_condition(HttpService::F.host, Value::Uuid(host_uuid));
        select_query.append_eq_condition(HttpService::F.host, Value::Uuid(host_uuid));
    }

    select_query.order_desc(HttpService::F.created_at);
    select_query.limit_offset(limit, offset);

    let (total,) = count_query.one(&mut tx).await?;
    let http_services: Vec<_> = select_query.stream(&mut tx).try_collect().await?;

    let mut domains = HashMap::new();
    let conds = http_services
        .iter()
        .filter_map(|x| x.3.as_ref().map(|y| Domain::F.uuid.equals(*y.key())))
        .collect::<Vec<_>>();
    if !conds.is_empty() {
        let mut domain_stream = query!(&mut tx, Domain)
            .condition(DynamicCollection::or(conds))
            .stream();

        while let Some(domain) = domain_stream.try_next().await? {
            domains.insert(domain.uuid, domain);
        }
    }

    let mut tags = HashMap::new();
    query_tags!(
        tags,
        tx,
        (
            HttpServiceWorkspaceTag::F.workspace_tag as WorkspaceTag,
            HttpServiceWorkspaceTag::F.http_service
        ),
        HttpServiceWorkspaceTag::F.http_service,
        (
            HttpServiceGlobalTag::F.global_tag as GlobalTag,
            HttpServiceGlobalTag::F.http_service
        ),
        HttpServiceGlobalTag::F.http_service,
        http_services.iter().map(|x| x.0)
    );

    let mut sources = SimpleAggregationSource::query(
        &mut tx,
        path.uuid,
        AggregationTable::HttpService,
        http_services.iter().map(|x| x.0),
    )
    .await?;

    let severities = query_many_severities(
        &mut tx,
        FindingAffected::F.http_service,
        http_services.iter().map(|x| x.0),
    )
    .await?;

    tx.commit().await?;

    let items = http_services
        .into_iter()
        .map(
            |(
                uuid,
                name,
                version,
                domain,
                host,
                port,
                base_path,
                tls,
                sni_required,
                comment,
                certainty,
                created_at,
            )| FullHttpService {
                uuid,
                name,
                version,
                domain: domain
                    .and_then(|fm| domains.get(fm.key()))
                    .map(|domain| SimpleDomain {
                        uuid: domain.uuid,
                        domain: domain.domain.clone(),
                        comment: domain.comment.clone(),
                        workspace: path.uuid,
                        created_at: domain.created_at,
                        certainty: FromDb::from_db(domain.certainty),
                    }),
                host: SimpleHost {
                    uuid: host.uuid,
                    ip_addr: host.ip_addr.ip(),
                    os_type: FromDb::from_db(host.os_type),
                    response_time: host.response_time,
                    comment: host.comment,
                    workspace: path.uuid,
                    created_at: host.created_at,
                    certainty: FromDb::from_db(host.certainty),
                },
                port: SimplePort {
                    uuid: port.uuid,
                    port: port.port as u16,
                    protocol: FromDb::from_db(port.protocol),
                    certainty: FromDb::from_db(port.certainty),
                    host: *port.host.key(),
                    comment: port.comment,
                    workspace: path.uuid,
                    created_at: port.created_at,
                },
                base_path,
                tls,
                sni_required,
                comment,
                certainty: FromDb::from_db(certainty),
                workspace: path.uuid,
                created_at,
                tags: tags.remove(&uuid).unwrap_or_default(),
                sources: sources.remove(&uuid).unwrap_or_default(),
                severity: severities.get(&uuid).copied().map(FromDb::from_db),
            },
        )
        .collect();

    Ok(Json(HttpServiceResultsPage {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// Retrieve all information about a single service
#[utoipa::path(
    tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved the selected http service", body = FullHttpService),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHttpService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/httpServices/{hs_uuid}")]
pub async fn get_http_service(
    path: Path<PathHttpService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullHttpService>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges)?;
    }

    let (
        uuid,
        name,
        version,
        domain,
        host,
        port,
        base_path,
        tls,
        sni_required,
        comment,
        certainty,
        created_at,
    ) = query!(
        &mut tx,
        (
            HttpService::F.uuid,
            HttpService::F.name,
            HttpService::F.version,
            HttpService::F.domain,
            HttpService::F.host as Host,
            HttpService::F.port as Port,
            HttpService::F.base_path,
            HttpService::F.tls,
            HttpService::F.sni_required,
            HttpService::F.comment,
            HttpService::F.certainty,
            HttpService::F.created_at,
        )
    )
    .condition(and!(
        HttpService::F.workspace.equals(path.w_uuid),
        HttpService::F.uuid.equals(path.hs_uuid)
    ))
    .optional()
    .await?
    .ok_or(ApiError::InvalidUuid)?;

    let domain = if let Some(domain) = domain.as_ref() {
        Some(
            query!(&mut tx, Domain)
                .condition(Domain::F.uuid.equals(*domain.key()))
                .one()
                .await?,
        )
    } else {
        None
    };

    let global_tags: Vec<_> = query!(&mut tx, (HttpServiceGlobalTag::F.global_tag as GlobalTag,))
        .condition(HttpServiceGlobalTag::F.http_service.equals(path.hs_uuid))
        .stream()
        .map_ok(|(tag,)| SimpleTag::from(tag))
        .try_collect()
        .await?;

    let mut tags: Vec<_> = query!(
        &mut tx,
        (HttpServiceWorkspaceTag::F.workspace_tag as WorkspaceTag,)
    )
    .condition(HttpServiceWorkspaceTag::F.http_service.equals(path.hs_uuid))
    .stream()
    .map_ok(|(tag,)| SimpleTag::from(tag))
    .try_collect()
    .await?;

    tags.extend(global_tags);

    let sources = query!(&mut tx, (AggregationSource::F.source_type,))
        .condition(AggregationSource::F.aggregated_uuid.equals(path.hs_uuid))
        .stream()
        .map_ok(|(x,)| x)
        .try_collect()
        .await?;

    let severity =
        query_single_severity(&mut tx, FindingAffected::F.http_service, path.hs_uuid).await?;

    tx.commit().await?;

    Ok(Json(FullHttpService {
        uuid,
        name,
        version,
        domain: domain.map(|domain| SimpleDomain {
            uuid: domain.uuid,
            domain: domain.domain,
            comment: domain.comment,
            workspace: path.w_uuid,
            created_at: domain.created_at,
            certainty: FromDb::from_db(domain.certainty),
        }),
        host: SimpleHost {
            uuid: host.uuid,
            ip_addr: host.ip_addr.ip(),
            os_type: FromDb::from_db(host.os_type),
            response_time: host.response_time,
            comment: host.comment,
            workspace: path.w_uuid,
            created_at: host.created_at,
            certainty: FromDb::from_db(host.certainty),
        },
        port: SimplePort {
            uuid: port.uuid,
            port: port.port as u16,
            protocol: FromDb::from_db(port.protocol),
            certainty: FromDb::from_db(port.certainty),
            host: *port.host.key(),
            comment: port.comment,
            workspace: path.w_uuid,
            created_at: port.created_at,
        },
        base_path,
        tls,
        sni_required,
        comment,
        certainty: FromDb::from_db(certainty),
        workspace: path.w_uuid,
        created_at,
        tags,
        sources,
        severity: severity.map(FromDb::from_db),
    }))
}

/// Manually add a http service
#[utoipa::path(
tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Http service was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateHttpServiceRequest,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/httpServices")]
pub async fn create_http_service(
    req: Json<CreateHttpServiceRequest>,
    path: Path<PathUuid>,
    SessionUser(user): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let CreateHttpServiceRequest {
        name,
        version,
        domain,
        ip_addr,
        port,
        port_protocol,
        certainty,
        base_path,
        tls,
        sni_require,
    } = req.into_inner();
    let PathUuid { uuid: workspace } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, workspace, user).await? {
        return Err(ApiError::MissingPrivileges)?;
    }
    let uuid = ManualHttpService::insert(
        &mut tx,
        workspace,
        user,
        name,
        version,
        domain,
        ip_addr,
        port,
        port_protocol.into_db(),
        certainty.into_db(),
        base_path,
        tls,
        sni_require,
    )
    .await?;
    tx.commit().await?;

    Ok(Json(UuidResponse { uuid }))
}

/// Update a http service
///
/// You must include at least on parameter
#[utoipa::path(
    tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Http service was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateHttpServiceRequest,
    params(PathHttpService),
    security(("api_key" = []))
)]
#[put("/workspaces/{w_uuid}/httpServices/{hs_uuid}")]
pub async fn update_http_service(
    req: Json<UpdateHttpServiceRequest>,
    path: Path<PathHttpService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();

    if matches!(
        &req,
        UpdateHttpServiceRequest {
            comment: None,
            global_tags: None,
            workspace_tags: None
        }
    ) {
        return Err(ApiError::EmptyJson);
    }

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (HttpService::F.uuid,))
        .condition(HttpService::F.uuid.equals(path.hs_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(global_tags) = &req.global_tags {
        GlobalTag::exist_all(&mut tx, global_tags.iter().copied())
            .await?
            .ok_or(ApiError::InvalidUuid)?;

        rorm::delete!(&mut tx, HttpServiceGlobalTag)
            .condition(HttpServiceGlobalTag::F.http_service.equals(path.hs_uuid))
            .await?;

        if !global_tags.is_empty() {
            insert!(&mut tx, HttpServiceGlobalTag)
                .return_nothing()
                .bulk(
                    &global_tags
                        .iter()
                        .map(|x| HttpServiceGlobalTag {
                            uuid: Uuid::new_v4(),
                            http_service: ForeignModelByField::Key(path.hs_uuid),
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

        rorm::delete!(&mut tx, HttpServiceWorkspaceTag)
            .condition(HttpServiceWorkspaceTag::F.http_service.equals(path.hs_uuid))
            .await?;

        if !workspace_tags.is_empty() {
            insert!(&mut tx, HttpServiceWorkspaceTag)
                .return_nothing()
                .bulk(
                    &workspace_tags
                        .iter()
                        .map(|x| HttpServiceWorkspaceTag {
                            uuid: Uuid::new_v4(),
                            http_service: ForeignModelByField::Key(path.hs_uuid),
                            workspace_tag: ForeignModelByField::Key(*x),
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;
        }
    }

    if let Some(comment) = req.comment {
        update!(&mut tx, HttpService)
            .condition(HttpService::F.uuid.equals(path.hs_uuid))
            .set(HttpService::F.comment, comment)
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
                    uuid: path.hs_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::HttpService,
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
                    uuid: path.hs_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::HttpService,
                    tags,
                },
            )
            .await;
    }

    Ok(HttpResponse::Ok().finish())
}

/// Delete the http service
///
/// This only deletes the aggregation. The raw results are still in place
#[utoipa::path(
    tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Http service was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHttpService),
    security(("api_key" = []))
)]
#[delete("/workspaces/{w_uuid}/httpServices/{hs_uuid}")]
pub async fn delete_http_service(
    path: Path<PathHttpService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let PathHttpService { w_uuid, hs_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (HttpService::F.uuid,))
        .condition(and!(
            HttpService::F.uuid.equals(hs_uuid),
            HttpService::F.workspace.equals(w_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, HttpService)
        // We can omit the check if the workspace is the same as we have already checked it in the query before
        .condition(HttpService::F.uuid.equals(hs_uuid))
        .await?;

    tx.commit().await?;

    GLOBAL
        .ws
        .message_workspace(
            w_uuid,
            WsMessage::DeletedHttpService {
                workspace: w_uuid,
                http_service: hs_uuid,
            },
        )
        .await;

    Ok(HttpResponse::Ok().finish())
}

/// Get all data sources which referenced this http service
#[utoipa::path(
    tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The http service's sources", body = FullAggregationSource),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHttpService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/httpServices/{hs_uuid}/sources")]
pub async fn get_http_service_sources(
    path: Path<PathHttpService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullAggregationSource>> {
    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }
    let source = FullAggregationSource::query(
        &mut tx,
        path.w_uuid,
        AggregationTable::HttpService,
        path.hs_uuid,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(source))
}

/// Get a http service's direct relations
#[utoipa::path(
    tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The http service's relations", body = HttpServiceRelations),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHttpService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/httpServices/{hs_uuid}/relations")]
pub async fn get_http_service_relations(
    path: Path<PathHttpService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<HttpServiceRelations>> {
    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (host, port, domain) = query!(
        &mut tx,
        (
            HttpService::F.host as Host,
            HttpService::F.port as Port,
            HttpService::F.domain
        )
    )
    .optional()
    .await?
    .ok_or(ApiError::InvalidUuid)?;

    let domain = if let Some(domain) = domain {
        Some(
            query!(&mut tx, Domain)
                .condition(Domain::F.uuid.equals(*domain.key()))
                .one()
                .await?,
        )
    } else {
        None
    };

    Ok(Json(HttpServiceRelations {
        host: SimpleHost {
            uuid: host.uuid,
            ip_addr: host.ip_addr.ip(),
            os_type: FromDb::from_db(host.os_type),
            response_time: host.response_time,
            comment: host.comment,
            workspace: path.w_uuid,
            created_at: host.created_at,
            certainty: FromDb::from_db(host.certainty),
        },
        port: SimplePort {
            uuid: port.uuid,
            port: port.port as u16,
            protocol: FromDb::from_db(port.protocol),
            certainty: FromDb::from_db(port.certainty),
            host: *port.host.key(),
            comment: port.comment,
            workspace: path.w_uuid,
            created_at: port.created_at,
        },
        domain: domain.map(|domain| SimpleDomain {
            uuid: domain.uuid,
            domain: domain.domain,
            comment: domain.comment,
            workspace: path.w_uuid,
            created_at: domain.created_at,
            certainty: FromDb::from_db(domain.certainty),
        }),
    }))
}

/// Get a http service's findings
#[utoipa::path(
    tag = "Http Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The http service's findings", body = ListFindings),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHttpService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/httpServices/{hs_uuid}/findings")]
pub async fn get_http_service_findings(
    path: Path<PathHttpService>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<Json<ListFindings>> {
    let PathHttpService { w_uuid, hs_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let findings = ListFindings::query_through_affected(
        &mut tx,
        w_uuid,
        FindingAffected::F.http_service,
        hs_uuid,
    )
    .await?;

    tx.commit().await?;
    Ok(Json(findings))
}
