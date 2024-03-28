use actix_toolbox::tb_middleware::Session;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use argon2::password_hash::Error;
use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordVerifier;
use chrono::Utc;
use log::debug;
use rorm::prelude::BackRef;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use swaggapi::utils::SchemalessJson;
use uuid::Uuid;
use webauthn_rs::prelude::CreationChallengeResponse;
use webauthn_rs::prelude::CredentialID;
use webauthn_rs::prelude::Passkey;
use webauthn_rs::prelude::PasskeyAuthentication;
use webauthn_rs::prelude::PasskeyRegistration;
use webauthn_rs::prelude::PublicKeyCredential;
use webauthn_rs::prelude::RequestChallengeResponse;
use webauthn_rs::Webauthn;

use crate::api::extractors::SessionUser;
use crate::api::handler::auth::schema::FinishRegisterRequest;
use crate::api::handler::auth::schema::LoginRequest;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::middleware::AuthenticationRequired;
use crate::chan::global::GLOBAL;
use crate::models::LocalUser;
use crate::models::LocalUserKey;
use crate::models::User;

/// Test the current login state
///
/// You can use this endpoint to test the current login state of your client.
///
/// If logged in, a 200 without a body is returned.
#[swaggapi::get("/test", tags("Authentication"))] // TODO , wrap = "AuthenticationRequired"
pub async fn test() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// Login to kraken
#[swaggapi::post("/login", tags("Authentication"))]
pub async fn login(req: Json<LoginRequest>, session: Session) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

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
#[swaggapi::get("/logout", tags("Authentication"))]
pub async fn logout(
    session: Session,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    session.purge();

    GLOBAL.ws.close_all(user_uuid).await;

    Ok(HttpResponse::Ok().finish())
}

/// Starts the authentication with a security key
///
/// Use the `login` endpoint before calling this one.
///
/// Proceed with `finishAuth`.
#[swaggapi::post("/startAuth", tags("Authentication"))]
pub async fn start_auth(
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<SchemalessJson<RequestChallengeResponse>> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }

    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    session.remove("auth_state");

    let keys = query!(&GLOBAL.db, LocalUserKey)
        .condition(LocalUserKey::F.user.equals(uuid))
        .all()
        .await?;

    if keys.is_empty() {
        return Err(ApiError::NoSecurityKeyAvailable);
    }

    let allowed_keys: Vec<Passkey> = keys.into_iter().map(|k| k.key.0).collect();

    let (rcr, auth_state) = webauthn.start_passkey_authentication(&allowed_keys)?;

    session.insert("auth_state", (uuid, auth_state))?;

    Ok(SchemalessJson(rcr))
}

/// Finishes the authentication with a security key
///
/// Use `startAuth` to retrieve the challenge response data.
#[swaggapi::post("/finishAuth", tags("Authentication"))]
pub async fn finish_auth(
    auth: SchemalessJson<PublicKeyCredential>,
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

    webauthn.finish_passkey_authentication(&auth.0, &auth_state)?;

    update!(&GLOBAL.db, User)
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
#[swaggapi::post("/startRegister", tags("Authentication"))]
pub async fn start_register(
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<SchemalessJson<CreationChallengeResponse>> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = GLOBAL.db.start_transaction().await?;

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

    // Populate fills cached, so it will be always Some()
    #[allow(clippy::unwrap_used)]
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

    Ok(SchemalessJson(ccr))
}

/// Finish the registration of a security key
///
/// Use `startRegister` to retrieve the challenge response data.
#[swaggapi::post("/finishRegister", tags("Authentication"))]
pub async fn finish_register(
    req: Json<FinishRegisterRequest>,
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

    LocalUserKey::insert(&GLOBAL.db, uuid, req.name, passkey).await?;

    Ok(HttpResponse::Ok().finish())
}
