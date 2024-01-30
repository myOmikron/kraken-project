use actix_toolbox::tb_middleware::Session;
use actix_web::web::Json;
use actix_web::{get, post, put, HttpResponse};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::thread_rng;
use rorm::{query, update, FieldAccess, Model};

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::{ApiError, ApiResult};
use crate::api::handler::users::schema::{
    FullUser, ListUsers, SetPasswordRequest, SimpleUser, UpdateMeRequest,
};
use crate::chan::global::GLOBAL;
use crate::models::{LocalUser, User};

/// Retrieve the own user
#[utoipa::path(
    tag = "User Management",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the own user", body = FullUser),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/users/me")]
pub async fn get_me(SessionUser(user_uuid): SessionUser) -> ApiResult<Json<FullUser>> {
    let user = query!(&GLOBAL.db, User)
        .condition(User::F.uuid.equals(user_uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    Ok(Json(FullUser {
        uuid: user.uuid,
        username: user.username,
        display_name: user.display_name,
        permission: user.permission,
        created_at: user.created_at,
        last_login: user.last_login,
    }))
}

/// Set a new password
#[utoipa::path(
    tag = "User Management",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Password was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = SetPasswordRequest,
    security(("api_key" = []))
)]
#[post("/users/setPassword")]
pub async fn set_password(
    req: Json<SetPasswordRequest>,
    SessionUser(user_uuid): SessionUser,
    session: Session,
) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    // TODO: Other error case
    let (password_hash, local_user_uuid) =
        query!(&mut tx, (LocalUser::F.password_hash, LocalUser::F.uuid))
            .condition(LocalUser::F.user.uuid.equals(user_uuid))
            .optional()
            .await?
            .ok_or(ApiError::SessionCorrupt)?;

    if Argon2::default()
        .verify_password(
            req.current_password.as_bytes(),
            &PasswordHash::try_from(password_hash.as_str())?,
        )
        .is_err()
    {
        return Err(ApiError::InvalidPassword);
    }

    let salt = SaltString::generate(&mut thread_rng());
    let password_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)?
        .to_string();

    update!(&mut tx, LocalUser)
        .set(LocalUser::F.password_hash, password_hash)
        .condition(LocalUser::F.uuid.equals(local_user_uuid))
        .exec()
        .await?;

    tx.commit().await?;

    session.purge();

    GLOBAL.ws.close_all(user_uuid).await;

    Ok(HttpResponse::Ok().finish())
}

/// Updates the own user
///
/// All parameters are optional, but at least one of them must be supplied.
#[utoipa::path(
    tag = "User Management",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Changes were applied"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateMeRequest,
    security(("api_key" = []))
)]
#[put("/users/me")]
pub async fn update_me(
    req: Json<UpdateMeRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if let Some(username) = &req.username {
        if query!(&mut tx, (User::F.uuid,))
            .condition(User::F.username.equals(username))
            .optional()
            .await?
            .is_some()
        {
            return Err(ApiError::UsernameAlreadyOccupied);
        }
    }

    update!(&mut tx, User)
        .condition(User::F.uuid.equals(user_uuid))
        .begin_dyn_set()
        .set_if(User::F.username, req.username)
        .set_if(User::F.display_name, req.display_name)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .await?;

    tx.commit().await?;

    GLOBAL.user_cache.refresh().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Request all users
///
/// This may be used to create invitations for workspaces
#[utoipa::path(
    tag = "User Management",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Simple representation of all users", body = ListUsers),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/users")]
pub async fn get_all_users() -> ApiResult<Json<ListUsers>> {
    let users = query!(
        &GLOBAL.db,
        (User::F.uuid, User::F.username, User::F.display_name)
    )
    .all()
    .await?;

    Ok(Json(ListUsers {
        users: users
            .into_iter()
            .map(|(uuid, username, display_name)| SimpleUser {
                uuid,
                username,
                display_name,
            })
            .collect(),
    }))
}
