//! Endpoints for inspecting and revoking a user's oauth decisions

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, HttpResponse};
use futures::TryStreamExt;
use rorm::prelude::*;
use rorm::{and, query, Database};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::{ApiError, ApiResult, PathUuid};
use crate::models::{OAuthDecision, OAuthDecisionAction};

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
    pub scope_workspace: Uuid,

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
    db: Data<Database>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<GetMyDecisionsResponse>> {
    let decisions = query!(
        db.as_ref(),
        (
            OAuthDecision::F.uuid,
            OAuthDecision::F.app.name,
            OAuthDecision::F.scope_workspace,
            OAuthDecision::F.action,
        )
    )
    .condition(OAuthDecision::F.user.equals(user_uuid))
    .stream()
    .map_ok(|(uuid, app, scope_workspace, action)| FullDecision {
        uuid,
        app,
        scope_workspace,
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
    db: Data<Database>,
    SessionUser(user_uuid): SessionUser,
    path: Path<PathUuid>,
) -> ApiResult<HttpResponse> {
    let deleted = rorm::delete!(db.as_ref(), OAuthDecision)
        .condition(and![
            OAuthDecision::F.uuid.equals(path.uuid),
            OAuthDecision::F.user.equals(user_uuid)
        ])
        .await?;

    if deleted > 0 {
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::InvalidUuid)
    }
}
