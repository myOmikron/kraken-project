use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rand::thread_rng;
use rorm::and;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::api_keys::schema::CreateApiKeyRequest;
use crate::api::handler::api_keys::schema::FullApiKey;
use crate::api::handler::api_keys::schema::ListApiKeys;
use crate::api::handler::api_keys::schema::UpdateApiKeyRequest;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::chan::global::GLOBAL;
use crate::models::LeechApiKey;

/// Create new api key
#[swaggapi::post("/apiKeys", tags("Api Keys"))]
pub async fn create_api_key(
    req: Json<CreateApiKeyRequest>,
    SessionUser(user): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let uuid = Uuid::new_v4();
    insert!(&GLOBAL.db, LeechApiKey)
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
#[swaggapi::delete("/apiKeys/{uuid}", tags("Api Keys"))]
pub async fn delete_api_key(
    path: Path<PathUuid>,
    SessionUser(user): SessionUser,
) -> ApiResult<HttpResponse> {
    let deleted = rorm::delete!(&GLOBAL.db, LeechApiKey)
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

/// Retrieve all api keys
#[swaggapi::get("/apiKeys", tags("Api Keys"))]
pub async fn get_api_keys(SessionUser(user): SessionUser) -> ApiResult<Json<ListApiKeys>> {
    let keys = query!(
        &GLOBAL.db,
        (LeechApiKey::F.key, LeechApiKey::F.name, LeechApiKey::F.uuid)
    )
    .condition(LeechApiKey::F.user.equals(user))
    .stream()
    .map_ok(|(key, name, uuid)| FullApiKey { key, name, uuid })
    .try_collect()
    .await?;
    Ok(Json(ListApiKeys { keys }))
}

/// Update an api key by its id
///
/// All parameter are optional, but at least one of them must be specified.
#[swaggapi::put("/apiKeys/{uuid}", tags("Api Keys"))]
pub async fn update_api_key(
    path: Path<PathUuid>,
    req: Json<UpdateApiKeyRequest>,
    SessionUser(user): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();

    if req.name.is_empty() {
        return Err(ApiError::InvalidName);
    }

    let updated = update!(&GLOBAL.db, LeechApiKey)
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
