use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::Utc;
use rorm::{query, update, Database, Model};
use serde::Deserialize;

use crate::api::handler::{ApiError, ApiResult};
use crate::models::User;

#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    username: String,
    password: String,
}

pub(crate) async fn login(
    req: Json<LoginRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    let user = query!(&db, User)
        .transaction(&mut tx)
        .condition(User::F.username.equals(&req.username))
        .optional()
        .await?
        .ok_or(ApiError::LoginFailed)?;

    Argon2::default()
        .verify_password(
            req.password.as_bytes(),
            &PasswordHash::new(&user.password_hash)?,
        )
        .map_err(|e| match e {
            Error::Password => ApiError::LoginFailed,
            _ => ApiError::InvalidHash(e),
        })?;

    update!(&db, User)
        .transaction(&mut tx)
        .condition(User::F.uuid.equals(&user.uuid))
        .set(User::F.last_login, Some(Utc::now().naive_utc()))
        .exec()
        .await?;

    tx.commit().await?;

    session.insert("logged_in", true)?;

    Ok(HttpResponse::Ok().finish())
}
