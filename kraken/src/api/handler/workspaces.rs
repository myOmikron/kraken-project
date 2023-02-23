use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use log::debug;
use rorm::internal::field::foreign_model::ForeignModelByField;
use rorm::{delete, insert, query, Database, ForeignModel, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::handler::{ApiError, ApiResult, PathId};
use crate::models::{User, Workspace, WorkspaceInsert};

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateWorkspaceRequest {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CreateWorkspaceResponse {
    pub(crate) id: i64,
}

#[utoipa::path(
    post,
    context_path = "/api/v1",
    path = "/workspaces",
    tag = "Workspaces",
    responses(
        (status = 200, description = "Workspace was created", body = CreateWorkspaceResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateWorkspaceRequest,
    security(("api_key" = []))
)]
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

#[utoipa::path(
    delete,
    context_path = "/api/v1",
    path = "/workspaces/{id}",
    tag = "Workspaces",
    responses(
        (status = 200, description = "Workspace was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathId),
    security(("api_key" = []))
)]
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

        delete!(&db, Workspace)
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
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetWorkspaceResponse {
    pub(crate) workspaces: Vec<GetWorkspace>,
}

#[utoipa::path(
    get,
    context_path = "/api/v1",
    path = "/workspaces/{id}",
    tag = "Workspaces",
    responses(
        (status = 200, description = "Returns the workspace", body = GetWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathId),
    security(("api_key" = []))
)]
pub(crate) async fn get_workspace(
    req: Path<PathId>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetWorkspace>> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let w = query!(&db, Workspace)
        .condition(Workspace::F.id.equals(req.id as i64))
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

    Ok(Json(GetWorkspace {
        id: w.id,
        name: w.name,
        description: w.description,
    }))
}

#[utoipa::path(
    get,
    context_path = "/api/v1",
    path = "/workspaces",
    tag = "Workspaces",
    responses(
        (status = 200, description = "Returns all workspaces owned by the executing user", body = GetWorkspaceResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
pub(crate) async fn get_all_workspaces(
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetWorkspaceResponse>> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let workspaces = query!(&db, Workspace)
        .condition(Workspace::F.owner.equals(&uuid))
        .all()
        .await?;

    Ok(Json(GetWorkspaceResponse {
        workspaces: workspaces
            .into_iter()
            .map(|w| GetWorkspace {
                id: w.id,
                name: w.name,
                description: w.description,
            })
            .collect(),
    }))
}

#[utoipa::path(
    get,
    context_path = "/api/v1",
    path = "/admin/workspaces/{id}",
    tag = "Admin Workspaces",
    responses(
        (status = 200, description = "Returns the workspace with the given id", body = GetWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathId),
    security(("api_key" = []))
)]
pub(crate) async fn get_workspace_admin(
    req: Path<PathId>,
    db: Data<Database>,
) -> ApiResult<Json<GetWorkspace>> {
    let w = query!(&db, Workspace)
        .condition(Workspace::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    Ok(Json(GetWorkspace {
        id: w.id,
        name: w.name,
        description: w.description,
    }))
}

#[utoipa::path(
    get,
    context_path = "/api/v1",
    path = "/admin/workspaces",
    tag = "Admin Workspaces",
    responses(
        (status = 200, description = "Returns the workspace with the given id", body = GetWorkspaceResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
pub(crate) async fn get_all_workspaces_admin(
    db: Data<Database>,
) -> ApiResult<Json<GetWorkspaceResponse>> {
    let workspaces = query!(&db, Workspace).all().await?;

    Ok(Json(GetWorkspaceResponse {
        workspaces: workspaces
            .into_iter()
            .map(|w| GetWorkspace {
                id: w.id,
                name: w.name,
                description: w.description,
            })
            .collect(),
    }))
}
