use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::workspace_invitations::schema::FullWorkspaceInvitation;
use crate::api::handler::workspace_invitations::schema::WorkspaceInvitationList;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::chan::global::GLOBAL;
use crate::models::Workspace;
use crate::models::WorkspaceInvitation;
use crate::models::WorkspaceMemberPermission;

/// Retrieve all open invitations to workspaces the currently logged-in user
/// has retrieved
#[swaggapi::get("/invitations", tags("Workspace Invitations"))]
pub async fn get_all_invitations(
    SessionUser(session_user): SessionUser,
) -> ApiResult<Json<WorkspaceInvitationList>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let mut invitations = vec![];

    for (uuid, workspace, from, target, owner) in query!(
        &mut tx,
        (
            WorkspaceInvitation::F.uuid,
            WorkspaceInvitation::F.workspace as Workspace,
            WorkspaceInvitation::F.from,
            WorkspaceInvitation::F.target,
            WorkspaceInvitation::F.workspace.owner,
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
                owner: GLOBAL
                    .user_cache
                    .get_simple_user(*owner.key())
                    .await?
                    .ok_or(ApiError::InternalServerError)?,
                created_at: workspace.created_at,
                archived: workspace.archived,
            },
            target: GLOBAL
                .user_cache
                .get_simple_user(*target.key())
                .await?
                .ok_or(ApiError::InternalServerError)?,
            from: GLOBAL
                .user_cache
                .get_simple_user(*from.key())
                .await?
                .ok_or(ApiError::InternalServerError)?,
        });
    }

    tx.commit().await?;

    Ok(Json(WorkspaceInvitationList { invitations }))
}

/// Accept an invitation to a workspace
#[swaggapi::post("/invitations/{uuid}/accept", tags("Workspace Invitations"))]
pub async fn accept_invitation(
    path: Path<PathUuid>,
    SessionUser(session_user): SessionUser,
) -> ApiResult<HttpResponse> {
    let invitation_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;

    let invitation = query!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(invitation_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if *invitation.target.key() != session_user {
        return Err(ApiError::MissingPrivileges);
    }

    Workspace::add_member(
        &mut tx,
        *invitation.workspace.key(),
        session_user,
        WorkspaceMemberPermission::ReadWrite,
    )
    .await?;

    rorm::delete!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(invitation_uuid))
        .await?;

    // Refresh cache
    GLOBAL
        .workspace_users_cache
        .refresh_users(*invitation.workspace.key(), &mut tx)
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Decline an invitation to a workspace
#[swaggapi::post("/invitations/{uuid}/decline", tags("Workspace Invitations"))]
pub async fn decline_invitation(
    path: Path<PathUuid>,
    SessionUser(session_user): SessionUser,
) -> ApiResult<HttpResponse> {
    let invitation_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;

    let invitation = query!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(invitation_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if *invitation.target.key() != session_user {
        return Err(ApiError::MissingPrivileges);
    }

    rorm::delete!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(invitation_uuid))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
