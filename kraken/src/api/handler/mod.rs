//! All handler for the frontend API are defined here
//!
//! This module also contains common types, such as [ApiError], [PathUuid] and the complete
//! error implementation

use std::sync::TryLockError;

use actix_toolbox::tb_middleware::{actix_session, Session};
use actix_web::body::BoxBody;
use actix_web::web::Query;
use actix_web::HttpResponse;
use log::{debug, error, info, trace, warn};
use rorm::db::Executor;
use rorm::{query, FieldAccess, Model};
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::Serialize_repr;
use thiserror::Error;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use webauthn_rs::prelude::WebauthnError;

use crate::api::handler::attacks::SimpleTcpPortScanResult;
use crate::api::handler::domains::SimpleDomain;
use crate::api::handler::hosts::SimpleHost;
use crate::api::handler::ports::SimplePort;
use crate::api::handler::services::SimpleService;
use crate::models::{Color, User};

pub mod api_keys;
pub mod attacks;
pub mod auth;
pub mod data_export;
pub mod domains;
pub mod global_tags;
pub mod hosts;
pub mod leeches;
pub mod oauth;
pub mod ports;
pub mod services;
pub mod settings;
pub mod users;
pub mod websocket;
pub mod workspace_tags;
pub mod workspaces;

/// Query the current user's model
pub(crate) async fn query_user(db: impl Executor<'_>, session: &Session) -> ApiResult<User> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
    query!(db, User)
        .condition(User::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)
}

/// A common response that contains a single uuid
#[derive(Serialize, ToSchema)]
pub struct UuidResponse {
    pub(crate) uuid: Uuid,
}

/// A path with an UUID
#[derive(Deserialize, IntoParams)]
pub struct PathUuid {
    pub(crate) uuid: Uuid,
}

/// Query parameters for paginated data
#[derive(Deserialize, ToSchema, IntoParams)]
pub struct PageParams {
    /// Number of items to retrieve
    #[schema(example = 50)]
    pub limit: u64,

    /// Position in the whole list to start retrieving from
    #[schema(example = 0)]
    pub offset: u64,
}

/// Response containing paginated data
#[derive(Serialize, ToSchema)]
#[aliases(
    TcpPortScanResultsPage = Page<SimpleTcpPortScanResult>,
    DomainResultsPage = Page<SimpleDomain>,
    HostResultsPage = Page<SimpleHost>,
    ServiceResultsPage = Page<SimpleService>,
    PortResultsPage = Page<SimplePort>
)]
pub struct Page<T> {
    /// The page's items
    pub items: Vec<T>,

    /// The limit this page was retrieved with
    #[schema(example = 50)]
    pub limit: u64,

    /// The offset this page was retrieved with
    #[schema(example = 0)]
    pub offset: u64,

    /// The total number of items this page is a subset of
    pub total: u64,
}

const QUERY_LIMIT_MAX: u64 = 1000;

pub(crate) async fn get_page_params(query: Query<PageParams>) -> Result<(u64, u64), ApiError> {
    let PageParams { limit, offset } = query.into_inner();

    if limit > QUERY_LIMIT_MAX {
        Err(ApiError::InvalidQueryLimit)
    } else {
        Ok((limit, offset))
    }
}

/// The type of a tag
#[derive(Serialize, Deserialize, Copy, Clone, ToSchema, Debug)]
pub enum TagType {
    /// Workspace tag
    Workspace,
    /// Global tag
    Global,
}

/// A simple tag
#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SimpleTag {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) color: Color,
    pub(crate) tag_type: TagType,
}

/// The result type of kraken.
pub type ApiResult<T> = Result<T, ApiError>;

/// This type holds all possible error types that can be returned by the API.
///
/// Numbers between 1000 and 1999 (inclusive) are client errors that can be handled by the client.
/// Numbers between 2000 and 2999 (inclusive) are server errors.
#[derive(Serialize_repr, ToSchema)]
#[repr(u16)]
#[schema(default = 1000, example = 1000)]
pub enum ApiStatusCode {
    /// Login failed
    LoginFailed = 1000,
    /// Not found
    NotFound = 1001,
    /// Invalid content type
    InvalidContentType = 1002,
    /// Invalid json
    InvalidJson = 1003,
    /// Payload overflow
    PayloadOverflow = 1004,

    /// User is unauthenticated
    Unauthenticated = 1005,
    /// User is missing a required second factor
    Missing2fa = 1006,
    /// The user is missing privileges
    MissingPrivileges = 1007,
    /// No security key is available, but it is required
    NoSecurityKeyAvailable = 1008,
    /// User already exists
    UserAlreadyExists = 1009,
    /// Invalid username
    InvalidUsername = 1010,
    /// Invalid address
    InvalidAddress = 1011,
    /// Address already exists
    AddressAlreadyExists = 1012,
    /// Name already exists
    NameAlreadyExists = 1013,
    /// Invalid uuid
    InvalidUuid = 1014,
    /// The given workspace is not valid
    InvalidWorkspace = 1015,
    /// Received an empty json request.
    ///
    /// Mostly happens in update endpoints without supplying an update
    EmptyJson = 1016,
    /// Invalid password
    InvalidPassword = 1017,
    /// Invalid leech
    InvalidLeech = 1018,
    /// Username is already occupied
    UsernameAlreadyExists = 1019,
    /// Invalid name specified
    InvalidName = 1020,
    /// Invalid query limit
    InvalidQueryLimit = 1021,

    /// Internal server error
    InternalServerError = 2000,
    /// An database error occurred
    DatabaseError = 2001,
    /// An error occurred while interacting with the user session
    SessionError = 2002,
    /// Webauthn error
    WebauthnError = 2003,
    /// Dehashed is not available due to missing credentials
    DehashedNotAvailable = 2004,
    /// There's no leech available
    NoLeechAvailable = 2005,
}

/// Representation of an error response
///
/// `status_code` holds the error code, `message` a human readable description of the error
#[derive(Serialize, ToSchema)]
pub struct ApiErrorResponse {
    status_code: ApiStatusCode,
    #[schema(example = "Error message will be here")]
    message: String,
}

impl ApiErrorResponse {
    pub(crate) fn new(status_code: ApiStatusCode, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }
}

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

/// Custom serializer to enable the distinction of missing keys vs null values in JSON requests
///
/// # Example
/// ```rust
/// #[derive(Deserialize)]
///  pub(crate) struct UpdateRequest {
///     name: Option<String>,
///
///     #[serde(default)]
///     #[serde(deserialize_with = "crate::api::handler::de_optional")]
///     description: Option<Option<String>>,
/// }
/// ```
pub(crate) fn de_optional<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(d)?))
}
