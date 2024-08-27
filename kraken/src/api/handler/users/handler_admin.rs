use actix_web::delete;
use actix_web::get;
use actix_web::post;
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
use crate::api::handler::users::schema::CreateUserRequest;
use crate::api::handler::users::schema::FullUser;
use crate::api::handler::users::schema::ListFullUsers;
use crate::chan::global::GLOBAL;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;
use crate::models::User;

/// Create a user
#[utoipa::path(
    tag = "User Admin Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "User got created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateUserRequest,
    security(("api_key" = []))
)]
#[post("/users")]
pub async fn create_user(req: Json<CreateUserRequest>) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();

    let uuid = User::insert_local_user(
        &GLOBAL.db,
        req.username,
        req.display_name,
        req.password,
        req.permission.into_db(),
    )
    .await?;

    Ok(Json(UuidResponse { uuid }))
}

/// Delete a user by its uuid
#[utoipa::path(
    tag = "User Admin Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "User got deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/users/{uuid}")]
pub async fn delete_user(req: Path<PathUuid>) -> ApiResult<HttpResponse> {
    rorm::delete!(&GLOBAL.db, User)
        .condition(User::F.uuid.equals(req.uuid))
        .await?;

    GLOBAL.user_cache.refresh().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Retrieve a user by its uuid
#[utoipa::path(
    tag = "User Admin Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the user", body = FullUser),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/users/{uuid}")]
pub async fn get_user(req: Path<PathUuid>) -> ApiResult<Json<FullUser>> {
    let user_uuid = req.into_inner().uuid;
    Ok(Json(
        GLOBAL
            .user_cache
            .get_full_user(user_uuid)
            .await?
            .ok_or(ApiError::InternalServerError)?,
    ))
}

/// Retrieve all users
#[utoipa::path(
    tag = "User Admin Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all users", body = ListFullUsers),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/users")]
pub async fn get_all_users_admin() -> ApiResult<Json<ListFullUsers>> {
    let users = query!(&GLOBAL.db, User).all().await?;

    Ok(Json(ListFullUsers {
        users: users
            .into_iter()
            .map(|u| FullUser {
                uuid: u.uuid,
                username: u.username,
                display_name: u.display_name,
                permission: FromDb::from_db(u.permission),
                created_at: u.created_at,
                last_login: u.last_login,
            })
            .collect(),
    }))
}
