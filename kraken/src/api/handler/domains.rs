//! The handlers for the aggregated data of domains are located here

use std::collections::HashMap;

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{get, put, HttpResponse};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use rorm::conditions::DynamicCollection;
use rorm::prelude::ForeignModelByField;
use rorm::{and, insert, query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::{
    get_page_params, ApiError, ApiResult, DomainResultsPage, PageParams, PathUuid,
    SimpleAggregationSource, SimpleTag, TagType,
};
use crate::models::{
    AggregationSource, AggregationTable, Domain, DomainGlobalTag, DomainHostRelation,
    DomainWorkspaceTag, GlobalTag, Host, Workspace, WorkspaceTag,
};
use crate::query_tags;

/// Query parameters for filtering the domains to get
#[derive(Deserialize, IntoParams)]
pub struct GetAllDomainsQuery {
    /// Only get domains pointing to a specific host
    ///
    /// This includes domains which point to another domain which points to this host.
    pub host: Option<Uuid>,
}

/// A simple representation of a domain in a workspace
#[derive(Serialize, ToSchema)]
pub struct SimpleDomain {
    pub(crate) uuid: Uuid,
    #[schema(example = "example.com")]
    pub(crate) domain: String,
    #[schema(example = "This is a important domain!")]
    pub(crate) comment: String,
    pub(crate) workspace: Uuid,
    pub(crate) created_at: DateTime<Utc>,
}

/// A full representation of a domain in a workspace
#[derive(Serialize, ToSchema)]
pub struct FullDomain {
    /// The primary key of the domain
    pub uuid: Uuid,
    /// The domain's name
    #[schema(example = "example.com")]
    pub domain: String,
    /// A comment
    #[schema(example = "This is a important domain!")]
    pub comment: String,
    /// The workspace this domain is in
    pub workspace: Uuid,
    /// The list of tags this domain has attached to
    pub tags: Vec<SimpleTag>,
    /// The number of attacks which found this domain
    pub sources: SimpleAggregationSource,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// Retrieve all domains of a specific workspace
#[utoipa::path(
    tag = "Domains",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all domains of a workspace", body = DomainResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams, GetAllDomainsQuery),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/domains")]
pub async fn get_all_domains(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    filter_params: Query<GetAllDomainsQuery>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<Json<DomainResultsPage>> {
    let path = path.into_inner();
    let user_uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let mut tags: HashMap<Uuid, Vec<SimpleTag>> = HashMap::new();

    if !Workspace::is_user_member_or_owner(&mut tx, path.uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (limit, offset) = get_page_params(page_params).await?;

    match filter_params.into_inner().host {
        None => {
            let (total,) = query!(&mut tx, (Domain::F.uuid.count()))
                .condition(Domain::F.workspace.equals(path.uuid))
                .one()
                .await?;

            let domains = query!(&mut tx, Domain)
                .condition(Domain::F.workspace.equals(path.uuid))
                .order_desc(Domain::F.created_at)
                .limit(limit)
                .offset(offset)
                .all()
                .await?;

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
        Some(host_uuid) => {
            query!(&mut tx, (Host::F.uuid,))
                .condition(and![
                    Host::F.workspace.equals(path.uuid),
                    Host::F.uuid.equals(host_uuid)
                ])
                .optional()
                .await?
                .ok_or(ApiError::InvalidUuid)?;

            let (total,) = query!(&mut tx, (DomainHostRelation::F.uuid.count()))
                .condition(DomainHostRelation::F.host.equals(host_uuid))
                .one()
                .await?;

            let domains: Vec<Domain> = query!(&mut tx, (DomainHostRelation::F.domain as Domain,))
                .condition(DomainHostRelation::F.host.equals(host_uuid))
                .order_desc(DomainHostRelation::F.domain.created_at)
                .limit(limit)
                .offset(offset)
                .stream()
                .map_ok(|x| x.0)
                .try_collect()
                .await?;

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
    }
}

/// The path parameter of a domain
#[derive(Deserialize, IntoParams)]
pub struct PathDomain {
    /// The workspace's uuid
    pub w_uuid: Uuid,
    /// The domain's uuid
    pub d_uuid: Uuid,
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
    db: Data<Database>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullDomain>> {
    let mut tx = db.start_transaction().await?;

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
        workspace: path.w_uuid,
        tags,
        sources,
        created_at: domain.created_at,
    }))
}

/// The request to update a domain
#[derive(Deserialize, ToSchema)]
pub struct UpdateDomainRequest {
    comment: Option<String>,
    global_tags: Option<Vec<Uuid>>,
    workspace_tags: Option<Vec<Uuid>>,
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
    db: Data<Database>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();

    if req.workspace_tags.is_none() && req.global_tags.is_none() && req.comment.is_none() {
        return Err(ApiError::EmptyJson);
    }

    let mut tx = db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (Domain::F.uuid,))
        .condition(Domain::F.uuid.equals(path.d_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(global_tags) = req.global_tags {
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
                        .into_iter()
                        .map(|x| DomainGlobalTag {
                            uuid: Uuid::new_v4(),
                            domain: ForeignModelByField::Key(path.d_uuid),
                            global_tag: ForeignModelByField::Key(x),
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;
        }
    }

    if let Some(workspace_tags) = req.workspace_tags {
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
                        .into_iter()
                        .map(|x| DomainWorkspaceTag {
                            uuid: Uuid::new_v4(),
                            domain: ForeignModelByField::Key(path.d_uuid),
                            workspace_tag: ForeignModelByField::Key(x),
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

    Ok(HttpResponse::Ok().finish())
}
