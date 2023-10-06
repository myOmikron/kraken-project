//! Everything regarding workspace management is located in this module

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use chrono::{DateTime, Utc};
use log::debug;
use rorm::db::transaction::Transaction;
use rorm::db::Executor;
use rorm::{and, query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::attacks::SimpleAttack;
use crate::api::handler::users::UserResponse;
use crate::api::handler::{de_optional, query_user, ApiError, ApiResult, PathUuid, UuidResponse};
use crate::models::{Attack, User, Workspace, WorkspaceMember};

/// The request to create a new workspace
#[derive(Deserialize, ToSchema)]
pub struct CreateWorkspaceRequest {
    #[schema(example = "secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is super secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
}

/// Create a new workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateWorkspaceRequest,
    security(("api_key" = []))
)]
#[post("/workspaces")]
pub async fn create_workspace(
    req: Json<CreateWorkspaceRequest>,
    db: Data<Database>,
    session: SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();

    let uuid = Workspace::insert(db.as_ref(), req.name, req.description, session.0).await?;

    Ok(Json(UuidResponse { uuid }))
}

/// Delete a workspace by its id
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/workspaces/{uuid}")]
pub async fn delete_workspace(
    req: Path<PathUuid>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    let executing_user = query_user(&mut tx, &session).await?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if executing_user.admin || *workspace.owner.key() == executing_user.uuid {
        debug!(
            "Workspace {} got deleted by {}",
            workspace.uuid, executing_user.username
        );

        rorm::delete!(&mut tx, Workspace).single(&workspace).await?;
    } else {
        debug!(
            "User {} does not has the privileges to delete the workspace {}",
            executing_user.username, workspace.uuid
        );

        return Err(ApiError::MissingPrivileges);
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// A simple version of a workspace
#[derive(Serialize, ToSchema)]
pub struct SimpleWorkspace {
    pub(crate) uuid: Uuid,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: UserResponse,
    pub(crate) created_at: DateTime<Utc>,
}

/// A full version of a workspace
#[derive(Serialize, ToSchema)]
pub struct FullWorkspace {
    pub(crate) uuid: Uuid,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: UserResponse,
    pub(crate) attacks: Vec<SimpleAttack>,
    pub(crate) members: Vec<UserResponse>,
    pub(crate) created_at: DateTime<Utc>,
}

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the workspace", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}")]
pub async fn get_workspace(
    req: Path<PathUuid>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<FullWorkspace>> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let workspace = if is_user_member_or_owner(&mut tx, user_uuid, req.uuid).await? {
        get_workspace_unchecked(req.uuid, &mut tx).await
    } else {
        Err(ApiError::MissingPrivileges)
    };

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// The response to retrieve all workspaces
#[derive(Serialize, ToSchema)]
pub struct GetAllWorkspacesResponse {
    pub(crate) workspaces: Vec<SimpleWorkspace>,
}

/// Retrieve all workspaces owned by executing user
///
/// For administration access, look at the `/admin/workspaces` endpoint.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all workspaces owned by the executing user", body = GetAllWorkspacesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces(
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetAllWorkspacesResponse>> {
    let mut tx = db.start_transaction().await?;

    let owner = query_user(&mut tx, &session).await?;

    let workspaces = query!(&mut tx, Workspace)
        .condition(Workspace::F.owner.equals(owner.uuid))
        .all()
        .await?;

    tx.commit().await?;

    Ok(Json(GetAllWorkspacesResponse {
        workspaces: workspaces
            .into_iter()
            .map(|w| SimpleWorkspace {
                uuid: w.uuid,
                name: w.name,
                description: w.description,
                owner: UserResponse {
                    uuid: owner.uuid,
                    username: owner.username.clone(),
                    display_name: owner.display_name.clone(),
                },
                created_at: w.created_at,
            })
            .collect(),
    }))
}

/// The request type to update a workspace
///
/// All parameter are optional, but at least one of them must be specified
#[derive(Deserialize, ToSchema)]
pub struct UpdateWorkspaceRequest {
    #[schema(example = "Workspace for work")]
    name: Option<String>,
    #[schema(example = "This workspace is for work and for work only!")]
    #[serde(deserialize_with = "de_optional")]
    description: Option<Option<String>>,
}

/// Updates a workspace by its id
///
/// All parameter are optional, but at least one of them must be specified.
///
/// `name` must not be empty.
///
/// You can set `description` to null to remove the description from the database.
/// If you leave the parameter out, the description will remain unchanged.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = UpdateWorkspaceRequest,
    security(("api_key" = []))
)]
#[put("/workspaces/{uuid}")]
pub async fn update_workspace(
    path: Path<PathUuid>,
    req: Json<UpdateWorkspaceRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let req = req.into_inner();

    let mut tx = db.start_transaction().await?;

    let w = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if *w.owner.key() != uuid {
        return Err(ApiError::MissingPrivileges);
    }

    if let Some(name) = &req.name {
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }
    }

    update!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(w.uuid))
        .begin_dyn_set()
        .set_if(Workspace::F.name, req.name)
        .set_if(Workspace::F.description, req.description)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

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
pub async fn get_workspace_admin(
    req: Path<PathUuid>,
    db: Data<Database>,
) -> ApiResult<Json<FullWorkspace>> {
    let mut tx = db.start_transaction().await?;

    let workspace = get_workspace_unchecked(req.uuid, &mut tx).await;

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// Retrieve all workspaces
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all workspaces", body = GetAllWorkspacesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces_admin(
    db: Data<Database>,
) -> ApiResult<Json<GetAllWorkspacesResponse>> {
    let mut tx = db.start_transaction().await?;

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

    Ok(Json(GetAllWorkspacesResponse {
        workspaces: workspaces
            .into_iter()
            .map(
                |(uuid, name, description, created_at, by_uuid, username, display_name)| {
                    SimpleWorkspace {
                        uuid,
                        name,
                        description,
                        owner: UserResponse {
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

/// Get a [`FullWorkspace`] by its uuid without permission checks
async fn get_workspace_unchecked(uuid: Uuid, tx: &mut Transaction) -> ApiResult<FullWorkspace> {
    let workspace = query!(&mut *tx, Workspace)
        .condition(Workspace::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let owner = query!(&mut *tx, User)
        .condition(User::F.uuid.equals(*workspace.owner.key()))
        .one()
        .await?;

    let attacks = query!(
        &mut *tx,
        (
            Attack::F.uuid,
            Attack::F.attack_type,
            Attack::F.finished_at,
            Attack::F.created_at,
            Attack::F.started_by.uuid,
            Attack::F.started_by.username,
            Attack::F.started_by.display_name,
        )
    )
    .condition(Attack::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(
        |(attack_uuid, attack_type, finished_at, created_at, by_uuid, username, display_name)| {
            SimpleAttack {
                uuid: attack_uuid,
                workspace_uuid: uuid,
                attack_type,
                started_from: UserResponse {
                    uuid: by_uuid,
                    username,
                    display_name,
                },
                finished_at,
                created_at,
            }
        },
    )
    .collect();

    let members = query!(
        &mut *tx,
        (
            WorkspaceMember::F.member.uuid,
            WorkspaceMember::F.member.username,
            WorkspaceMember::F.member.display_name
        )
    )
    .condition(WorkspaceMember::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(|(uuid, username, display_name)| UserResponse {
        uuid,
        username,
        display_name,
    })
    .collect();

    Ok(FullWorkspace {
        uuid: workspace.uuid,
        name: workspace.name,
        description: workspace.description,
        owner: UserResponse {
            uuid: owner.uuid,
            username: owner.username,
            display_name: owner.display_name,
        },
        attacks,
        members,
        created_at: workspace.created_at,
    })
}

/// Check whether a user is privileged to access a workspace
pub async fn is_user_member_or_owner(
    tx: impl Executor<'_>,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
) -> ApiResult<bool> {
    let mut tx = tx.ensure_transaction().await?;

    let is_member = query!(tx.get_transaction(), (WorkspaceMember::F.id,))
        .condition(and!(
            WorkspaceMember::F.member.equals(user_uuid),
            WorkspaceMember::F.workspace.equals(workspace_uuid)
        ))
        .optional()
        .await?
        .is_some();

    let is_owner = query!(tx.get_transaction(), (Workspace::F.uuid,))
        .condition(and!(
            Workspace::F.uuid.equals(workspace_uuid),
            Workspace::F.owner.equals(user_uuid)
        ))
        .optional()
        .await?
        .is_some();

    tx.commit().await?;

    Ok(is_member || is_owner)
}
