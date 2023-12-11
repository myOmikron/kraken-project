//! All handler for the frontend API are defined here
//!
//! This module also contains common types, such as [ApiError], [PathUuid] and the complete
//! error implementation

use std::collections::HashMap;
use std::sync::TryLockError;

use actix_toolbox::tb_middleware::actix_session;
use actix_web::body::BoxBody;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use log::{debug, error, info, trace, warn};
use rorm::conditions::DynamicCollection;
use rorm::db::transaction::Transaction;
use rorm::{and, query, FieldAccess, Model};
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::Serialize_repr;
use thiserror::Error;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use webauthn_rs::prelude::WebauthnError;

use crate::api::handler::attack_results::{
    FullQueryCertificateTransparencyResult, FullServiceDetectionResult,
    SimpleBruteforceSubdomainsResult, SimpleDnsResolutionResult, SimpleHostAliveResult,
    SimpleQueryUnhashedResult, SimpleTcpPortScanResult,
};
use crate::api::handler::domains::FullDomain;
use crate::api::handler::hosts::FullHost;
use crate::api::handler::ports::FullPort;
use crate::api::handler::services::FullService;
use crate::api::handler::workspaces::{SearchEntry, SearchResultEntry};
use crate::models::{AggregationSource, AggregationTable, Color, SourceType};
use crate::modules::filter::ParseError;

pub mod api_keys;
pub mod attack_results;
pub mod attacks;
pub mod auth;
pub mod data_export;
pub mod domains;
pub mod global_tags;
pub mod hosts;
pub mod leeches;
pub mod oauth;
pub mod oauth_applications;
pub mod oauth_decisions;
pub mod ports;
pub mod services;
pub mod settings;
pub mod users;
pub mod websocket;
pub mod wordlists;
pub mod workspace_invitations;
pub mod workspace_tags;
pub mod workspaces;

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
#[derive(Copy, Clone, Deserialize, IntoParams, ToSchema)]
pub struct PageParams {
    /// Number of items to retrieve
    #[param(example = 50, minimum = 1)]
    pub limit: u64,

    /// Position in the whole list to start retrieving from
    #[param(example = 0)]
    pub offset: u64,
}

pub use utoipa_fix::Page;
pub(crate) use utoipa_fix::{
    BruteforceSubdomainsResultsPage, DnsResolutionResultsPage, DomainResultsPage,
    HostAliveResultsPage, HostResultsPage, PortResultsPage,
    QueryCertificateTransparencyResultsPage, QueryUnhashedResultsPage, SearchResultPage,
    SearchesResultPage, ServiceDetectionResultsPage, ServiceResultsPage, TcpPortScanResultsPage,
};
mod utoipa_fix {
    use serde::Serialize;
    use utoipa::ToSchema;

    use super::*;

    /// Response containing paginated data
    #[derive(Serialize, ToSchema)]
    #[aliases(
        DomainResultsPage = Page<FullDomain>,
        HostResultsPage = Page<FullHost>,
        ServiceResultsPage = Page<FullService>,
        PortResultsPage = Page<FullPort>,
        BruteforceSubdomainsResultsPage = Page<SimpleBruteforceSubdomainsResult>,
        TcpPortScanResultsPage = Page<SimpleTcpPortScanResult>,
        QueryCertificateTransparencyResultsPage = Page<FullQueryCertificateTransparencyResult>,
        QueryUnhashedResultsPage = Page<SimpleQueryUnhashedResult>,
        HostAliveResultsPage = Page<SimpleHostAliveResult>,
        ServiceDetectionResultsPage = Page<FullServiceDetectionResult>,
        DnsResolutionResultsPage = Page<SimpleDnsResolutionResult>,
        SearchResultPage = Page<SearchResultEntry>,
        SearchesResultPage = Page<SearchEntry>,
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
}

const QUERY_LIMIT_MAX: u64 = 1000;

