use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use rorm::and;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::workspace_tags::schema::CreateWorkspaceTagRequest;
use crate::api::handler::workspace_tags::schema::FullWorkspaceTag;
use crate::api::handler::workspace_tags::schema::ListWorkspaceTags;
use crate::api::handler::workspace_tags::schema::PathWorkspaceTag;
use crate::api::handler::workspace_tags::schema::UpdateWorkspaceTag;
use crate::chan::global::GLOBAL;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;
use crate::models::Workspace;
use crate::models::WorkspaceTag;

/// Create a workspace tag.
#[utoipa::path(
    tag = "Workspace Tags",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace tag was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = CreateWorkspaceTagRequest,
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/tags")]
pub async fn create_workspace_tag(
    path: Path<PathUuid>,
    req: Json<CreateWorkspaceTagRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();
    let path = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if Workspace::is_user_member_or_owner(&mut tx, path.uuid, user_uuid).await? {
        let uuid = WorkspaceTag::insert(&mut tx, req.name, req.color, path.uuid).await?;

        tx.commit().await?;

        Ok(Json(UuidResponse { uuid }))
    } else {
        Err(ApiError::MissingPrivileges)
    }
}

/// Update a workspace tag
///
/// One of the options must be set
///
/// Requires privileges to access the workspace this tags belongs to.
#[utoipa::path(
    tag = "Workspace Tags",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace tag was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathWorkspaceTag),
    request_body = UpdateWorkspaceTag,
    security(("api_key" = []))
)]
#[put("/workspaces/{w_uuid}/tags/{t_uuid}")]
pub async fn update_workspace_tag(
    req: Json<UpdateWorkspaceTag>,
    path: Path<PathWorkspaceTag>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let path = path.into_inner();
    let req = req.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (WorkspaceTag::F.uuid,))
        .condition(WorkspaceTag::F.uuid.equals(path.t_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(name) = &req.name {
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }

        let name_exists = query!(&mut tx, (WorkspaceTag::F.uuid,))
            .condition(and!(
                WorkspaceTag::F.name.equals(name),
                WorkspaceTag::F.workspace.equals(path.w_uuid)
            ))
            .optional()
            .await?
            .is_some();

        if name_exists {
            return Err(ApiError::NameAlreadyExists);
        }
    }

    update!(&mut tx, WorkspaceTag)
        .condition(WorkspaceTag::F.uuid.equals(path.w_uuid))
        .begin_dyn_set()
        .set_if(WorkspaceTag::F.name, req.name)
        .set_if(WorkspaceTag::F.color, req.color.map(|x| x.into_db()))
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Delete a workspace tag
///
/// Requires privileges to access the workspace this tag belongs to.
#[utoipa::path(
    tag = "Workspace Tags",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace tag was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathWorkspaceTag),
    security(("api_key" = []))
)]
#[delete("/workspaces/{w_uuid}/tags/{t_uuid}")]
pub async fn delete_workspace_tag(
    path: Path<PathWorkspaceTag>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let path = path.into_inner();
    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, path.w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (WorkspaceTag::F.uuid,))
        .condition(and!(
            WorkspaceTag::F.uuid.equals(path.t_uuid),
            WorkspaceTag::F.workspace.equals(path.w_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, WorkspaceTag)
        .condition(WorkspaceTag::F.uuid.equals(path.t_uuid))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Retrieve all workspace tags
#[utoipa::path(
    tag = "Workspace Tags",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all workspace tags", body = ListWorkspaceTags),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/tags")]
pub async fn get_all_workspace_tags(
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<ListWorkspaceTags>> {
    let path = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if Workspace::is_user_member_or_owner(&mut tx, path.uuid, user_uuid).await? {
        let workspace_tags = query!(&mut tx, WorkspaceTag)
            .condition(WorkspaceTag::F.workspace.equals(path.uuid))
            .all()
            .await?;

        tx.commit().await?;

        Ok(Json(ListWorkspaceTags {
            workspace_tags: workspace_tags
                .into_iter()
                .map(|x| FullWorkspaceTag {
                    uuid: x.uuid,
                    name: x.name,
                    color: FromDb::from_db(x.color),
                    workspace: *x.workspace.key(),
                })
                .collect(),
        }))
    } else {
        Err(ApiError::MissingPrivileges)
    }
}
