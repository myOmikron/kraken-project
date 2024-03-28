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
#[swaggapi::post("/wordlists", tags("Wordlist management"))]
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
#[swaggapi::get("/wordlists", tags("Wordlist management"))]
pub async fn get_all_wordlists_admin() -> ApiResult<Json<ListWordlistsAdmin>> {
    Ok(Json(ListWordlistsAdmin {
        wordlists: query!(&GLOBAL.db, FullWordlist).all().await?,
    }))
}

/// Update an existing wordlist
#[swaggapi::put("/wordlists/{uuid}", tags("Wordlist management"))]
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
#[swaggapi::delete("/wordlists/{uuid}", tags("Wordlist management"))]
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
