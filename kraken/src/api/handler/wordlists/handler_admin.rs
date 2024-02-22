use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::wordlists::schema::CreateWordlistRequest;
use crate::api::handler::wordlists::schema::FullWordlist;
use crate::api::handler::wordlists::schema::ListWordlistsAdmin;
use crate::api::handler::wordlists::schema::UpdateWordlistRequest;
use crate::chan::global::GLOBAL;
use crate::models::WordList;

/// Create a new wordlist
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Wordlist got created successfully", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = CreateWordlistRequest,
    security(("api_key" = []))
)]
#[post("/wordlists")]
pub async fn create_wordlist_admin(
    req: Json<CreateWordlistRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let CreateWordlistRequest {
        name,
        description,
        path,
    } = req.into_inner();
    Ok(Json(UuidResponse {
        uuid: WordList::insert(&GLOBAL.db, name, description, path).await?,
    }))
}

/// Get a list of all wordlists including their paths
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "List of all wordlists", body = ListWordlistsAdmin),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    security(("api_key" = []))
)]
#[get("/wordlists")]
pub async fn get_all_wordlists_admin() -> ApiResult<Json<ListWordlistsAdmin>> {
    Ok(Json(ListWordlistsAdmin {
        wordlists: query!(&GLOBAL.db, FullWordlist).all().await?,
    }))
}

/// Update an existing wordlist
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Wordlist got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    request_body = UpdateWordlistRequest,
    security(("api_key" = []))
)]
#[put("/wordlists/{uuid}")]
pub async fn update_wordlist_admin(req: Json<UpdateWordlistRequest>) -> ApiResult<HttpResponse> {
    let UpdateWordlistRequest {
        uuid,
        name,
        description,
        path,
    } = req.into_inner();
    WordList::update(&GLOBAL.db, uuid, name, description, path).await?;
    Ok(HttpResponse::Ok().finish())
}

/// Delete an existing wordlist
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Wordlist got deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/wordlists/{uuid}")]
pub async fn delete_wordlist_admin(path: Path<PathUuid>) -> ApiResult<HttpResponse> {
    let deleted = rorm::delete!(&GLOBAL.db, WordList)
        .condition(WordList::F.uuid.equals(path.uuid))
        .await?;
    if deleted > 0 {
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::InvalidUuid)
    }
}
