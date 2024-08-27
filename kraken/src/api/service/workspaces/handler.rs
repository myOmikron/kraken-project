use actix_web::post;
use actix_web::web::Json;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::service::workspaces::schema::CreateWorkspaceRequest;
use crate::chan::global::GLOBAL;
use crate::models::User;
use crate::models::Workspace;

/// Create a workspace and set its owner
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateWorkspaceRequest,
    security(("bearer_token" = []))
    )]
#[post("/workspaces")]
pub async fn create_workspace(req: Json<CreateWorkspaceRequest>) -> ApiResult<Json<UuidResponse>> {
    let CreateWorkspaceRequest {
        name,
        description,
        owner,
    } = req.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !User::exists(&mut tx, owner).await? {
        return Err(ApiError::InvalidUuid);
    }

    let uuid = Workspace::insert(&mut tx, name, description, owner).await?;

    tx.commit().await?;

    Ok(Json(UuidResponse { uuid }))
}
