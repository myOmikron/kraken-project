//! Invitations to workspaces are defined here

use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, HttpResponse};
use rorm::{query, Database, FieldAccess, Model};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::users::SimpleUser;
use crate::api::handler::workspaces::SimpleWorkspace;
use crate::api::handler::{ApiError, ApiResult, PathUuid};
use crate::models::{User, Workspace, WorkspaceInvitation};

/// The full representation of an invitation to a workspace
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FullWorkspaceInvitation {
    /// The uuid of the invitation
    pub uuid: Uuid,
    /// The workspace the invitation is
    pub workspace: SimpleWorkspace,
    /// The user that has issued the invitation
    pub from: SimpleUser,
}

/// A list of invitations to workspaces
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WorkspaceInvitationList {
    /// All invitations of the current user
    pub invitations: Vec<FullWorkspaceInvitation>,
}

/// Retrieve all open invitations to workspaces the currently logged-in user
/// has retrieved
#[utoipa::path(
    tag = "Workspace Invitations",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all invitations of a user", body = WorkspaceInvitationList),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/invitations")]
pub async fn get_all_workspace_invitations(
    SessionUser(session_user): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<WorkspaceInvitationList>> {
    let mut tx = db.start_transaction().await?;

    let mut invitations = vec![];

    for (uuid, workspace, from, owner) in query!(
        &mut tx,
        (
            WorkspaceInvitation::F.uuid,
            WorkspaceInvitation::F.workspace as Workspace,
            WorkspaceInvitation::F.from as User,
            WorkspaceInvitation::F.workspace.owner as User,
        )
    )
    .condition(WorkspaceInvitation::F.target.equals(session_user))
    .all()
    .await?
    {
        invitations.push(FullWorkspaceInvitation {
            uuid,
            workspace: SimpleWorkspace {
                uuid: workspace.uuid,
                name: workspace.name,
                description: workspace.description,
                owner: SimpleUser {
                    uuid: owner.uuid,
                    username: owner.username,
                    display_name: owner.display_name,
                },
                created_at: workspace.created_at,
            },
            from: SimpleUser {
                uuid: from.uuid,
                username: from.username,
                display_name: from.display_name,
            },
        });
    }

    tx.commit().await?;

    Ok(Json(WorkspaceInvitationList { invitations }))
}

/// Accept an invitation to a workspace
#[utoipa::path(
    tag = "Workspace Invitations",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Accept an invitation"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/invitations/{uuid}/accept")]
pub async fn accept_invitation(
    path: Path<PathUuid>,
    db: Data<Database>,
    SessionUser(session_user): SessionUser,
) -> ApiResult<HttpResponse> {
    let invitation_uuid = path.into_inner().uuid;

    let mut tx = db.start_transaction().await?;

    let invitation = query!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(invitation_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if *invitation.target.key() != session_user {
        return Err(ApiError::MissingPrivileges);
    }

    Workspace::add_member(&mut tx, *invitation.workspace.key(), session_user).await?;

    rorm::delete!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(invitation_uuid))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
