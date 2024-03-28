use actix_toolbox::tb_middleware::Session;
use actix_web::web::Json;
use actix_web::HttpResponse;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordHasher;
use argon2::PasswordVerifier;
use rand::thread_rng;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::users::schema::FullUser;
use crate::api::handler::users::schema::ListUsers;
use crate::api::handler::users::schema::SetPasswordRequest;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::users::schema::UpdateMeRequest;
use crate::chan::global::GLOBAL;
use crate::models::LocalUser;
use crate::models::User;

/// Retrieve the own user
#[swaggapi::get("/users/me")]
pub async fn get_me(SessionUser(user_uuid): SessionUser) -> ApiResult<Json<FullUser>> {
    Ok(Json(
        GLOBAL
            .user_cache
            .get_full_user(user_uuid)
            .await?
            .ok_or(ApiError::SessionCorrupt)?,
    ))
}

/// Set a new password
#[swaggapi::post("/users/setPassword")]
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
#[swaggapi::put("/users/me")]
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
#[swaggapi::get("/users")]
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
