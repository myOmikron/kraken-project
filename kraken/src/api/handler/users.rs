use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use log::error;
use rand::thread_rng;
use rorm::{query, update, Database, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult};
use crate::chan::{WsManagerChan, WsManagerMessage};
use crate::models::User;
use crate::modules::user::create::create_user_transaction;

/// This struct holds the user information.
///
/// Note that `username` is unique, but as it is changeable,
/// identify the user by its `uuid`
#[derive(Serialize, ToSchema)]
pub(crate) struct UserResponse {
    pub(crate) uuid: Uuid,
    #[schema(example = "user123")]
    pub(crate) username: String,
    #[schema(example = "Anon")]
    pub(crate) display_name: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateUserRequest {
    #[schema(example = "user123")]
    pub(crate) username: String,
    #[schema(example = "Anon")]
    pub(crate) display_name: String,
    #[schema(example = "super-secure-password")]
    pub(crate) password: String,
    #[schema(example = true)]
    pub(crate) admin: bool,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CreateUserResponse {
    pub(crate) uuid: String,
}

/// Create a user
#[utoipa::path(
    tag = "User Admin Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "User got created", body = CreateUserResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateUserRequest,
    security(("api_key" = []))
)]
#[post("/users")]
pub(crate) async fn create_user(
    req: Json<CreateUserRequest>,
    db: Data<Database>,
) -> ApiResult<Json<CreateUserResponse>> {
    let uuid = create_user_transaction(
        req.username.clone(),
        req.display_name.clone(),
        req.password.clone(),
        req.admin,
        &db,
    )
    .await?;

    Ok(Json(CreateUserResponse {
        uuid: uuid.to_string(),
    }))
}

#[derive(Deserialize, IntoParams)]
pub(crate) struct PathUuid {
    pub(crate) uuid: Uuid,
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
pub(crate) async fn delete_user(
    req: Path<PathUuid>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    rorm::delete!(db.as_ref(), User)
        .condition(User::F.uuid.equals(req.uuid.as_ref()))
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetUser {
    pub(crate) uuid: Uuid,
    #[schema(example = "user123")]
    pub(crate) username: String,
    #[schema(example = "Anon")]
    pub(crate) display_name: String,
    #[schema(example = true)]
    pub(crate) admin: bool,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) last_login: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetUserResponse {
    pub(crate) users: Vec<GetUser>,
}

/// Retrieve a user by its uuid
#[utoipa::path(
    tag = "User Admin Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the user", body = GetUser),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/users/{uuid}")]
pub(crate) async fn get_user(req: Path<PathUuid>, db: Data<Database>) -> ApiResult<Json<GetUser>> {
    let user = query!(db.as_ref(), User)
        .condition(User::F.uuid.equals(req.uuid.as_ref()))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUsername)?;

    Ok(Json(GetUser {
        uuid: user.uuid,
        username: user.username,
        display_name: user.display_name,
        admin: user.admin,
        created_at: DateTime::from_naive_utc_and_offset(user.created_at, Utc),
        last_login: user
            .last_login
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
    }))
}

/// Retrieve all users
#[utoipa::path(
    tag = "User Admin Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all users", body = GetUserResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/users")]
pub(crate) async fn get_all_users(db: Data<Database>) -> ApiResult<Json<GetUserResponse>> {
    let users = query!(db.as_ref(), User).all().await?;

    Ok(Json(GetUserResponse {
        users: users
            .into_iter()
            .map(|u| GetUser {
                uuid: u.uuid,
                username: u.username,
                display_name: u.display_name,
                admin: u.admin,
                created_at: DateTime::from_naive_utc_and_offset(u.created_at, Utc),
                last_login: u
                    .last_login
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
            .collect(),
    }))
}

/// Retrieve the own user
#[utoipa::path(
    tag = "User Management",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the own user", body = GetUser),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/users/me")]
pub(crate) async fn get_me(session: Session, db: Data<Database>) -> ApiResult<Json<GetUser>> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let user = query!(db.as_ref(), User)
        .condition(User::F.uuid.equals(uuid.as_ref()))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    Ok(Json(GetUser {
        uuid: user.uuid,
        username: user.username,
        display_name: user.display_name,
        admin: user.admin,
        created_at: DateTime::from_naive_utc_and_offset(user.created_at, Utc),
        last_login: user
            .last_login
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
    }))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct SetPasswordRequest {
    #[schema(example = "super-secure-password")]
    current_password: String,
    #[schema(example = "ultra-secure-password!1!1!")]
    new_password: String,
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
pub(crate) async fn set_password(
    req: Json<SetPasswordRequest>,
    session: Session,
    db: Data<Database>,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let user = query!(&mut tx, User)
        .condition(User::F.uuid.equals(uuid.as_ref()))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    if Argon2::default()
        .verify_password(
            req.current_password.as_bytes(),
            &PasswordHash::try_from(user.password_hash.as_str())?,
        )
        .is_err()
    {
        return Err(ApiError::InvalidPassword);
    }

    let salt = SaltString::generate(&mut thread_rng());
    let password_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)?
        .to_string();

    update!(&mut tx, User)
        .set(User::F.password_hash, &password_hash)
        .exec()
        .await?;

    tx.commit().await?;

    session.purge();

    if let Err(err) = ws_manager_chan
        .send(WsManagerMessage::CloseSocket(uuid))
        .await
    {
        error!("Error sending to websocket manager: {err}");
        return Err(ApiError::InternalServerError);
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UpdateMeRequest {
    #[schema(example = "cyber-user-123")]
    username: Option<String>,
    #[schema(example = "Cyberhacker")]
    display_name: Option<String>,
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
pub(crate) async fn update_me(
    req: Json<UpdateMeRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

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
        .condition(User::F.uuid.equals(uuid.as_ref()))
        .begin_dyn_set()
        .set_if(User::F.username, req.username.as_ref())
        .set_if(User::F.display_name, req.display_name.as_ref())
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
