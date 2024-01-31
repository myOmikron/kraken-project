use actix_web::web::Path;
use actix_web::{delete, HttpResponse};
use rorm::{query, FieldAccess, Model};

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::{ApiError, ApiResult};
use crate::api::handler::common::schema::PathUuid;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::{FindingDefinition, User, UserPermission};

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

    let mut tx = GLOBAL.db.start_transaction().await?;

    let user = query!(&mut tx, User)
        .condition(User::F.uuid.equals(user_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InternalServerError)?;

    if user.permission != UserPermission::Admin {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (FindingDefinition::F.uuid,))
        .condition(FindingDefinition::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, FindingDefinition)
        .condition(FindingDefinition::F.uuid.equals(uuid))
        .await?;

    tx.commit().await?;

    // Notify every user about deleted finding definition
    GLOBAL
        .ws
        .message_all(WsMessage::DeletedFindingDefinition { uuid })
        .await;

    Ok(HttpResponse::Ok().finish())
}
