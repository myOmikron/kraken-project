//!

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

use crate::api::extractors::SessionUser;
use crate::api::handler::{ApiError, ApiResult, PathUuid, UuidResponse};
use crate::models::LeechApiKey;

/// Request to create a new api key
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

/// A representation of a full api key
#[derive(Serialize, ToSchema)]
pub struct FullApiKey {
    /// The key's identifier
    pub uuid: Uuid,

    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    pub name: String,

    /// The actual key's value
    #[schema(example = "fsn83r0jfis84nfthw...")]
    pub key: String,
}

/// The response that contains all api keys
#[derive(Serialize, ToSchema)]
pub struct GetApiKeysResponse {
    keys: Vec<FullApiKey>,
}

/// Retrieve all api keys
#[utoipa::path(
    tag = "Api Keys",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The uses api keys", body = GetApiKeysResponse),
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
    .map_ok(|(key, name, uuid)| FullApiKey { key, name, uuid })
    .try_collect()
    .await?;
    Ok(Json(GetApiKeysResponse { keys }))
}

/// The request to update an api key
#[derive(Deserialize, ToSchema)]
pub struct UpdateApiKeyRequest {
    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    name: String,
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
pub async fn update_api_key(
    path: Path<PathUuid>,
    req: Json<UpdateApiKeyRequest>,
    db: Data<Database>,
    SessionUser(user): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();

    if req.name.is_empty() {
        return Err(ApiError::InvalidName);
    }

    let updated = update!(db.as_ref(), LeechApiKey)
        .set(LeechApiKey::F.name, req.name)
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
