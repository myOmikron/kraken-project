use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;

use crate::api::handler::bearer_tokens::schema::CreateBearerTokenRequest;
use crate::api::handler::bearer_tokens::schema::FullBearerToken;
use crate::api::handler::bearer_tokens::schema::ListBearerTokens;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::findings::utils::ApiResult;
use crate::chan::global::GLOBAL;
use crate::models::BearerToken;

/// Create a new bearer token
#[utoipa::path(
    tag = "Bearer Token",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Bearer token was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateBearerTokenRequest,
    security(("api_key" = []))
)]
#[post("/bearer")]
pub async fn create_bearer_token(
    req: Json<CreateBearerTokenRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let CreateBearerTokenRequest { name } = req.into_inner();

    let uuid = BearerToken::insert(&GLOBAL.db, name).await?;

    Ok(Json(UuidResponse { uuid }))
}

/// List all available bearer tokens
#[utoipa::path(
    tag = "Bearer Token",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "List of all bearer tokens", body = ListBearerTokens),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/bearer")]
pub async fn list_all_bearer_tokens() -> ApiResult<Json<ListBearerTokens>> {
    let tokens = query!(&GLOBAL.db, BearerToken)
        .stream()
        .map_ok(|t| FullBearerToken {
            uuid: t.uuid,
            name: t.name,
            token: t.token,
        })
        .try_collect()
        .await?;

    Ok(Json(ListBearerTokens { tokens }))
}

/// Delete an existing token
#[utoipa::path(
    tag = "Bearer Token",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Bearer token was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/bearer/{uuid}")]
pub async fn delete_bearer_token(path: Path<PathUuid>) -> ApiResult<HttpResponse> {
    let PathUuid { uuid } = path.into_inner();

    let deleted = rorm::delete!(&GLOBAL.db, BearerToken)
        .condition(BearerToken::F.uuid.equals(uuid))
        .await?;

    if deleted > 0 {
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::InvalidUuid)
    }
}
