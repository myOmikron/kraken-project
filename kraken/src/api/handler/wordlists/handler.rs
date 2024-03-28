use actix_web::web::Json;
use rorm::query;

use crate::api::handler::common::error::ApiResult;
use crate::api::handler::wordlists::schema::ListWordlists;
use crate::api::handler::wordlists::schema::SimpleWordlist;
use crate::chan::global::GLOBAL;

/// Get a list of all wordlist for the user to select from when starting an bruteforce subdomains attack
#[swaggapi::get("/wordlists", tags("Wordlist"))]
pub async fn get_all_wordlists() -> ApiResult<Json<ListWordlists>> {
    Ok(Json(ListWordlists {
        wordlists: query!(&GLOBAL.db, SimpleWordlist).all().await?,
    }))
}
