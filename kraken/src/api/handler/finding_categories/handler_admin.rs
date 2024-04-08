use actix_web::delete;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use rorm::and;
use rorm::insert;
use rorm::prelude::*;
use rorm::query;
use rorm::update;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::finding_categories::schema::CreateFindingCategoryRequest;
use crate::api::handler::finding_categories::schema::UpdateFindingCategoryRequest;
use crate::chan::global::GLOBAL;
use crate::models::FindingCategory;

/// Create a finding category.
///
/// This action requires admin privileges.
#[utoipa::path(
    tag = "Finding Categories",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Finding category was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateFindingCategoryRequest,
    security(("api_key" = []))
)]
#[post("/findingCategories")]
pub async fn create_finding_category(
    Json(request): Json<CreateFindingCategoryRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let CreateFindingCategoryRequest { name } = request;

    let mut tx = GLOBAL.db.start_transaction().await?;
    if query!(&mut tx, (FindingCategory::F.uuid,))
        .condition(FindingCategory::F.name.equals(&name))
        .optional()
        .await?
        .is_some()
    {
        return Err(ApiError::NameAlreadyExists);
    }

    let uuid = Uuid::new_v4();
    insert!(&mut tx, FindingCategory)
        .return_nothing()
        .single(&FindingCategory { uuid, name })
        .await?;

    tx.commit().await?;

    Ok(Json(UuidResponse { uuid }))
}

/// Update a finding category
///
/// One of the options must be set
///
/// Requires admin privileges.
#[utoipa::path(
    tag = "Finding Categories",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Finding category was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = UpdateFindingCategoryRequest,
    security(("api_key" = []))
)]
#[put("/findingCategories/{uuid}")]
pub async fn update_finding_category(
    path: Path<PathUuid>,
    Json(request): Json<UpdateFindingCategoryRequest>,
) -> ApiResult<HttpResponse> {
    let PathUuid { uuid } = path.into_inner();

    if matches!(request, UpdateFindingCategoryRequest { name: None }) {
        return Err(ApiError::EmptyJson);
    }

    let mut tx = GLOBAL.db.start_transaction().await?;

    if let Some(name) = request.name.as_deref() {
        if query!(&mut tx, (FindingCategory::F.uuid,))
            .condition(and![
                FindingCategory::F.name.equals(name),
                FindingCategory::F.uuid.not_equals(uuid)
            ])
            .optional()
            .await?
            .is_some()
        {
            return Err(ApiError::NameAlreadyExists);
        }
    }

    if let Ok(update) = update!(&mut tx, FindingCategory)
        .begin_dyn_set()
        .set_if(FindingCategory::F.name, request.name)
        .finish_dyn_set()
    {
        update.await?;
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Delete a finding category
///
/// Requires admin privileges.
#[utoipa::path(
    tag = "Finding Categories",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Finding category was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/findingCategories/{uuid}")]
pub async fn delete_finding_category(path: Path<PathUuid>) -> ApiResult<HttpResponse> {
    let PathUuid { uuid } = path.into_inner();

    rorm::delete!(&GLOBAL.db, FindingCategory)
        .condition(FindingCategory::F.uuid.equals(uuid))
        .await?;

    Ok(HttpResponse::Ok().finish())
}
