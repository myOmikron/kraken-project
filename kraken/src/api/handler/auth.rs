//! Everything regarding authentication, besides oauth

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpResponse};
use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::Utc;
use log::{debug, error};
use rorm::prelude::{BackRef, ForeignModelByField};
use rorm::{query, update, Database, FieldAccess, Model};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use webauthn_rs::prelude::{
    CreationChallengeResponse, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
    PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse,
};
use webauthn_rs::Webauthn;

use crate::api::handler::{ApiError, ApiResult};
use crate::api::middleware::AuthenticationRequired;
use crate::chan::{WsManagerChan, WsManagerMessage};
use crate::models::{LocalUser, LocalUserKey, User};

/// Test the current login state
///
/// You can use this endpoint to test the current login state of your client.
///
/// If logged in, a 200 without a body is returned.
#[utoipa::path(
    tag = "Authentication",
    context_path = "/api/v1/auth",
    responses(
        (status = 200, description = "Logged in"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    )
)]
#[get("/test", wrap = "AuthenticationRequired")]
pub async fn test() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// The request to login
#[derive(ToSchema, Deserialize)]
pub struct LoginRequest {
    #[schema(example = "user123")]
    username: String,
    #[schema(example = "super-secure-password")]
    password: String,
}

/// Login to kraken
#[utoipa::path(
    tag = "Authentication",
    context_path = "/api/v1/auth",
    responses(
        (status = 200, description = "Login successful"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = LoginRequest,
)]
#[post("/login")]
pub async fn login(
    req: Json<LoginRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    let (user, password_hash) = query!(
        &mut tx,
        (LocalUser::F.user as User, LocalUser::F.password_hash)
    )
    .condition(LocalUser::F.user.username.equals(&req.username))
    .optional()
    .await?
    .ok_or(ApiError::LoginFailed)?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &PasswordHash::new(&password_hash)?)
        .map_err(|e| match e {
            Error::Password => ApiError::LoginFailed,
            _ => ApiError::InvalidHash(e),
        })?;

    update!(&mut tx, User)
        .condition(User::F.uuid.equals(user.uuid))
        .set(User::F.last_login, Some(Utc::now()))
        .exec()
        .await?;

    tx.commit().await?;

    session.insert("uuid", user.uuid)?;
    session.insert("logged_in", true)?;

    Ok(HttpResponse::Ok().finish())
}

/// Log out of this session
///
/// Logs a logged-in user out of his session.
#[utoipa::path(
    tag = "Authentication",
    context_path = "/api/v1/auth",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
)]
#[get("/logout")]
pub async fn logout(
    session: Session,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
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

/// Starts the authentication with a security key
///
/// Use the `login` endpoint before calling this one.
///
/// Proceed with `finishAuth`.
#[utoipa::path(
    tag = "Authentication",
    context_path = "/api/v1/auth",
    responses(
        (status = 200, description = "2FA Authentication started", body = inline(Object)),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
)]
#[post("/startAuth")]
pub async fn start_auth(
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<Json<RequestChallengeResponse>> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }

    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    session.remove("auth_state");

    let keys = query!(db.as_ref(), LocalUserKey)
        .condition(LocalUserKey::F.user.equals(uuid))
        .all()
        .await?;

    if keys.is_empty() {
        return Err(ApiError::NoSecurityKeyAvailable);
    }

    let allowed_keys: Vec<Passkey> = keys.into_iter().map(|k| k.key.0).collect();

    let (rcr, auth_state) = webauthn.start_passkey_authentication(&allowed_keys)?;

    session.insert("auth_state", (uuid, auth_state))?;

    Ok(Json(rcr))
}

