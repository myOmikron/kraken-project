use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use log::debug;
use rorm::internal::field::foreign_model::ForeignModelByField;
use rorm::{and, insert, query, update, Database, ForeignModel, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use webauthn_rs::prelude::Uuid;

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
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let id = insert!(&db, WorkspaceInsert)
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
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let executing_user = query!(&db, User)
        .condition(User::F.uuid.equals(&uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    let workspace = query!(&db, Workspace)
        .transaction(&mut tx)
        .condition(Workspace::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    if !workspace.deletable {
        debug!("Workspace {} is not deletable", workspace.id);

        return Err(ApiError::WorkspaceNotDeletable);
    }

    if executing_user.admin
        || match &workspace.owner {
            ForeignModelByField::Key(v) => v.clone(),
            _ => unreachable!("only key is queried"),
        } == executing_user.uuid
    {
        debug!(
            "Workspace {} got deleted by {}",
            workspace.id, executing_user.username
        );

        rorm::delete!(&db, Workspace)
            .transaction(&mut tx)
            .single(&workspace)
            .await?;
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
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let is_member = query!(&db, (WorkspaceMember::F.id,))
        .transaction(&mut tx)
        .condition(and!(
            WorkspaceMember::F.member.equals(&uuid),
            WorkspaceMember::F.workspace.equals(req.id as i64)
        ))
        .optional()
        .await?
        .is_some();

    let w = query!(&db, Workspace)
        .transaction(&mut tx)
        .condition(Workspace::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    // User is no member of workspace, if it doesn't is the owner, return
    if !is_member {
        match &w.owner {
            ForeignModel::Key(k) => {
                if *k != uuid {
                    return Err(ApiError::MissingPrivileges);
                }
            }
            ForeignModel::Instance(u) => {
                if u.uuid != uuid {
                    return Err(ApiError::MissingPrivileges);
                }
            }
        };
    }

    let owner = query!(&db, User)
        .transaction(&mut tx)
        .condition(User::F.uuid.equals(&match w.owner {
            ForeignModelByField::Key(k) => k,
            ForeignModelByField::Instance(v) => v.uuid,
        }))
        .optional()
        .await?
        .ok_or(ApiError::InternalServerError)?;

    tx.commit().await?;

    Ok(Json(GetWorkspace {
        id: w.id,
        name: w.name,
        description: w.description,
        owner: UserResponse {
            uuid: Uuid::from_slice(&owner.uuid).unwrap(),
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
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let workspaces = query!(&db, Workspace)
        .transaction(&mut tx)
        .condition(Workspace::F.owner.equals(&uuid))
        .all()
        .await?;

    let owner = query!(&db, User)
        .transaction(&mut tx)
        .condition(User::F.uuid.equals(&uuid))
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
                    uuid: Uuid::from_slice(&owner.uuid).unwrap(),
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
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let w = query!(&db, Workspace)
        .transaction(&mut tx)
        .condition(Workspace::F.id.equals(path.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    match &w.owner {
        ForeignModel::Key(k) => {
            if *k != uuid {
                return Err(ApiError::MissingPrivileges);
            }
        }
        ForeignModel::Instance(u) => {
            if u.uuid != uuid {
                return Err(ApiError::MissingPrivileges);
            }
        }
    };

    let mut ub = update!(&db, Workspace)
        .condition(Workspace::F.id.equals(w.id))
        .begin_dyn_set();

    if let Some(name) = &req.name {
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }

        ub = ub.set(Workspace::F.name, name);
    }

    if let Some(description) = &req.description {
        ub = ub.set(Workspace::F.description, description.as_ref());
    }

    ub.transaction(&mut tx)
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

    let w = query!(&db, Workspace)
        .transaction(&mut tx)
        .condition(Workspace::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    let owner = query!(&db, User)
        .transaction(&mut tx)
        .condition(User::F.uuid.equals(&match w.owner {
            ForeignModelByField::Key(k) => k,
            ForeignModelByField::Instance(x) => x.uuid,
        }))
        .one()
        .await?;

    tx.commit().await?;

    Ok(Json(GetWorkspace {
        id: w.id,
        name: w.name,
        description: w.description,
        owner: UserResponse {
            uuid: Uuid::from_slice(&owner.uuid).unwrap(),
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
        &db,
        (
            Workspace::F.id,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.owner.f().uuid,
            Workspace::F.owner.f().username,
            Workspace::F.owner.f().display_name
        )
    )
    .transaction(&mut tx)
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
                        uuid: Uuid::from_slice(&uuid).unwrap(),
                        username,
                        display_name,
                    },
                },
            )
            .collect(),
    }))
}
