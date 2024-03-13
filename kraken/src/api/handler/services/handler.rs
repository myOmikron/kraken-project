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
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::ServiceResultsPage;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::common::schema::TagType;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::common::utils::get_page_params;
use crate::api::handler::findings::schema::ListFindings;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::CreateServiceRequest;
use crate::api::handler::services::schema::FullService;
use crate::api::handler::services::schema::GetAllServicesQuery;
use crate::api::handler::services::schema::PathService;
use crate::api::handler::services::schema::ServiceRelations;
use crate::api::handler::services::schema::UpdateServiceRequest;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::AggregationType;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::FindingAffected;
use crate::models::GlobalTag;
use crate::models::Host;
use crate::models::ManualService;
use crate::models::Port;
use crate::models::Service;
use crate::models::ServiceGlobalTag;
use crate::models::ServiceWorkspaceTag;
use crate::models::Workspace;
use crate::models::WorkspaceTag;
use crate::modules::filter::GlobalAST;
use crate::modules::filter::ServiceAST;
use crate::modules::raw_query::RawQueryBuilder;
use crate::query_tags;

/// List the services of a workspace
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all services of a workspace", body = ServiceResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = GetAllServicesQuery,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/services/all")]
pub async fn get_all_services(
    path: Path<PathUuid>,
    params: Json<GetAllServicesQuery>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<ServiceResultsPage>> {
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

    let service_filter = params
        .service_filter
        .as_deref()
        .map(ServiceAST::parse)
        .transpose()?
        .unwrap_or_default();

    // Count host's uuid instead of directly service's to force the implicit join required by the conditions
    let mut count_query = RawQueryBuilder::new((Service::F.host.uuid.count(),));
    let mut select_query = RawQueryBuilder::new((
        Service::F.uuid,
        Service::F.name,
        Service::F.version,
        Service::F.certainty,
        Service::F.comment,
        Service::F.created_at,
        Service::F.host.select_as::<Host>(),
        Service::F.port,
        Service::F.protocols,
        Service::F.workspace,
    ));

    service_filter.apply_to_query(&global_filter, &mut count_query);
    service_filter.apply_to_query(&global_filter, &mut select_query);

    count_query.append_eq_condition(Service::F.workspace, Value::Uuid(path.uuid));
    select_query.append_eq_condition(Service::F.workspace, Value::Uuid(path.uuid));

    if let Some(host_uuid) = params.host {
        count_query.append_eq_condition(Service::F.host, Value::Uuid(host_uuid));
        select_query.append_eq_condition(Service::F.host, Value::Uuid(host_uuid));
    }

    select_query.order_desc(Service::F.created_at);
    select_query.limit_offset(limit, offset);

    let (total,) = count_query.one(&mut tx).await?;
    let services: Vec<_> = select_query.stream(&mut tx).try_collect().await?;

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
                    port: port.port as u16,
                    protocol: port.protocol,
                    certainty: port.certainty,
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

    let mut sources = SimpleAggregationSource::query(
        &mut tx,
        path.uuid,
        AggregationTable::Service,
        services.iter().map(|x| x.0),
    )
    .await?;

    tx.commit().await?;

    let items = services
        .into_iter()
        .map(
            |(
                uuid,
                name,
                version,
                certainty,
                comment,
                created_at,
                host,
                port,
                protocols,
                workspace,
            )| {
                FullService {
                    uuid,
                    name,
                    version,
                    certainty,
                    comment,
                    host: SimpleHost {
                        uuid: host.uuid,
                        ip_addr: host.ip_addr.ip(),
                        os_type: host.os_type,
                        response_time: host.response_time,
                        certainty: host.certainty,
                        comment: host.comment,
                        workspace: *host.workspace.key(),
                        created_at: host.created_at,
                    },
                    port: port.as_ref().map(|y| {
                        // There is an entry with the key y.key(), as y.key() was used to construct
                        // the values in the HashMap
                        #[allow(clippy::unwrap_used)]
                        ports.get(y.key()).unwrap().clone()
                    }),
                    protocols: port
                        .and_then(|y| ports.get(y.key()))
                        .map(|port| port.protocol.decode_service(protocols)),
                    workspace: *workspace.key(),
                    tags: tags.remove(&uuid).unwrap_or_default(),
                    sources: sources.remove(&uuid).unwrap_or_default(),
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

/// Retrieve all information about a single service
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved the selected service", body = FullService),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/services/{s_uuid}")]
pub async fn get_service(
    path: Path<PathService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullService>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges)?;
    }

    let service = query!(&mut tx, Service)
        .condition(and!(
            Service::F.workspace.equals(path.w_uuid),
            Service::F.uuid.equals(path.s_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let host = query!(&mut tx, Host)
        .condition(Host::F.uuid.equals(*service.host.key()))
        .one()
        .await?;

    let port = if let Some(port) = service.port.as_ref() {
        Some(
            query!(&mut tx, Port)
                .condition(and!(
                    Port::F.workspace.equals(path.w_uuid),
                    Port::F.uuid.equals(*port.key())
                ))
                .one()
                .await?,
        )
    } else {
        None
    };

    let mut tags: Vec<_> = query!(&mut tx, (ServiceGlobalTag::F.global_tag as GlobalTag,))
        .condition(ServiceGlobalTag::F.service.equals(path.s_uuid))
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
        (ServiceWorkspaceTag::F.workspace_tag as WorkspaceTag,)
    )
    .condition(ServiceWorkspaceTag::F.service.equals(path.s_uuid))
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

    Ok(Json(FullService {
        uuid: path.s_uuid,
        name: service.name,
        version: service.version,
        certainty: service.certainty,
        host: SimpleHost {
            uuid: host.uuid,
            ip_addr: host.ip_addr.ip(),
            os_type: host.os_type,
            response_time: host.response_time,
            certainty: host.certainty,
            comment: host.comment,
            workspace: path.w_uuid,
            created_at: host.created_at,
        },
        protocols: port
            .as_ref()
            .map(|port| port.protocol.decode_service(service.protocols)),
        port: port.map(|port| SimplePort {
            uuid: port.uuid,
            port: port.port as u16,
            protocol: port.protocol,
            certainty: port.certainty,
            host: host.uuid,
            comment: port.comment,
            workspace: path.w_uuid,
            created_at: port.created_at,
        }),
        comment: service.comment,
        workspace: path.w_uuid,
        tags,
        sources,
        created_at: service.created_at,
    }))
}

/// Manually add a service
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Service was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateServiceRequest,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/services")]
pub async fn create_service(
    req: Json<CreateServiceRequest>,
    path: Path<PathUuid>,
    SessionUser(user): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let CreateServiceRequest {
        name,
        certainty,
        host,
        port,
        protocols,
    } = req.into_inner();
    let PathUuid { uuid: workspace } = path.into_inner();

    if port.is_some() && protocols.is_none() || port.is_none() && protocols.is_some() {
        return Err(ApiError::InvalidPort);
    }

    Ok(Json(UuidResponse {
        uuid: ManualService::insert(
            &GLOBAL.db,
            workspace,
            user,
            name,
            host,
            port.zip(protocols),
            certainty,
        )
        .await?,
    }))
}

/// Update a service
///
/// You must include at least on parameter
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Service was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateServiceRequest,
    params(PathService),
    security(("api_key" = []))
)]
#[put("/workspaces/{w_uuid}/services/{s_uuid}")]
pub async fn update_service(
    req: Json<UpdateServiceRequest>,
    path: Path<PathService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();

    if req.workspace_tags.is_none() && req.global_tags.is_none() && req.comment.is_none() {
        return Err(ApiError::EmptyJson);
    }

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (Service::F.uuid,))
        .condition(Service::F.uuid.equals(path.s_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(global_tags) = &req.global_tags {
        GlobalTag::exist_all(&mut tx, global_tags.iter().copied())
            .await?
            .ok_or(ApiError::InvalidUuid)?;

        rorm::delete!(&mut tx, ServiceGlobalTag)
            .condition(ServiceGlobalTag::F.service.equals(path.s_uuid))
            .await?;

        if !global_tags.is_empty() {
            insert!(&mut tx, ServiceGlobalTag)
                .return_nothing()
                .bulk(
                    &global_tags
                        .iter()
                        .map(|x| ServiceGlobalTag {
                            uuid: Uuid::new_v4(),
                            service: ForeignModelByField::Key(path.s_uuid),
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

        rorm::delete!(&mut tx, ServiceWorkspaceTag)
            .condition(ServiceWorkspaceTag::F.service.equals(path.s_uuid))
            .await?;

        if !workspace_tags.is_empty() {
            insert!(&mut tx, ServiceWorkspaceTag)
                .return_nothing()
                .bulk(
                    &workspace_tags
                        .iter()
                        .map(|x| ServiceWorkspaceTag {
                            uuid: Uuid::new_v4(),
                            service: ForeignModelByField::Key(path.s_uuid),
                            workspace_tag: ForeignModelByField::Key(*x),
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;
        }
    }

    if let Some(comment) = req.comment {
        update!(&mut tx, Service)
            .condition(Service::F.uuid.equals(path.s_uuid))
            .set(Service::F.comment, comment)
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
                    uuid: path.s_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::Service,
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
                    uuid: path.s_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::Service,
                    tags,
                },
            )
            .await;
    }

    Ok(HttpResponse::Ok().finish())
}

/// Delete the service
///
/// This only deletes the aggregation. The raw results are still in place
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Service was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathService),
    security(("api_key" = []))
)]
#[delete("/workspaces/{w_uuid}/services/{s_uuid}")]
pub async fn delete_service(
    path: Path<PathService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let PathService { w_uuid, s_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (Service::F.uuid,))
        .condition(and!(
            Service::F.uuid.equals(s_uuid),
            Service::F.workspace.equals(w_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, Service)
        // We can omit the check if the workspace is the same as we have already checked it in the query before
        .condition(Service::F.uuid.equals(s_uuid))
        .await?;

    tx.commit().await?;

    let msg = WsMessage::DeletedService {
        workspace: w_uuid,
        service: s_uuid,
    };
    GLOBAL.ws.message_workspace(w_uuid, msg).await;

    Ok(HttpResponse::Ok().finish())
}

/// Get all data sources which referenced this service
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The service's sources", body = FullAggregationSource),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/services/{s_uuid}/sources")]
pub async fn get_service_sources(
    path: Path<PathService>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullAggregationSource>> {
    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }
    let source =
        FullAggregationSource::query(&mut tx, path.w_uuid, AggregationTable::Service, path.s_uuid)
            .await?;
    tx.commit().await?;
    Ok(Json(source))
}

/// Get a service's direct relations
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The service's relations", body = ServiceRelations),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/services/{s_uuid}/relations")]
pub async fn get_service_relations(path: Path<PathService>) -> ApiResult<Json<ServiceRelations>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let (host, port) = query!(&mut tx, (Service::F.host as Host, Service::F.port,))
        .condition(Service::F.uuid.equals(path.s_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let port = if let Some(port) = port {
        let p = query!(&mut tx, Port)
            .condition(Port::F.uuid.equals(*port.key()))
            .one()
            .await?;
        Some(SimplePort {
            uuid: p.uuid,
            port: p.port as u16,
            protocol: p.protocol,
            certainty: p.certainty,
            host: *p.host.key(),
            comment: p.comment,
            workspace: *p.workspace.key(),
            created_at: p.created_at,
        })
    } else {
        None
    };

    tx.commit().await?;

    Ok(Json(ServiceRelations {
        host: SimpleHost {
            uuid: host.uuid,
            ip_addr: host.ip_addr.ip(),
            os_type: host.os_type,
            response_time: host.response_time,
            certainty: host.certainty,
            comment: host.comment,
            workspace: *host.workspace.key(),
            created_at: host.created_at,
        },
        port,
    }))
}

/// Get a service's findings
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The service's findings", body = ListFindings),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathService),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/services/{s_uuid}/findings")]
pub async fn get_service_findings(
    path: Path<PathService>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<Json<ListFindings>> {
    let PathService { w_uuid, s_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let findings = ListFindings::query_through_affected(
        &mut tx,
        w_uuid,
        FindingAffected::F.service.equals(s_uuid),
    )
    .await?;

    tx.commit().await?;
    Ok(Json(findings))
}
