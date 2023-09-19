use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use futures::TryStreamExt;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use rorm::prelude::*;
use rorm::{and, insert, query, update, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult, PathUuid, SessionUser, UuidResponse};
use crate::models::LeechApiKey;

#[derive(Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    pub name: String,
}

/// Create new api key
#[utoipa::path(
    tag = "Api Keys",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Api key was created successfully", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = CreateApiKeyRequest,
    security(("api_key" = []))
)]
#[post("/apiKeys")]
pub async fn create_api_key(
    req: Json<CreateApiKeyRequest>,
    db: Data<Database>,
    SessionUser(user): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let uuid = Uuid::new_v4();
    insert!(db.as_ref(), LeechApiKey)
        .return_nothing()
        .single(&LeechApiKey {
            uuid,
            user: ForeignModelByField::Key(user),
            key: Alphanumeric.sample_string(&mut thread_rng(), 32),
            name: req.name.clone(),
        })
        .await?;
    Ok(Json(UuidResponse { uuid }))
}

/// Delete an existing api key
#[utoipa::path(
    tag = "Api Keys",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Api key got deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/apiKeys/{uuid}")]
pub async fn delete_api_key(
    path: Path<PathUuid>,
    db: Data<Database>,
    SessionUser(user): SessionUser,
) -> ApiResult<HttpResponse> {
    let deleted = rorm::delete!(db.as_ref(), LeechApiKey)
        .condition(and!(
            LeechApiKey::F.uuid.equals(path.uuid),
            LeechApiKey::F.user.equals(user)
        ))
        .await?;

    if deleted == 0 {
        Err(ApiError::InvalidUuid)
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

#[derive(Serialize, ToSchema)]
pub struct SimpleApiKey {
    /// The key's identifier
    pub uuid: Uuid,

    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    pub name: String,

    /// The actual key's value
    #[schema(example = "fsn83r0jfis84nfthw...")]
    pub key: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetApiKeysResponse {
    keys: Vec<SimpleApiKey>,
}

/// Retrieve all leeches
#[utoipa::path(
    tag = "Api Keys",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The uses api keys", body = GetLeechResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    security(("api_key" = []))
)]
#[get("/apiKeys")]
pub async fn get_api_keys(
    db: Data<Database>,
    SessionUser(user): SessionUser,
) -> ApiResult<Json<GetApiKeysResponse>> {
    let keys = query!(
        db.as_ref(),
        (LeechApiKey::F.key, LeechApiKey::F.name, LeechApiKey::F.uuid)
    )
    .condition(LeechApiKey::F.user.equals(user))
    .stream()
    .map_ok(|(key, name, uuid)| SimpleApiKey { key, name, uuid })
    .try_collect()
    .await?;
    Ok(Json(GetApiKeysResponse { keys }))
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateApiKeyRequest {
    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    name: Option<String>,
}

/// Update an api key by its id
///
/// All parameter are optional, but at least one of them must be specified.
#[utoipa::path(
    tag = "Api Keys",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Api key got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    request_body = UpdateApiKeyRequest,
    security(("api_key" = []))
)]
#[put("/apiKeys/{uuid}")]
pub async fn update_leech(
    path: Path<PathUuid>,
    req: Json<UpdateApiKeyRequest>,
    db: Data<Database>,
    SessionUser(user): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();
    let updated = update!(db.as_ref(), LeechApiKey)
        .begin_dyn_set()
        .set_if(LeechApiKey::F.name, req.name)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .condition(and!(
            LeechApiKey::F.uuid.equals(path.uuid),
            LeechApiKey::F.user.equals(user)
        ))
        .exec()
        .await?;

    if updated == 0 {
        Err(ApiError::InvalidUuid)
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}
