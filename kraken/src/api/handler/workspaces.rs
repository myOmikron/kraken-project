use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use log::debug;
use rorm::fields::ForeignModelByField;
use rorm::{and, insert, query, update, Database, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{de_optional, ApiError, ApiResult, PathId, UserResponse};
use crate::models::{User, Workspace, WorkspaceInsert, WorkspaceMember};

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateWorkspaceRequest {
    #[schema(example = "secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is super secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CreateWorkspaceResponse {
    #[schema(example = 1)]
    pub(crate) id: i64,
}

/// Create a new workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was created", body = CreateWorkspaceResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateWorkspaceRequest,
    security(("api_key" = []))
)]
#[post("/workspaces")]
pub(crate) async fn create_workspace(
    req: Json<CreateWorkspaceRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<CreateWorkspaceResponse>> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let id = insert!(db.as_ref(), WorkspaceInsert)
        .return_primary_key()
        .single(&WorkspaceInsert {
            name: req.name.clone(),
            description: req.description.clone(),
            owner: ForeignModelByField::Key(uuid),
            deletable: true,
        })
        .await?;

    Ok(Json(CreateWorkspaceResponse { id }))
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
    params(PathId),
    security(("api_key" = []))
)]
#[delete("/workspaces/{id}")]
pub(crate) async fn delete_workspace(
    req: Path<PathId>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let executing_user = query!(&mut tx, User)
        .condition(User::F.uuid.equals(uuid.as_ref()))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    if !workspace.deletable {
        debug!("Workspace {} is not deletable", workspace.id);

        return Err(ApiError::WorkspaceNotDeletable);
    }

    if executing_user.admin || *workspace.owner.key() == executing_user.uuid {
        debug!(
            "Workspace {} got deleted by {}",
            workspace.id, executing_user.username
        );

        rorm::delete!(&mut tx, Workspace).single(&workspace).await?;
    } else {
        debug!(
            "User {} does not has the privileges to delete the workspace {}",
            executing_user.username, workspace.id
        );

        return Err(ApiError::MissingPrivileges);
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetWorkspace {
    #[schema(example = 1337)]
    pub(crate) id: i64,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: UserResponse,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetWorkspaceResponse {
    pub(crate) workspaces: Vec<GetWorkspace>,
}

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the workspace", body = GetWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathId),
    security(("api_key" = []))
)]
#[get("/workspaces/{id}")]
pub(crate) async fn get_workspace(
    req: Path<PathId>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetWorkspace>> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let is_member = query!(&mut tx, (WorkspaceMember::F.id,))
        .condition(and!(
            WorkspaceMember::F.member.equals(uuid.as_ref()),
            WorkspaceMember::F.workspace.equals(req.id as i64)
        ))
        .optional()
        .await?
        .is_some();

    let w = query!(&mut tx, Workspace)
        .condition(Workspace::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    // User is no member of workspace, if it doesn't is the owner, return
    if !is_member && *w.owner.key() != uuid {
        return Err(ApiError::MissingPrivileges);
    }

    let owner = query!(&mut tx, User)
        .condition(User::F.uuid.equals(w.owner.key().as_ref()))
        .optional()
        .await?
        .ok_or(ApiError::InternalServerError)?;

    tx.commit().await?;

    Ok(Json(GetWorkspace {
        id: w.id,
        name: w.name,
        description: w.description,
        owner: UserResponse {
            uuid: owner.uuid,
            username: owner.username,
            display_name: owner.display_name,
        },
    }))
}

/// Retrieve all workspaces owned by executing user
///
/// For administration access, look at the `/admin/workspaces` endpoint.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all workspaces owned by the executing user", body = GetWorkspaceResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub(crate) async fn get_all_workspaces(
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetWorkspaceResponse>> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let workspaces = query!(&mut tx, Workspace)
        .condition(Workspace::F.owner.equals(uuid.as_ref()))
        .all()
        .await?;

    let owner = query!(&mut tx, User)
        .condition(User::F.uuid.equals(uuid.as_ref()))
        .one()
        .await?;

    tx.commit().await?;

    Ok(Json(GetWorkspaceResponse {
        workspaces: workspaces
            .into_iter()
            .map(|w| GetWorkspace {
                id: w.id,
                name: w.name,
                description: w.description,
                owner: UserResponse {
                    uuid: owner.uuid,
                    username: owner.username.clone(),
                    display_name: owner.display_name.clone(),
                },
            })
            .collect(),
    }))
}

/// The request type to update a workspace
///
/// All parameter are optional, but at least one of them must be specified
#[derive(Deserialize, ToSchema)]
pub(crate) struct UpdateWorkspaceRequest {
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
    params(PathId),
    request_body = UpdateWorkspaceRequest,
    security(("api_key" = []))
)]
#[put("/workspaces/{id}")]
pub(crate) async fn update_workspace(
    path: Path<PathId>,
    req: Json<UpdateWorkspaceRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let req = req.into_inner();

    let mut tx = db.start_transaction().await?;

    let w = query!(&mut tx, Workspace)
        .condition(Workspace::F.id.equals(path.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    if *w.owner.key() != uuid {
        return Err(ApiError::MissingPrivileges);
    }

    if let Some(name) = &req.name {
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }
    }

    update!(&mut tx, Workspace)
        .condition(Workspace::F.id.equals(w.id))
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
        (status = 200, description = "Returns the workspace with the given id", body = GetWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathId),
    security(("api_key" = []))
)]
#[get("/workspaces/{id}")]
pub(crate) async fn get_workspace_admin(
    req: Path<PathId>,
    db: Data<Database>,
) -> ApiResult<Json<GetWorkspace>> {
    let mut tx = db.start_transaction().await?;

    let w = query!(&mut tx, Workspace)
        .condition(Workspace::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    let owner = query!(&mut tx, User)
        .condition(User::F.uuid.equals(w.owner.key().as_ref()))
        .one()
        .await?;

    tx.commit().await?;

    Ok(Json(GetWorkspace {
        id: w.id,
        name: w.name,
        description: w.description,
        owner: UserResponse {
            uuid: owner.uuid,
            username: owner.username,
            display_name: owner.display_name,
        },
    }))
}

/// Retrieve all workspaces
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all workspaces", body = GetWorkspaceResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub(crate) async fn get_all_workspaces_admin(
    db: Data<Database>,
) -> ApiResult<Json<GetWorkspaceResponse>> {
    let mut tx = db.start_transaction().await?;

    let workspaces = query!(
        &mut tx,
        (
            Workspace::F.id,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.owner.uuid,
            Workspace::F.owner.username,
            Workspace::F.owner.display_name
        )
    )
    .all()
    .await?;

    tx.commit().await?;

    Ok(Json(GetWorkspaceResponse {
        workspaces: workspaces
            .into_iter()
            .map(
                |(id, name, description, uuid, username, display_name)| GetWorkspace {
                    id,
                    name,
                    description,
                    owner: UserResponse {
                        uuid,
                        username,
                        display_name,
                    },
                },
            )
            .collect(),
    }))
}
