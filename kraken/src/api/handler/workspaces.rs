use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use chrono::{DateTime, TimeZone, Utc};
use log::debug;
use rorm::fields::ForeignModelByField;
use rorm::transaction::Transaction;
use rorm::{and, insert, query, update, Database, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{
    de_optional, query_user, ApiError, ApiResult, PathId, SimpleAttack, UserResponse,
};
use crate::models::{Attack, User, Workspace, WorkspaceInsert, WorkspaceMember};

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
    let mut tx = db.start_transaction().await?;

    let executing_user = query_user(&mut tx, &session).await?;

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

/// A simple version of a workspace
#[derive(Serialize, ToSchema)]
pub(crate) struct SimpleWorkspace {
    #[schema(example = 1337)]
    pub(crate) id: i64,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: UserResponse,
    pub(crate) created_at: DateTime<Utc>,
}

/// A full version of a workspace
#[derive(Serialize, ToSchema)]
pub(crate) struct FullWorkspace {
    #[schema(example = 1337)]
    pub(crate) id: i64,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: UserResponse,
    pub(crate) attacks: Vec<SimpleAttack>,
    pub(crate) members: Vec<UserResponse>,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetWorkspaceResponse {
    pub(crate) workspaces: Vec<SimpleWorkspace>,
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
    params(PathId),
    security(("api_key" = []))
)]
#[get("/workspaces/{id}")]
pub(crate) async fn get_workspace(
    req: Path<PathId>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<FullWorkspace>> {
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

    let is_owner = query!(&mut tx, (Workspace::F.id,))
        .condition(and!(
            Workspace::F.id.equals(req.id as i64),
            Workspace::F.owner.equals(uuid.as_ref())
        ))
        .optional()
        .await?
        .is_some();

    let workspace = if is_owner || is_member {
        get_workspace_unchecked(req.id as i64, &mut tx).await
    } else {
        Err(ApiError::MissingPrivileges)
    };

    tx.commit().await?;

    Ok(Json(workspace?))
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
    let mut tx = db.start_transaction().await?;

    let owner = query_user(&mut tx, &session).await?;

    let workspaces = query!(&mut tx, Workspace)
        .condition(Workspace::F.owner.equals(owner.uuid.as_ref()))
        .all()
        .await?;

    tx.commit().await?;

    Ok(Json(GetWorkspaceResponse {
        workspaces: workspaces
            .into_iter()
            .map(|w| SimpleWorkspace {
                id: w.id,
                name: w.name,
                description: w.description,
                owner: UserResponse {
                    uuid: owner.uuid,
                    username: owner.username.clone(),
                    display_name: owner.display_name.clone(),
                },
                created_at: w.created_at.and_utc(),
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
        (status = 200, description = "Returns the workspace with the given id", body = FullWorkspace),
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
) -> ApiResult<Json<FullWorkspace>> {
    let mut tx = db.start_transaction().await?;

    let workspace = get_workspace_unchecked(req.id as i64, &mut tx).await;

    tx.commit().await?;

    Ok(Json(workspace?))
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
            Workspace::F.created_at,
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
                |(id, name, description, created_at, uuid, username, display_name)| {
                    SimpleWorkspace {
                        id,
                        name,
                        description,
                        owner: UserResponse {
                            uuid,
                            username,
                            display_name,
                        },
                        created_at: created_at.and_utc(),
                    }
                },
            )
            .collect(),
    }))
}

/// Get a [`FullWorkspace`] by its id without permission checks
async fn get_workspace_unchecked(id: i64, tx: &mut Transaction) -> ApiResult<FullWorkspace> {
    let workspace = query!(&mut *tx, Workspace)
        .condition(Workspace::F.id.equals(id))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    let owner = query!(&mut *tx, User)
        .condition(User::F.uuid.equals(workspace.owner.key().as_ref()))
        .one()
        .await?;

    let attacks = query!(
        &mut *tx,
        (
            Attack::F.id,
            Attack::F.attack_type,
            Attack::F.finished_at,
            Attack::F.created_at,
            Attack::F.started_from.uuid,
            Attack::F.started_from.username,
            Attack::F.started_from.display_name,
        )
    )
    .condition(Attack::F.workspace.equals(id))
    .all()
    .await?
    .into_iter()
    .map(
        |(attack_id, attack_type, finished_at, created_at, uuid, username, display_name)| {
            SimpleAttack {
                id: attack_id,
                workspace_id: id,
                attack_type: attack_type.into(),
                started_from: UserResponse {
                    uuid,
                    username,
                    display_name,
                },
                finished_at: finished_at.map(|finished_at| Utc.from_utc_datetime(&finished_at)),
                created_at: Utc.from_utc_datetime(&created_at),
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
    .condition(WorkspaceMember::F.workspace.equals(id))
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
        id: workspace.id,
        name: workspace.name,
        description: workspace.description,
        owner: UserResponse {
            uuid: owner.uuid,
            username: owner.username,
            display_name: owner.display_name,
        },
        attacks,
        members,
        created_at: workspace.created_at.and_utc(),
    })
}
