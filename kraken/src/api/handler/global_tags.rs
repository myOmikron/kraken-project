//! The handlers for global tags are defined in this module

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use rorm::{query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult, Color, PathUuid, UuidResponse};
use crate::models::GlobalTag;

/// The request to create a global tag
#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateGlobalTagRequest {
    /// Name of the tag
    pub name: String,
    /// Color of a tag
    pub color: Color,
}

/// Create a global tag.
///
/// This action requires admin privileges.
#[utoipa::path(
    tag = "Global Tags",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Global tag was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateGlobalTagRequest,
    security(("api_key" = []))
)]
#[post("/globalTags")]
pub async fn create_global_tag(
    req: Json<CreateGlobalTagRequest>,
    db: Data<Database>,
) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();

    let uuid = GlobalTag::insert(db.as_ref(), req.name, req.color).await?;

    Ok(Json(UuidResponse { uuid }))
}

/// The full representation of a full
#[derive(Serialize, ToSchema, Debug)]
pub struct FullGlobalTag {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) color: Color,
}

/// The response to a request to retrieve all global tags
#[derive(Serialize, ToSchema, Debug)]
pub struct GetGlobalTagsResponse {
    pub(crate) global_tags: Vec<FullGlobalTag>,
}

/// Retrieve all global tags
#[utoipa::path(
    tag = "Global Tags",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all global tags", body = GetGlobalTagsResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/globalTags")]
pub async fn get_all_global_tags(db: Data<Database>) -> ApiResult<Json<GetGlobalTagsResponse>> {
    let global_tags = query!(db.as_ref(), GlobalTag).all().await?;

    Ok(Json(GetGlobalTagsResponse {
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

/// The request to update a global tag
#[derive(Deserialize, ToSchema)]
pub struct UpdateGlobalTag {
    name: Option<String>,
    color: Option<Color>,
}

/// Update a global tag
///
/// One of the options must be set
///
/// Requires admin privileges.
#[utoipa::path(
tag = "Global Tags",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Global tag was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = UpdateGlobalTag,
    security(("api_key" = []))
)]
#[put("/globalTags/{uuid}")]
pub async fn update_global_tag(
    req: Json<UpdateGlobalTag>,
    path: Path<PathUuid>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let path = path.into_inner();
    let req = req.into_inner();

    let mut tx = db.start_transaction().await?;

    query!(&mut tx, (GlobalTag::F.uuid,))
        .condition(GlobalTag::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(name) = &req.name {
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }
    }

    update!(&mut tx, GlobalTag)
        .begin_dyn_set()
        .set_if(GlobalTag::F.name, req.name)
        .set_if(GlobalTag::F.color, req.color.map(|x| x.into()))
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Delete a global tag
///
/// Requires admin privileges.
#[utoipa::path(
    tag = "Global Tags",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Global tag was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/globalTags/{uuid}")]
pub async fn delete_global_tag(
    path: Path<PathUuid>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let path = path.into_inner();
    let mut tx = db.start_transaction().await?;

    query!(&mut tx, (GlobalTag::F.uuid,))
        .condition(GlobalTag::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, GlobalTag)
        .condition(GlobalTag::F.uuid.equals(path.uuid))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
