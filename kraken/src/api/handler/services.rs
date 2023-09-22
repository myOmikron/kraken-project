use actix_toolbox::tb_middleware::Session;
use actix_web::get;
use actix_web::web::{Data, Json, Path};
use futures::TryStreamExt;
use rorm::{query, Database, FieldAccess, Model};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{workspaces, ApiError, ApiResult, PathUuid};
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

/// Response of all services
#[derive(Serialize, ToSchema)]
pub struct GetAllServicesResponse {
    services: Vec<SimpleService>,
}

/// List the services of a workspace
#[utoipa::path(
    tag = "Services",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all services of a workspace", body = GetAllServicesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/services")]
pub async fn get_all_services(
    path: Path<PathUuid>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetAllServicesResponse>> {
    let path = path.into_inner();
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let services = query!(&mut tx, Service)
        .condition(Service::F.workspace.equals(path.uuid))
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

    Ok(Json(GetAllServicesResponse { services }))
}