pub(crate) async fn get_page_params(query: PageParams) -> Result<(u64, u64), ApiError> {
    let PageParams { limit, offset } = query;
    if limit > QUERY_LIMIT_MAX || limit == 0 {
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
    /// Invalid port
    InvalidPort = 1022,
    /// Empty targets
    EmptyTargets = 1023,
    /// Path already exists
    PathAlreadyExists = 1024,
    /// The target is invalid
    InvalidTarget = 1025,
    /// The user was already invited
    AlreadyInvited = 1026,
    /// The user is already a member
    AlreadyMember = 1027,
    /// The invitation is invalid
    InvalidInvitation = 1028,
    /// The search term was invalid
    InvalidSearch = 1029,
    /// The filter string is invalid
    InvalidFilter = 1030,

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

/// Query all tags related to a list of aggregated results
///
/// Create a HashMap and provide it has $map
/// Provide the Transaction as $tx
/// Provide the Query for the WorkspaceTags that provides a tuple of the WorkspaceTag and the Item struct
/// Provide the field the condition should be built on for querying WorkspaceTags
/// Provide the Query for the GlobalTags that provides a tuple of the GlobalTag and the Item struct
/// Provide the field the condition should be built on for querying GlobalTags
/// Provide an iterator over the list of Item Uuids as $items
#[macro_export]
macro_rules! query_tags {
    ($map: ident, $tx: ident, $workspace_query: tt, $workspace_cond: expr, $global_query: tt, $global_cond: expr, $items: expr) => {{
        {
            let workspace_conditions: Vec<_> = $items.map(|x| $workspace_cond.equals(x)).collect();

            if !workspace_conditions.is_empty() {
                let mut workspace_tag_stream = query!(&mut $tx, $workspace_query)
                    .condition(DynamicCollection::or(workspace_conditions))
                    .stream();

                while let Some((tag, item)) = workspace_tag_stream.try_next().await? {
                    $map.entry(*item.key()).or_insert(vec![]).push(SimpleTag {
                        uuid: tag.uuid,
                        name: tag.name,
                        tag_type: TagType::Workspace,
                        color: tag.color.into(),
                    });
                }
            }
        }

        {
            let global_conditions: Vec<_> = $items.map(|x| $global_cond.equals(x)).collect();

            if !global_conditions.is_empty() {
                let mut global_tag_stream = query!(&mut $tx, $global_query)
                    .condition(DynamicCollection::or(global_conditions))
                    .stream();

                while let Some((tag, item)) = global_tag_stream.try_next().await? {
                    $map.entry(*item.key()).or_insert(vec![]).push(SimpleTag {
                        uuid: tag.uuid,
                        name: tag.name,
                        tag_type: TagType::Global,
                        color: tag.color.into(),
                    });
                }
            }
        }
    }};
}

/// Numbers how many attacks of a certain kind found an aggregated model
#[derive(Copy, Clone, Serialize, ToSchema, Debug, Default)]
pub struct SimpleAggregationSource {
    /// Bruteforce subdomains via DNS requests
    bruteforce_subdomains: usize,
    /// Scan tcp ports
    tcp_port_scan: usize,
    /// Query certificate transparency
    query_certificate_transparency: usize,
    /// Query the dehashed API
    query_dehashed: usize,
    /// Check if a host is reachable via icmp
    host_alive: usize,
    /// Detect the service that is running on a port
    service_detection: usize,
    /// Resolve domain names
    dns_resolution: usize,
    /// Perform forced browsing
    forced_browsing: usize,
    /// Detect the OS of the target
    os_detection: usize,
    /// Detect if anti-port scanning techniques are in place
    anti_port_scanning_detection: usize,
    /// Scan udp ports
    udp_port_scan: usize,
    /// Perform version detection
    version_detection: usize,
    /// Manually inserted
    manual: bool,
}

impl SimpleAggregationSource {
    /// Queries the [`SimpleAggregationSource`] for a list of aggregated models
    pub async fn query(
        tx: &mut Transaction,
        workspace: Uuid,
        aggregated_table: AggregationTable,
        aggregated_uuids: impl IntoIterator<Item = Uuid>,
    ) -> Result<HashMap<Uuid, Self>, rorm::Error> {
        let aggregated_uuids: Vec<_> = aggregated_uuids
            .into_iter()
            .map(|uuid| AggregationSource::F.aggregated_uuid.equals(uuid))
            .collect();

        if aggregated_uuids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut stream = query!(
            tx,
            (
                AggregationSource::F.aggregated_uuid,
                AggregationSource::F.source_type
            )
        )
        .condition(and![
            AggregationSource::F.workspace.equals(workspace),
            AggregationSource::F
                .aggregated_table
                .equals(aggregated_table),
            DynamicCollection::or(aggregated_uuids)
        ])
        .stream();

        let mut sources: HashMap<Uuid, SimpleAggregationSource> = HashMap::new();
        while let Some((uuid, source_type)) = stream.try_next().await? {
            sources.entry(uuid).or_default().add(source_type);
        }
        Ok(sources)
    }

    fn add(&mut self, source_type: SourceType) {
        match source_type {
            SourceType::BruteforceSubdomains => self.bruteforce_subdomains += 1,
            SourceType::TcpPortScan => self.tcp_port_scan += 1,
            SourceType::QueryCertificateTransparency => self.query_certificate_transparency += 1,
            SourceType::QueryDehashed => self.query_dehashed += 1,
            SourceType::HostAlive => self.host_alive += 1,
            SourceType::ServiceDetection => self.service_detection += 1,
            SourceType::DnsResolution => self.dns_resolution += 1,
            SourceType::UdpPortScan
            | SourceType::ForcedBrowsing
            | SourceType::OSDetection
            | SourceType::VersionDetection
            | SourceType::AntiPortScanningDetection => {
                error!("Encountered unimplemented source types");
            }
            SourceType::ManualDomain
            | SourceType::ManualHost
            | SourceType::ManualPort
            | SourceType::ManualService => self.manual = true,
        }
    }
}

impl Extend<SourceType> for SimpleAggregationSource {
    fn extend<T: IntoIterator<Item = SourceType>>(&mut self, iter: T) {
        for result_type in iter {
            self.add(result_type)
        }
    }
}
