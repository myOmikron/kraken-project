use actix_web::web::Json;
use rorm::query;

use crate::api::handler::common::error::ApiResult;
use crate::api::handler::global_tags::schema::FullGlobalTag;
use crate::api::handler::global_tags::schema::ListGlobalTags;
use crate::chan::global::GLOBAL;
use crate::models::GlobalTag;

/// Retrieve all global tags
#[swaggapi::get("/globalTags", tags("Global Tags"))]
pub async fn get_all_global_tags() -> ApiResult<Json<ListGlobalTags>> {
    let global_tags = query!(&GLOBAL.db, GlobalTag).all().await?;

    Ok(Json(ListGlobalTags {
        global_tags: global_tags
            .into_iter()
            .map(|x| FullGlobalTag {
                uuid: x.uuid,
                name: x.name,
                color: x.color.into(),
            })
            .collect(),
    }))
}
