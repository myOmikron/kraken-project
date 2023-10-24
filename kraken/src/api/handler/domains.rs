//! The handlers for the aggregated data of domains are located here

use std::collections::HashMap;

use actix_toolbox::tb_middleware::Session;
use actix_web::get;
use actix_web::web::{Data, Json, Path, Query};
use futures::TryStreamExt;
use rorm::conditions::DynamicCollection;
use rorm::{and, query, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::{
    get_page_params, ApiError, ApiResult, DomainResultsPage, PageParams, PathUuid, SimpleTag,
    TagType,
};
use crate::models::{
    Domain, DomainGlobalTag, DomainHostRelation, DomainWorkspaceTag, GlobalTag, Host, Workspace,
    WorkspaceTag,
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
    uuid: Uuid,
    #[schema(example = "example.com")]
    domain: String,
    #[schema(example = "This is a important domain!")]
    comment: String,
    workspace: Uuid,
    tags: Vec<SimpleTag>,
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
                domains
            );

            let items = domains
                .into_iter()
                .map(|x| SimpleDomain {
                    uuid: x.uuid,
                    domain: x.domain,
                    comment: x.comment,
                    workspace: *x.workspace.key(),
                    tags: tags.remove(&x.uuid).unwrap_or_default(),
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
                domains
            );

            let items = domains
                .into_iter()
                .map(|x| SimpleDomain {
                    uuid: x.uuid,
                    domain: x.domain,
                    comment: x.comment,
                    workspace: *x.workspace.key(),
                    tags: tags.remove(&x.uuid).unwrap_or_default(),
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
