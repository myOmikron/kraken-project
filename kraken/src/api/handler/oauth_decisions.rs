//! Endpoints for inspecting and revoking a user's oauth decisions

use actix_web::web::{Json, Path};
use actix_web::{delete, get, HttpResponse};
use futures::TryStreamExt;
use rorm::prelude::*;
use rorm::{and, query};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::users::SimpleUser;
use crate::api::handler::workspaces::SimpleWorkspace;
use crate::api::handler::{ApiError, ApiResult, PathUuid};
use crate::chan::GLOBAL;
use crate::models::{OAuthDecision, OAuthDecisionAction, Workspace, WorkspaceAccessToken};

/// Response holding a user's oauth decisions
#[derive(Serialize, ToSchema)]
pub struct GetMyDecisionsResponse {
    /// A user's oauth decisions
    pub decisions: Vec<FullDecision>,
}

/// A user's remembered oauth decision
#[derive(Serialize, ToSchema)]
pub struct FullDecision {
    /// The primary key
    pub uuid: Uuid,

    /// The application the decision was made for
    pub app: String,

    /// The requested workspace
    pub workspace: SimpleWorkspace,

    /// Action what to do with new incoming oauth requests
    #[schema(inline)]
    pub action: OAuthDecisionAction,
}

/// Retrieve a user's remembered oauth decisions
#[utoipa::path(
    tag = "OAuth Decisions",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The user's remember oauth decisions", body = GetMyDecisionsResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/oauthDecisions")]
pub async fn get_decisions(
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<GetMyDecisionsResponse>> {
    let decisions = query!(
        &GLOBAL.db,
        (
            OAuthDecision::F.uuid,
            OAuthDecision::F.application.name,
            OAuthDecision::F.workspace as Workspace,
            OAuthDecision::F.workspace.owner as SimpleUser,
            OAuthDecision::F.action,
        )
    )
    .condition(OAuthDecision::F.user.equals(user_uuid))
    .stream()
    .map_ok(|(uuid, app, workspace, owner, action)| FullDecision {
        uuid,
        app,
        workspace: SimpleWorkspace {
            uuid: workspace.uuid,
            name: workspace.name,
            description: workspace.description,
            created_at: workspace.created_at,
            owner,
        },
        action,
    })
    .try_collect()
    .await?;
    Ok(Json(GetMyDecisionsResponse { decisions }))
}

/// Revoke a user's remembered oauth decision
#[utoipa::path(
    tag = "OAuth Decisions",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Revoked decision"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = [])),
)]
#[delete("/oauthDecisions/{uuid}")]
pub async fn revoke_decision(
    SessionUser(user_uuid): SessionUser,
    path: Path<PathUuid>,
) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let decision = query!(&mut tx, OAuthDecision)
        .condition(OAuthDecision::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if *decision.user.key() != user_uuid {
        return Err(ApiError::MissingPrivileges);
    }

    rorm::delete!(&GLOBAL.db, OAuthDecision)
        .condition(OAuthDecision::F.uuid.equals(path.uuid))
        .await?;

    rorm::delete!(&mut tx, WorkspaceAccessToken)
        .condition(and!(
            WorkspaceAccessToken::F.user.equals(user_uuid),
            WorkspaceAccessToken::F
                .workspace
                .equals(*decision.workspace.key()),
            WorkspaceAccessToken::F
                .application
                .equals(*decision.application.key())
        ))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
