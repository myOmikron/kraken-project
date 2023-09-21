use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use rorm::prelude::ForeignModelByField;
use rorm::{and, insert, query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::{workspaces, ApiError, ApiResult, Color, PathUuid, UuidResponse};
use crate::models::{GlobalTag, WorkspaceTag, WorkspaceTagInsert};

/// The request to create a workspace tag
#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateWorkspaceTagRequest {
    /// Name of the tag
    pub name: String,
    /// Color of a tag
    pub color: Color,
}

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
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<UuidResponse>> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
    let req = req.into_inner();
    let path = path.into_inner();

    let mut tx = db.start_transaction().await?;

    if req.name.is_empty() {
        return Err(ApiError::InvalidName);
    }

    if workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.uuid).await? {
        if query!(&mut tx, (WorkspaceTag::F.uuid,))
            .condition(and!(
                WorkspaceTag::F.name.equals(&req.name),
                WorkspaceTag::F.workspace.equals(path.uuid)
            ))
            .optional()
            .await?
            .is_some()
        {
            return Err(ApiError::NameAlreadyExists);
        }

        let uuid = insert!(&mut tx, WorkspaceTagInsert)
            .return_primary_key()
            .single(&WorkspaceTagInsert {
                uuid: Uuid::new_v4(),
                name: req.name,
                color: req.color.into(),
                workspace: ForeignModelByField::Key(path.uuid),
            })
            .await?;

        tx.commit().await?;

        Ok(Json(UuidResponse { uuid }))
    } else {
        Err(ApiError::MissingPrivileges)
    }
}

/// The full representation of a full workspace tag
#[derive(Serialize, ToSchema, Debug)]
pub struct FullWorkspaceTag {
    pub(crate) uuid: Uuid,
    #[schema(example = "seems broken")]
    pub(crate) name: String,
    pub(crate) color: Color,
    pub(crate) workspace: Uuid,
}

/// The response to a request to retrieve all workspace tags
#[derive(Serialize, ToSchema, Debug)]
pub struct GetWorkspaceTagsResponse {
    pub(crate) workspace_tags: Vec<FullWorkspaceTag>,
}

/// Retrieve all workspace tags
#[utoipa::path(
    tag = "Workspace Tags",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all workspace tags", body = GetWorkspaceTagsResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/tags")]
pub async fn get_all_workspace_tags(
    path: Path<PathUuid>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetWorkspaceTagsResponse>> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
    let path = path.into_inner();

    let mut tx = db.start_transaction().await?;

    if workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.uuid).await? {
        let workspace_tags = query!(&mut tx, WorkspaceTag)
            .condition(WorkspaceTag::F.workspace.equals(path.uuid))
            .all()
            .await?;

        tx.commit().await?;

        Ok(Json(GetWorkspaceTagsResponse {
            workspace_tags: workspace_tags
                .into_iter()
                .map(|x| FullWorkspaceTag {
                    uuid: x.uuid,
                    name: x.name,
                    color: x.color.into(),
                    workspace: *x.workspace.key(),
                })
                .collect(),
        }))
    } else {
        Err(ApiError::MissingPrivileges)
    }
}

/// The request to update a workspace tag
#[derive(Deserialize, ToSchema)]
pub struct UpdateWorkspaceTag {
    name: Option<String>,
    color: Option<Color>,
}

/// The path of a workspace tag
#[derive(Deserialize, IntoParams)]
pub struct PathWorkspaceTag {
    w_uuid: Uuid,
    t_uuid: Uuid,
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
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
    let path = path.into_inner();
    let req = req.into_inner();

    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.w_uuid).await? {
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
        .begin_dyn_set()
        .set_if(WorkspaceTag::F.name, req.name)
        .set_if(WorkspaceTag::F.color, req.color.map(|x| x.into()))
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
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
    let path = path.into_inner();
    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.w_uuid).await? {
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
        .condition(GlobalTag::F.uuid.equals(path.t_uuid))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
