use actix_web::get;
use actix_web::web::Json;
use futures::TryStreamExt;
use rorm::query;

use crate::api::handler::common::error::ApiResult;
use crate::api::handler::finding_categories::schema::ListFindingCategories;
use crate::api::handler::finding_categories::schema::SimpleFindingCategory;
use crate::chan::global::GLOBAL;
use crate::models::convert::FromDb;
use crate::models::FindingCategory;

/// Retrieve all finding categories
#[utoipa::path(
    tag = "Finding Categories",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all finding categories", body = ListFindingCategories),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/findingCategories")]
pub async fn get_all_finding_categories() -> ApiResult<Json<ListFindingCategories>> {
    Ok(Json(ListFindingCategories {
        categories: query!(&GLOBAL.db, FindingCategory)
            .stream()
            .map_ok(|c| SimpleFindingCategory {
                uuid: c.uuid,
                name: c.name,
                color: FromDb::from_db(c.color),
            })
            .try_collect()
            .await?,
    }))
}
