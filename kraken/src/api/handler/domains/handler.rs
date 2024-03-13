use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

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
use rorm::field;
use rorm::insert;
use rorm::internal::field::Field;
use rorm::model::PatchSelector;
use rorm::or;
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
use crate::api::handler::common::schema::DomainResultsPage;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::common::schema::TagType;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::common::utils::get_page_params;
use crate::api::handler::domains::schema::CreateDomainRequest;
use crate::api::handler::domains::schema::DomainRelations;
use crate::api::handler::domains::schema::FullDomain;
use crate::api::handler::domains::schema::GetAllDomainsQuery;
use crate::api::handler::domains::schema::PathDomain;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::domains::schema::UpdateDomainRequest;
use crate::api::handler::findings::schema::ListFindings;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::AggregationType;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::Domain;
use crate::models::DomainDomainRelation;
use crate::models::DomainGlobalTag;
use crate::models::DomainHostRelation;
use crate::models::DomainWorkspaceTag;
use crate::models::FindingAffected;
use crate::models::GlobalTag;
use crate::models::Host;
use crate::models::ManualDomain;
use crate::models::Workspace;
use crate::models::WorkspaceTag;
use crate::modules::filter::DomainAST;
use crate::modules::filter::GlobalAST;
use crate::modules::raw_query::RawQueryBuilder;
use crate::query_tags;

