//! This module holds the aggregated data of services

use actix_toolbox::tb_middleware::Session;
use actix_web::get;
use actix_web::web::{Data, Json, Path, Query};
use futures::TryStreamExt;
use rorm::{query, Database, FieldAccess, Model};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{
    get_page_params, workspaces, ApiError, ApiResult, PageParams, PathUuid, ServiceResultsPage,
};
use crate::models::Service;

/// A simple representation of a service
#[derive(Serialize, ToSchema)]
pub struct SimpleService {
    uuid: Uuid,
    #[schema(example = "postgresql")]
    name: String,
    #[schema(example = "13.0.1")]
    version: Option<String>,
    host: Uuid,
    port: Option<Uuid>,
    #[schema(example = "Holds all relevant information")]
    comment: String,
    workspace: Uuid,
}

/// List the services of a workspace
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all services of a workspace", body = ServiceResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/services")]
pub async fn get_all_services(
    path: Path<PathUuid>,
    query: Query<PageParams>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<ServiceResultsPage>> {
    let path = path.into_inner();
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (limit, offset) = get_page_params(query).await?;

    let (total,) = query!(&mut tx, (Service::F.uuid.count()))
        .condition(Service::F.workspace.equals(path.uuid))
        .one()
        .await?;

    let services = query!(&mut tx, Service)
        .condition(Service::F.workspace.equals(path.uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|x| SimpleService {
            uuid: x.uuid,
            name: x.name,
            version: x.version,
            host: *x.host.key(),
            port: x.port.map(|y| *y.key()),
            comment: x.comment,
            workspace: *x.workspace.key(),
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(ServiceResultsPage {
        items: services,
        limit,
        offset,
        total: total as u64,
    }))
}
