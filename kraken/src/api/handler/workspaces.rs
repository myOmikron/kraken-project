use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use log::debug;
use rorm::internal::field::foreign_model::ForeignModelByField;
use rorm::{delete, insert, query, Database, ForeignModel, Model};
use serde::{Deserialize, Serialize};

use crate::api::handler::{ApiError, ApiResult};
use crate::models::{User, Workspace, WorkspaceInsert};

#[derive(Deserialize)]
pub(crate) struct CreateWorkspaceRequest {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CreateWorkspaceResponse {
    pub(crate) id: i64,
}

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

#[derive(Deserialize)]
pub(crate) struct DeleteWorkspaceRequest {
    pub(crate) id: u32,
}

pub(crate) async fn delete_workspace(
    req: Path<DeleteWorkspaceRequest>,
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

#[derive(Deserialize)]
pub(crate) struct GetWorkspaceRequest {
    pub(crate) id: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct GetWorkspace {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct GetWorkspaceResponse {
    pub(crate) workspaces: Vec<GetWorkspace>,
}

pub(crate) async fn get_workspaces(
    req: Path<GetWorkspaceRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetWorkspaceResponse>> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let workspaces = if let Some(id) = req.id {
        let w = query!(&db, Workspace)
            .transaction(&mut tx)
            .condition(Workspace::F.id.equals(id as i64))
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
        }
        vec![w]
    } else {
        query!(&db, Workspace)
            .transaction(&mut tx)
            .condition(Workspace::F.owner.equals(&uuid))
            .all()
            .await?
    };

    tx.commit().await?;

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

pub(crate) async fn get_workspaces_admin(
    req: Path<GetWorkspaceRequest>,
    db: Data<Database>,
) -> ApiResult<Json<GetWorkspaceResponse>> {
    let mut tx = db.start_transaction().await?;

    let workspaces = if let Some(id) = req.id {
        let w = query!(&db, Workspace)
            .transaction(&mut tx)
            .condition(Workspace::F.id.equals(id as i64))
            .optional()
            .await?
            .ok_or(ApiError::InvalidId)?;
        vec![w]
    } else {
        query!(&db, Workspace).transaction(&mut tx).all().await?
    };

    tx.commit().await?;

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
