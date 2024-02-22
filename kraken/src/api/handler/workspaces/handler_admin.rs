use actix_web::get;
use actix_web::web::Json;
use actix_web::web::Path;
use rorm::query;
use rorm::Model;

use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::FullWorkspace;
use crate::api::handler::workspaces::schema::ListWorkspaces;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::api::handler::workspaces::utils::get_workspace_unchecked;
use crate::chan::global::GLOBAL;
use crate::models::Workspace;

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the workspace with the given id", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}")]
pub async fn get_workspace_admin(req: Path<PathUuid>) -> ApiResult<Json<FullWorkspace>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspace = get_workspace_unchecked(req.uuid, &mut tx).await;

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// Retrieve all workspaces
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all workspaces", body = ListWorkspaces),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces_admin() -> ApiResult<Json<ListWorkspaces>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspaces = query!(
        &mut tx,
        (
            Workspace::F.uuid,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.created_at,
            Workspace::F.owner.uuid,
            Workspace::F.owner.username,
            Workspace::F.owner.display_name
        )
    )
    .all()
    .await?;

    tx.commit().await?;

    Ok(Json(ListWorkspaces {
        workspaces: workspaces
            .into_iter()
            .map(
                |(uuid, name, description, created_at, by_uuid, username, display_name)| {
                    SimpleWorkspace {
                        uuid,
                        name,
                        description,
                        owner: SimpleUser {
                            uuid: by_uuid,
                            username,
                            display_name,
                        },
                        created_at,
                    }
                },
            )
            .collect(),
    }))
}
