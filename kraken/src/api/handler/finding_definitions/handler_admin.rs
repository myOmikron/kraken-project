use actix_web::web::Path;
use actix_web::{delete, HttpResponse};

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::{ApiError, ApiResult};
use crate::api::handler::common::schema::PathUuid;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::UserPermission;

/// Delete a finding definition
#[utoipa::path(
    tag = "Knowledge Base",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Finding Definition was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/findingDefinitions/{uuid}")]
pub async fn delete_finding_definition(
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let uuid = path.into_inner().uuid;

    let user = GLOBAL
        .user_cache
        .get_full_user(user_uuid)
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    if user.permission != UserPermission::Admin {
        return Err(ApiError::MissingPrivileges);
    }

    let deleted = GLOBAL.finding_definition_cache.delete(uuid).await?;

    if deleted == 0 {
        return Err(ApiError::InvalidUuid);
    }

    // Notify every user about deleted finding definition
    GLOBAL
        .ws
        .message_all(WsMessage::DeletedFindingDefinition { uuid })
        .await;

    Ok(HttpResponse::Ok().finish())
}
