use actix_web::get;
use actix_web::web::Json;
use rorm::query;

use crate::api::handler::common::error::ApiResult;
use crate::api::handler::wordlists::schema::ListWordlists;
use crate::api::handler::wordlists::schema::SimpleWordlist;
use crate::chan::global::GLOBAL;

/// Get a list of all wordlist for the user to select from when starting an bruteforce subdomains attack
#[utoipa::path(
    tag = "Wordlist",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Matched leeches", body = ListWordlists),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    security(("api_key" = []))
)]
#[get("/wordlists")]
pub async fn get_all_wordlists() -> ApiResult<Json<ListWordlists>> {
    Ok(Json(ListWordlists {
        wordlists: query!(&GLOBAL.db, SimpleWordlist).all().await?,
    }))
}
