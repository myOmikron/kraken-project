use std::sync::TryLockError;

use actix_toolbox::tb_middleware::actix_session;
use actix_web::body::BoxBody;
use actix_web::HttpResponse;
use log::debug;
use log::error;
use log::info;
use log::trace;
use log::warn;
use thiserror::Error;
use webauthn_rs::prelude::WebauthnError;

use crate::api::handler::common::schema::ApiErrorResponse;
use crate::api::handler::common::schema::ApiStatusCode;
use crate::modules::filter::ParseError;

/// The result type of kraken.
pub type ApiResult<T> = Result<T, ApiError>;

/// All available errors that can occur while using the API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Login failed
    #[error("Login failed")]
    LoginFailed,
    /// Not found
    #[error("Not found")]
    NotFound,
    /// Content type error
    #[error("Content type error")]
    InvalidContentType,
    /// serde raised an error
    #[error("Json error: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// Payload overflow
    #[error("Payload overflow: {0}")]
    PayloadOverflow(String),

    /// User is unauthenticated
    #[error("Unauthenticated")]
    Unauthenticated,
    /// User is missing a required second factor
    #[error("2FA is missing")]
    Missing2FA,
    /// The user is missing privileges
    #[error("You are missing privileges")]
    MissingPrivileges,
    /// No security key is available, but it is required
    #[error("No security key is available")]
    NoSecurityKeyAvailable,
    /// User already exists
    #[error("User already exists")]
    UserAlreadyExists,
    /// Invalid username
    #[error("Invalid username")]
    InvalidUsername,
    /// Invalid address
    #[error("Invalid address")]
    InvalidAddress,
    /// Address already exists
    #[error("Address already exists")]
    AddressAlreadyExists,
    /// Name already exists
    #[error("Name already exists")]
    NameAlreadyExists,
    /// Path already exists
    #[error("Name already exists")]
    PathAlreadyExists,
    /// Invalid uuid
    #[error("Invalid uuid")]
    InvalidUuid,
    /// Invalid workspace
    #[error("Invalid workspace")]
    InvalidWorkspace,
    /// Received an empty json request.
    ///
    /// Mostly happens in update endpoints without supplying an update
    #[error("Received an empty json request")]
    EmptyJson,
    /// Invalid password
    #[error("Invalid password supplied")]
    InvalidPassword,
    /// Invalid leech
    #[error("Invalid leech")]
    InvalidLeech,
    /// Username is already occupied
    #[error("Username is already occupied")]
    UsernameAlreadyOccupied,
    /// Invalid name specified
    #[error("Invalid name specified")]
    InvalidName,
    /// Invalid query limit
    #[error("Invalid limit query")]
    InvalidQueryLimit,
    /// Invalid port specified
    #[error("Invalid port")]
    InvalidPort,
    /// Empty Targets
    #[error("Empty Targets")]
    EmptyTargets,
    /// The target is invalid
    #[error("Invalid target")]
    InvalidTarget,
    /// The user was already invited
    #[error("The user is already invited")]
    AlreadyInvited,
    /// The user is already a member
    #[error("The user is already a member")]
    AlreadyMember,
    /// The invitation is invalid
    #[error("Invalid invitation")]
    InvalidInvitation,
    /// The search term was invalid
    #[error("The search term was invalid")]
    InvalidSearch,
    /// The filter string is invalid
    #[error("Failed to parse filter string: {0}")]
    InvalidFilter(#[from] ParseError),

    /// An internal server error occurred
    #[error("Internal server error")]
    InternalServerError,
    /// An database error occurred
    #[error("Database error occurred")]
    DatabaseError(#[from] rorm::Error),
    /// An invalid hash was retrieved from the database
    #[error("Internal server error")]
    InvalidHash(argon2::password_hash::Error),
    /// Could not insert into the session
    #[error("Session error occurred")]
    SessionInsert(#[from] actix_session::SessionInsertError),
    /// Could not retrieve data from the session
    #[error("Session error occurred")]
    SessionGet(#[from] actix_session::SessionGetError),
    /// Could not retrieve expected state from the session
    #[error("Corrupt session")]
    SessionCorrupt,
    /// Webauthn error
    #[error("Webauthn error")]
    Webauthn(#[from] WebauthnError),
    /// Dehashed credentials are not available
    #[error("Dehashed is not available")]
    DehashedNotAvailable,
    /// There's no leech available
    #[error("No leech available")]
    NoLeechAvailable,
    /// Error returned by the [`Payload`](actix_web::web::Payload) stream which is used to
    /// process a request body as stream of [`Bytes`](bytes::Bytes)
    #[error("File upload failed")]
    PayloadError(#[from] actix_web::error::PayloadError),

    /// The uploaded image file is invalid
    #[error("File is an invalid image")]
    InvalidImage,
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            ApiError::LoginFailed => {
                trace!("Login failed");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::LoginFailed,
                    self.to_string(),
                ))
            }
            ApiError::DatabaseError(err) => {
                error!("Database error occurred: {err}");

                HttpResponse::InternalServerError().json(ApiErrorResponse::new(
                    ApiStatusCode::DatabaseError,
                    self.to_string(),
                ))
            }
            ApiError::InvalidHash(err) => {
                error!("Got invalid password hash from db: {err}");

                HttpResponse::InternalServerError().json(ApiErrorResponse::new(
                    ApiStatusCode::InternalServerError,
                    self.to_string(),
                ))
            }
            ApiError::SessionInsert(err) => {
                error!("Session insert error: {err}");

                HttpResponse::InternalServerError().json(ApiErrorResponse::new(
                    ApiStatusCode::SessionError,
                    self.to_string(),
                ))
            }
            ApiError::SessionGet(err) => {
                error!("Session get error: {err}");

                HttpResponse::InternalServerError().json(ApiErrorResponse::new(
                    ApiStatusCode::SessionError,
                    self.to_string(),
                ))
            }
            ApiError::NotFound => HttpResponse::NotFound().json(ApiErrorResponse::new(
                ApiStatusCode::NotFound,
                self.to_string(),
            )),
            ApiError::InvalidContentType => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidContentType,
                self.to_string(),
            )),
            ApiError::InvalidJson(err) => {
                debug!("Received invalid json: {err}");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::InvalidJson,
                    self.to_string(),
                ))
            }
            ApiError::InternalServerError => HttpResponse::InternalServerError().json(
                ApiErrorResponse::new(ApiStatusCode::InternalServerError, self.to_string()),
            ),
            ApiError::PayloadOverflow(err) => {
                debug!("Payload overflow: {err}");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::PayloadOverflow,
                    self.to_string(),
                ))
            }
            ApiError::Unauthenticated => {
                trace!("Unauthenticated");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::Unauthenticated,
                    self.to_string(),
                ))
            }
            ApiError::Missing2FA => {
                trace!("Missing 2fa");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::Missing2fa,
                    self.to_string(),
                ))
            }
            ApiError::SessionCorrupt => {
                warn!("Corrupt session");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::SessionError,
                    self.to_string(),
                ))
            }
            ApiError::MissingPrivileges => {
                trace!("Missing privileges");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::MissingPrivileges,
                    self.to_string(),
                ))
            }
            ApiError::NoSecurityKeyAvailable => {
                debug!("Missing security key");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::NoSecurityKeyAvailable,
                    self.to_string(),
                ))
            }
            ApiError::Webauthn(err) => {
                info!("Webauthn error: {err}");

                HttpResponse::InternalServerError().json(ApiErrorResponse::new(
                    ApiStatusCode::WebauthnError,
                    self.to_string(),
                ))
            }
            ApiError::UserAlreadyExists => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::UserAlreadyExists,
                self.to_string(),
            )),
            ApiError::InvalidUsername => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidUsername,
                self.to_string(),
            )),
            ApiError::InvalidAddress => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidAddress,
                self.to_string(),
            )),
            ApiError::AddressAlreadyExists => HttpResponse::BadRequest().json(
                ApiErrorResponse::new(ApiStatusCode::AddressAlreadyExists, self.to_string()),
            ),
            ApiError::NameAlreadyExists => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::NameAlreadyExists,
                self.to_string(),
            )),
            ApiError::PathAlreadyExists => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::PathAlreadyExists,
                self.to_string(),
            )),
            ApiError::InvalidUuid => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidUuid,
                self.to_string(),
            )),
            ApiError::EmptyJson => {
                trace!("Received an empty json");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::EmptyJson,
                    self.to_string(),
                ))
            }
            ApiError::InvalidPassword => {
                debug!("Invalid password supplied");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::InvalidPassword,
                    self.to_string(),
                ))
            }
            ApiError::InvalidLeech => {
                debug!("Invalid leech id");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::InvalidLeech,
                    self.to_string(),
                ))
            }
            ApiError::UsernameAlreadyOccupied => {
                debug!("Username already occupied");

                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::UsernameAlreadyExists,
                    self.to_string(),
                ))
            }
            ApiError::InvalidName => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidName,
                self.to_string(),
            )),
            ApiError::DehashedNotAvailable => HttpResponse::InternalServerError().json(
                ApiErrorResponse::new(ApiStatusCode::DehashedNotAvailable, self.to_string()),
            ),
            ApiError::NoLeechAvailable => HttpResponse::InternalServerError().json(
                ApiErrorResponse::new(ApiStatusCode::NoLeechAvailable, self.to_string()),
            ),
            ApiError::InvalidQueryLimit => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidQueryLimit,
                self.to_string(),
            )),
            ApiError::InvalidWorkspace => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidWorkspace,
                self.to_string(),
            )),
            ApiError::InvalidPort => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidPort,
                self.to_string(),
            )),
            ApiError::EmptyTargets => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::EmptyTargets,
                self.to_string(),
            )),
            ApiError::AlreadyInvited => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::AlreadyInvited,
                self.to_string(),
            )),
            ApiError::AlreadyMember => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::AlreadyMember,
                self.to_string(),
            )),
            ApiError::InvalidTarget => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidTarget,
                self.to_string(),
            )),
            ApiError::InvalidSearch => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidSearch,
                self.to_string(),
            )),
            ApiError::InvalidInvitation => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidInvitation,
                self.to_string(),
            )),
            ApiError::InvalidFilter(_) => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidFilter,
                self.to_string(),
            )),
            ApiError::PayloadError(err) => {
                debug!("File upload failed: {err}");
                HttpResponse::BadRequest().json(ApiErrorResponse::new(
                    ApiStatusCode::PayloadError,
                    self.to_string(),
                ))
            }
            ApiError::InvalidImage => HttpResponse::BadRequest().json(ApiErrorResponse::new(
                ApiStatusCode::InvalidImage,
                self.to_string(),
            )),
        }
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(value: argon2::password_hash::Error) -> Self {
        ApiError::InvalidHash(value)
    }
}

impl<T> From<TryLockError<T>> for ApiError {
    fn from(_: TryLockError<T>) -> Self {
        Self::InternalServerError
    }
}
