use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::Utc;
use log::debug;
use rorm::internal::field::foreign_model::ForeignModelByField;
use rorm::{insert, query, update, Database, Model};
use serde::Deserialize;
use webauthn_rs::prelude::{
    CreationChallengeResponse, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
    PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse, Uuid,
};
use webauthn_rs::Webauthn;

use crate::api::handler::{ApiError, ApiResult};
use crate::models::{User, UserKey, UserKeyInsert};

pub(crate) async fn test() -> HttpResponse {
    HttpResponse::Ok().finish()
}

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

pub(crate) async fn logout(session: Session) -> HttpResponse {
    session.purge();

    HttpResponse::Ok().finish()
}

pub(crate) async fn start_auth(
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<Json<RequestChallengeResponse>> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }

    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    session.remove("auth_state");

    let keys = query!(&db, UserKey)
        .condition(UserKey::F.user.equals(&uuid))
        .all()
        .await?;

    if keys.is_empty() {
        return Err(ApiError::NoSecurityKeyAvailable);
    }

    let allowed_keys: Vec<Passkey> = keys
        .into_iter()
        .map(|k| serde_json::from_slice(&k.key).unwrap())
        .collect();

    let (rcr, auth_state) = webauthn.start_passkey_authentication(&allowed_keys)?;

    session.insert("auth_state", (uuid, auth_state))?;

    Ok(Json(rcr))
}

pub(crate) async fn finish_auth(
    auth: Json<PublicKeyCredential>,
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<HttpResponse> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }

    let (uuid, auth_state): (Vec<u8>, PasskeyAuthentication) = session
        .get("auth_state")?
        .ok_or(ApiError::Unauthenticated)?;

    session.remove("auth_state");

    webauthn.finish_passkey_authentication(&auth, &auth_state)?;

    update!(&db, User)
        .condition(User::F.uuid.equals(&uuid))
        .set(User::F.last_login, Utc::now().naive_utc())
        .exec()
        .await?;

    session.insert("2fa", true)?;

    debug!("Challenge response successful");

    Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn start_register(
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<Json<CreationChallengeResponse>> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut user = query!(&db, User)
        .condition(User::F.uuid.equals(&uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    User::F.user_keys.populate(&db, &mut user).await?;

    if !user.user_keys.cached.unwrap().is_empty()
        && !session.get("2fa")?.ok_or(ApiError::Missing2FA)?
    {
        return Err(ApiError::Missing2FA);
    }

    session.remove("reg_state");

    let excluded_keys: Vec<CredentialID> = query!(&db, UserKey)
        .condition(UserKey::F.user.equals(&uuid))
        .all()
        .await?
        .into_iter()
        .map(|k| {
            serde_json::from_slice::<Passkey>(&k.key)
                .unwrap()
                .cred_id()
                .clone()
        })
        .collect();

    let (ccr, reg_state) = webauthn.start_passkey_registration(
        Uuid::from_slice(&uuid).unwrap(),
        &user.username,
        &user.display_name,
        Some(excluded_keys),
    )?;

    session.insert("reg_state", (uuid, reg_state))?;

    debug!("Registered key");

    Ok(Json(ccr))
}

#[derive(Deserialize)]
pub(crate) struct FinishRegisterRequest {
    #[serde(flatten)]
    register_pk_credential: RegisterPublicKeyCredential,
    name: String,
}

pub(crate) async fn finish_register(
    req: Json<FinishRegisterRequest>,
    db: Data<Database>,
    session: Session,
    webauthn: Data<Webauthn>,
) -> ApiResult<HttpResponse> {
    if !session.get("logged_in")?.ok_or(ApiError::Unauthenticated)? {
        return Err(ApiError::Unauthenticated);
    }

    let (uuid, reg_state): (Vec<u8>, PasskeyRegistration) =
        session.get("reg_state")?.ok_or(ApiError::SessionCorrupt)?;

    session.remove("reg_state");

    let passkey = webauthn.finish_passkey_registration(&req.register_pk_credential, &reg_state)?;

    insert!(&db, UserKeyInsert)
        .single(&UserKeyInsert {
            user: ForeignModelByField::Key(uuid),
            key: serde_json::to_vec(&passkey).unwrap(),
            name: req.name.clone(),
        })
        .await?;

    Ok(HttpResponse::Ok().finish())
}
