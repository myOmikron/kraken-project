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
use crate::models::User;

/// Create a user
#[swaggapi::post("/users", tags("User Admin Management"))]
pub async fn create_user(req: Json<CreateUserRequest>) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();

    let uuid = User::insert_local_user(
        &GLOBAL.db,
        req.username,
        req.display_name,
        req.password,
        req.permission,
    )
    .await?;

    Ok(Json(UuidResponse { uuid }))
}

/// Delete a user by its uuid
#[swaggapi::delete("/users/{uuid}", tags("User Admin Management"))]
pub async fn delete_user(req: Path<PathUuid>) -> ApiResult<HttpResponse> {
    rorm::delete!(&GLOBAL.db, User)
        .condition(User::F.uuid.equals(req.uuid))
        .await?;

    GLOBAL.user_cache.refresh().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Retrieve a user by its uuid
#[swaggapi::get("/users/{uuid}", tags("User Admin Management"))]
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
#[swaggapi::get("/users", tags("User Admin Management"))]
pub async fn get_all_users_admin() -> ApiResult<Json<ListFullUsers>> {
    let users = query!(&GLOBAL.db, User).all().await?;

    Ok(Json(ListFullUsers {
        users: users
            .into_iter()
            .map(|u| FullUser {
                uuid: u.uuid,
                username: u.username,
                display_name: u.display_name,
                permission: u.permission,
                created_at: u.created_at,
                last_login: u.last_login,
            })
            .collect(),
    }))
}