/// Finishes the authentication with a security key
///
/// Use `startAuth` to retrieve the challenge response data.  
#[utoipa::path(
    tag = "Authentication",
    context_path = "/api/v1/auth",
    responses(
        (status = 200, description = "2FA Authentication finished"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = inline(Object)
)]
#[post("/finishAuth")]
pub async fn finish_auth(
    auth: Json<PublicKeyCredential>,
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<HttpResponse> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }

    let (uuid, auth_state): (Uuid, PasskeyAuthentication) = session
        .get("auth_state")?
        .ok_or(ApiError::Unauthenticated)?;

    session.remove("auth_state");

    webauthn.finish_passkey_authentication(&auth, &auth_state)?;

    update!(db.as_ref(), User)
        .condition(User::F.uuid.equals(uuid))
        .set(User::F.last_login, Some(Utc::now()))
        .exec()
        .await?;

    session.insert("2fa", true)?;

    debug!("Challenge response successful");

    Ok(HttpResponse::Ok().finish())
}

/// Start the registration of a security key
///
/// Proceed to the `finishRegister` endpoint.
#[utoipa::path(
    tag = "Authentication",
    context_path = "/api/v1/auth",
    responses(
        (status = 200, description = "2FA Key registration started", body = inline(Object)),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
)]
#[post("/startRegister")]
pub async fn start_register(
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<Json<CreationChallengeResponse>> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    // TODO: Make other error for this
    let (user, local_user_uuid, password_hash) = query!(
        &mut tx,
        (
            LocalUser::F.user as User,
            LocalUser::F.uuid,
            LocalUser::F.password_hash
        )
    )
    .condition(LocalUser::F.user.equals(uuid))
    .optional()
    .await?
    .ok_or(ApiError::SessionCorrupt)?;

    let mut local_user = LocalUser {
        uuid: local_user_uuid,
        user: ForeignModelByField::Key(user.uuid),
        password_hash,
        user_keys: BackRef { cached: None },
    };

    LocalUser::F
        .user_keys
        .populate(&mut tx, &mut local_user)
        .await?;

    if !local_user.user_keys.cached.unwrap().is_empty()
        && !session.get("2fa")?.ok_or(ApiError::Missing2FA)?
    {
        return Err(ApiError::Missing2FA);
    }

    session.remove("reg_state");

    let excluded_keys: Vec<CredentialID> = query!(&mut tx, LocalUserKey)
        .condition(LocalUserKey::F.user.equals(local_user_uuid))
        .all()
        .await?
        .into_iter()
        .map(|k| k.key.cred_id().clone())
        .collect();

    let (ccr, reg_state) = webauthn.start_passkey_registration(
        uuid,
        &user.username,
        &user.display_name,
        Some(excluded_keys),
    )?;

    session.insert("reg_state", (uuid, reg_state))?;

    debug!("Registered key");

    Ok(Json(ccr))
}

/// The request to finish the registration of a security key
#[derive(Deserialize, ToSchema)]
pub struct FinishRegisterRequest {
    // TODO: provide a example json for this request
    #[serde(flatten)]
    #[schema(example = json!({}), value_type = Object)]
    register_pk_credential: RegisterPublicKeyCredential,
    #[schema(example = "my-security-key-01")]
    name: String,
}

/// Finish the registration of a security key
///
/// Use `startRegister` to retrieve the challenge response data.
#[utoipa::path(
    tag = "Authentication",
    context_path = "/api/v1/auth",
    responses(
        (status = 200, description = "2FA Key registration finished"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = FinishRegisterRequest
)]
#[post("/finishRegister")]
pub async fn finish_register(
    req: Json<FinishRegisterRequest>,
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<HttpResponse> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }

    let req = req.into_inner();

    let (uuid, reg_state): (Uuid, PasskeyRegistration) =
        session.get("reg_state")?.ok_or(ApiError::SessionCorrupt)?;

    session.remove("reg_state");

    let passkey = webauthn.finish_passkey_registration(&req.register_pk_credential, &reg_state)?;

    LocalUserKey::insert(db.as_ref(), uuid, req.name, passkey).await?;

    Ok(HttpResponse::Ok().finish())
}