/// Retrieve all domains of a specific workspace
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all domains of a workspace", body = DomainResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = GetAllDomainsQuery,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/domains/all")]
pub async fn get_all_domains(
    path: Path<PathUuid>,
    params: Json<GetAllDomainsQuery>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<DomainResultsPage>> {
    let path = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    let mut tags: HashMap<Uuid, Vec<SimpleTag>> = HashMap::new();

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

    let domain_filter = params
        .domain_filter
        .as_deref()
        .map(DomainAST::parse)
        .transpose()?
        .unwrap_or_default();

    let mut count_query = RawQueryBuilder::new((Domain::F.uuid.count(),));
    let mut select_query = RawQueryBuilder::new(PatchSelector::<Domain>::new());

    domain_filter.apply_to_query(&global_filter, &mut count_query);
    domain_filter.apply_to_query(&global_filter, &mut select_query);

    count_query.append_eq_condition(Domain::F.workspace, Value::Uuid(path.uuid));
    select_query.append_eq_condition(Domain::F.workspace, Value::Uuid(path.uuid));

    if let Some(host_uuid) = params.host {
        fn append(sql: &mut String, values: &mut Vec<Value>, uuid: Uuid) -> fmt::Result {
            const DOMAIN_TABLE: &str = Domain::TABLE;
            const DOMAIN_UUID: &str = <field!(DomainHostRelation::F.uuid)>::NAME;
            const M2M_TABLE: &str = DomainHostRelation::TABLE;
            const M2M_DOMAIN: &str = <field!(DomainHostRelation::F.domain)>::NAME;
            const M2M_HOST: &str = <field!(DomainHostRelation::F.host)>::NAME;
            values.push(Value::Uuid(uuid));
            write!(
                sql,
                r#""{DOMAIN_TABLE}"."{DOMAIN_UUID}" IN (SELECT "{M2M_TABLE}"."{M2M_DOMAIN}" FROM "{M2M_TABLE}" WHERE "{M2M_TABLE}"."{M2M_HOST}" = ${})"#,
                values.len()
            )
        }
        count_query.append_condition(|sql, values| append(sql, values, host_uuid));
        select_query.append_condition(|sql, values| append(sql, values, host_uuid));
    }

    select_query.order_desc(Domain::F.created_at);
    select_query.limit_offset(limit, offset);

    let (total,) = count_query.one(&mut tx).await?;
    let domains: Vec<_> = select_query.stream(&mut tx).try_collect().await?;

    query_tags!(
        tags,
        tx,
        (
            DomainWorkspaceTag::F.workspace_tag as WorkspaceTag,
            DomainWorkspaceTag::F.domain
        ),
        DomainWorkspaceTag::F.domain,
        (
            DomainGlobalTag::F.global_tag as GlobalTag,
            DomainGlobalTag::F.domain
        ),
        DomainGlobalTag::F.domain,
        domains.iter().map(|x| x.uuid)
    );

    let mut sources = SimpleAggregationSource::query(
        &mut tx,
        path.uuid,
        AggregationTable::Domain,
        domains.iter().map(|x| x.uuid),
    )
    .await?;

    let items = domains
        .into_iter()
        .map(|x| FullDomain {
            uuid: x.uuid,
            domain: x.domain,
            certainty: x.certainty,
            comment: x.comment,
            workspace: *x.workspace.key(),
            tags: tags.remove(&x.uuid).unwrap_or_default(),
            sources: sources.remove(&x.uuid).unwrap_or_default(),
            created_at: x.created_at,
        })
        .collect();

    tx.commit().await?;
    Ok(Json(DomainResultsPage {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// Retrieve all information about a single domain
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved the selected domain", body = FullDomain),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathDomain),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/domains/{d_uuid}")]
pub async fn get_domain(
    path: Path<PathDomain>,

    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullDomain>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges)?;
    }

    let domain = query!(&mut tx, Domain)
        .condition(and!(
            Domain::F.workspace.equals(path.w_uuid),
            Domain::F.uuid.equals(path.d_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let mut tags: Vec<_> = query!(&mut tx, (DomainGlobalTag::F.global_tag as GlobalTag,))
        .condition(DomainGlobalTag::F.domain.equals(path.d_uuid))
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
        (DomainWorkspaceTag::F.workspace_tag as WorkspaceTag,)
    )
    .condition(DomainWorkspaceTag::F.domain.equals(path.d_uuid))
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
        .condition(AggregationSource::F.aggregated_uuid.equals(domain.uuid))
        .stream()
        .map_ok(|(x,)| x)
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(FullDomain {
        uuid: path.d_uuid,
        domain: domain.domain,
        comment: domain.comment,
        certainty: domain.certainty,
        workspace: path.w_uuid,
        tags,
        sources,
        created_at: domain.created_at,
    }))
}

/// Manually add a domain
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Domain was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateDomainRequest,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/domains")]
pub async fn create_domain(
    req: Json<CreateDomainRequest>,
    path: Path<PathUuid>,

    SessionUser(user): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let CreateDomainRequest { domain } = req.into_inner();
    let PathUuid { uuid: workspace } = path.into_inner();
    Ok(Json(UuidResponse {
        uuid: ManualDomain::insert(&GLOBAL.db, workspace, user, domain).await?,
    }))
}

/// Update a domain
///
/// You must include at least on parameter
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Domain was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateDomainRequest,
    params(PathDomain),
    security(("api_key" = []))
)]
#[put("/workspaces/{w_uuid}/domains/{d_uuid}")]
pub async fn update_domain(
    req: Json<UpdateDomainRequest>,
    path: Path<PathDomain>,

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

    query!(&mut tx, (Domain::F.uuid,))
        .condition(Domain::F.uuid.equals(path.d_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(global_tags) = &req.global_tags {
        GlobalTag::exist_all(&mut tx, global_tags.iter().copied())
            .await?
            .ok_or(ApiError::InvalidUuid)?;

        rorm::delete!(&mut tx, DomainGlobalTag)
            .condition(DomainGlobalTag::F.domain.equals(path.d_uuid))
            .await?;

        if !global_tags.is_empty() {
            insert!(&mut tx, DomainGlobalTag)
                .return_nothing()
                .bulk(
                    &global_tags
                        .iter()
                        .map(|x| DomainGlobalTag {
                            uuid: Uuid::new_v4(),
                            domain: ForeignModelByField::Key(path.d_uuid),
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

        rorm::delete!(&mut tx, DomainWorkspaceTag)
            .condition(DomainWorkspaceTag::F.domain.equals(path.d_uuid))
            .await?;

        if !workspace_tags.is_empty() {
            insert!(&mut tx, DomainWorkspaceTag)
                .return_nothing()
                .bulk(
                    &workspace_tags
                        .iter()
                        .map(|x| DomainWorkspaceTag {
                            uuid: Uuid::new_v4(),
                            domain: ForeignModelByField::Key(path.d_uuid),
                            workspace_tag: ForeignModelByField::Key(*x),
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;
        }
    }

    if let Some(comment) = req.comment {
        update!(&mut tx, Domain)
            .condition(Domain::F.uuid.equals(path.d_uuid))
            .set(Domain::F.comment, comment)
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
                    uuid: path.d_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::Domain,
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
                    uuid: path.d_uuid,
                    workspace: path.w_uuid,
                    aggregation: AggregationType::Domain,
                    tags,
                },
            )
            .await;
    }

    Ok(HttpResponse::Ok().finish())
}

/// Delete the domain
///
/// This only deletes the aggregation. The raw results are still in place
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Domain was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathDomain),
    security(("api_key" = []))
)]
#[delete("/workspaces/{w_uuid}/domains/{d_uuid}")]
pub async fn delete_domain(
    path: Path<PathDomain>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let PathDomain { w_uuid, d_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (Domain::F.uuid,))
        .condition(and!(
            Domain::F.uuid.equals(d_uuid),
            Domain::F.workspace.equals(w_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, Domain)
        // We can omit the check if the workspace is the same as we have already checked it in the query before
        .condition(Domain::F.uuid.equals(d_uuid))
        .await?;

    tx.commit().await?;

    let msg = WsMessage::DeletedDomain {
        workspace: w_uuid,
        domain: d_uuid,
    };
    GLOBAL.ws.message_workspace(w_uuid, msg).await;

    Ok(HttpResponse::Ok().finish())
}

/// Get all data sources which referenced this domain
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The domain's sources", body = FullAggregationSource),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathDomain),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/domains/{d_uuid}/sources")]
pub async fn get_domain_sources(
    path: Path<PathDomain>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullAggregationSource>> {
    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }
    let source =
        FullAggregationSource::query(&mut tx, path.w_uuid, AggregationTable::Domain, path.d_uuid)
            .await?;
    tx.commit().await?;
    Ok(Json(source))
}

/// Get a domain's direct relations
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The domain's relations", body = DomainRelations),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathDomain),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/domains/{d_uuid}/relations")]
pub async fn get_domain_relations(path: Path<PathDomain>) -> ApiResult<Json<DomainRelations>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let mut source_domains = Vec::new();
    let mut target_domains = Vec::new();
    {
        let mut stream = query!(
            &mut tx,
            (
                DomainDomainRelation::F.source as Domain,
                DomainDomainRelation::F.destination as Domain,
            )
        )
        .condition(or![
            DomainDomainRelation::F.source.equals(path.d_uuid),
            DomainDomainRelation::F.destination.equals(path.d_uuid)
        ])
        .stream();
        while let Some((source, target)) = stream.try_next().await? {
            let vec;
            let d;
            if source.uuid == path.d_uuid {
                vec = &mut target_domains;
                d = target;
            } else {
                vec = &mut source_domains;
                d = source;
            }
            vec.push(SimpleDomain {
                uuid: d.uuid,
                domain: d.domain,
                certainty: d.certainty,
                comment: d.comment,
                workspace: *d.workspace.key(),
                created_at: d.created_at,
            })
        }
    }

    let mut direct_hosts = Vec::new();
    let mut indirect_hosts = Vec::new();
    {
        let mut stream = query!(
            &mut tx,
            (
                DomainHostRelation::F.host as Host,
                DomainHostRelation::F.is_direct,
            )
        )
        .condition(DomainHostRelation::F.domain.equals(path.d_uuid))
        .stream();
        while let Some((h, is_direct)) = stream.try_next().await? {
            (if is_direct {
                &mut direct_hosts
            } else {
                &mut indirect_hosts
            })
            .push(SimpleHost {
                uuid: h.uuid,
                ip_addr: h.ip_addr.ip(),
                os_type: h.os_type,
                comment: h.comment,
                response_time: h.response_time,
                certainty: h.certainty,
                workspace: *h.workspace.key(),
                created_at: h.created_at,
            });
        }
    }

    tx.commit().await?;

    Ok(Json(DomainRelations {
        source_domains,
        target_domains,
        direct_hosts,
        indirect_hosts,
    }))
}

/// Get a domain's findings
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The domain's findings", body = ListFindings),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathDomain),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/domains/{d_uuid}/findings")]
pub async fn get_domain_findings(
    path: Path<PathDomain>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<Json<ListFindings>> {
    let PathDomain { w_uuid, d_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let findings = ListFindings::query_through_affected(
        &mut tx,
        w_uuid,
        FindingAffected::F.domain.equals(d_uuid),
    )
    .await?;

    tx.commit().await?;
    Ok(Json(findings))
}
