use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use log::error;
use rand::thread_rng;
use rorm::{query, update, Database, Model};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::Uuid;

use crate::api::handler::{ApiError, ApiResult};
use crate::chan::{WsManagerChan, WsManagerMessage};
use crate::models::User;
use crate::modules::user::create::create_user_transaction;
use crate::modules::user::delete::delete_user_transaction;

#[derive(Deserialize)]
pub(crate) struct CreateUserRequest {
    pub(crate) username: String,
    pub(crate) display_name: String,
    pub(crate) password: String,
    pub(crate) admin: bool,
}

#[derive(Serialize)]
pub(crate) struct CreateUserResponse {
    pub(crate) uuid: String,
}

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

#[derive(Deserialize)]
pub(crate) struct DeleteUserRequest {
    pub(crate) username: String,
}

pub(crate) async fn delete_user(
    req: Path<DeleteUserRequest>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    delete_user_transaction(req.username.clone(), &db).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub(crate) struct GetUserRequest {
    pub(crate) username: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct GetUser {
    pub(crate) uuid: String,
    pub(crate) username: String,
    pub(crate) display_name: String,
    pub(crate) admin: bool,
    pub(crate) created_at: chrono::NaiveDateTime,
    pub(crate) last_login: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize)]
pub(crate) struct GetUserResponse {
    pub(crate) users: Vec<GetUser>,
}

pub(crate) async fn get_user(
    req: Path<GetUserRequest>,
    db: Data<Database>,
) -> ApiResult<Json<GetUserResponse>> {
    let users = if let Some(username) = &req.username {
        query!(&db, User)
            .condition(User::F.username.equals(username))
            .all()
            .await?
    } else {
        query!(&db, User).all().await?
    };

    Ok(Json(GetUserResponse {
        users: users
            .into_iter()
            .map(|u| GetUser {
                uuid: Uuid::from_slice(u.uuid.as_slice()).unwrap().to_string(),
                username: u.username,
                display_name: u.display_name,
                admin: u.admin,
                created_at: u.created_at,
                last_login: u.last_login,
            })
            .collect(),
    }))
}

pub(crate) async fn get_me(session: Session, db: Data<Database>) -> ApiResult<Json<GetUser>> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let user = query!(&db, User)
        .condition(User::F.uuid.equals(&uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    Ok(Json(GetUser {
        uuid: Uuid::from_slice(&user.uuid).unwrap().to_string(),
        username: user.username,
        display_name: user.display_name,
        admin: user.admin,
        created_at: user.created_at,
        last_login: user.last_login,
    }))
}

#[derive(Deserialize)]
pub(crate) struct SetPasswordRequest {
    current_password: String,
    new_password: String,
}

pub(crate) async fn set_password(
    req: Json<SetPasswordRequest>,
    session: Session,
    db: Data<Database>,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let user = query!(&db, User)
        .transaction(&mut tx)
        .condition(User::F.uuid.equals(&uuid))
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

    update!(&db, User)
        .transaction(&mut tx)
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
