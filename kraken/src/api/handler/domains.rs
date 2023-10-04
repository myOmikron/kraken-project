//! The handlers for the aggregated data of domains are located here

use actix_toolbox::tb_middleware::Session;
use actix_web::get;
use actix_web::web::{Data, Json, Path, Query};
use futures::TryStreamExt;
use rorm::{query, Database, FieldAccess, Model};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{
    get_page_params, workspaces, ApiError, ApiResult, DomainResultsPage, PageParams, PathUuid,
};
use crate::models::Domain;

/// A simple representation of a domain in a workspace
#[derive(Serialize, ToSchema)]
pub struct SimpleDomain {
    uuid: Uuid,
    #[schema(example = "example.com")]
    domain: String,
    #[schema(example = "This is a important domain!")]
    comment: String,
    workspace: Uuid,
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
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/domains")]
pub async fn get_all_domains(
    path: Path<PathUuid>,
    query: Query<PageParams>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<Json<DomainResultsPage>> {
    let path = path.into_inner();
    let user_uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (limit, offset) = get_page_params(query).await?;

    let (total,) = query!(&mut tx, (Domain::F.uuid.count()))
        .condition(Domain::F.workspace.equals(path.uuid))
        .one()
        .await?;

    let domains = query!(&mut tx, Domain)
        .condition(Domain::F.workspace.equals(path.uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|x| SimpleDomain {
            uuid: x.uuid,
            domain: x.domain,
            comment: x.comment,
            workspace: *x.workspace.key(),
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(DomainResultsPage {
        items: domains,
        limit,
        offset,
        total: total as u64,
    }))
}
