use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use utoipa::IntoParams;
use utoipa::ToSchema;
pub use utoipa_fix::BruteforceSubdomainsResultsPage;
pub use utoipa_fix::DnsResolutionResultsPage;
pub use utoipa_fix::DnsTxtScanResultsPage;
pub use utoipa_fix::DomainResultsPage;
pub use utoipa_fix::HostAliveResultsPage;
pub use utoipa_fix::HostResultsPage;
pub use utoipa_fix::OsDetectionResultsPage;
pub use utoipa_fix::Page;
pub use utoipa_fix::PortResultsPage;
pub use utoipa_fix::QueryCertificateTransparencyResultsPage;
pub use utoipa_fix::QueryUnhashedResultsPage;
pub use utoipa_fix::SearchResultPage;
pub use utoipa_fix::SearchesResultPage;
pub use utoipa_fix::ServiceDetectionResultsPage;
pub use utoipa_fix::ServiceResultsPage;
pub use utoipa_fix::UdpServiceDetectionResultsPage;
use uuid::Uuid;

use crate::models::Color;

/// A common response that contains a single uuid
#[derive(Deserialize, Serialize, ToSchema, Debug, Copy, Clone)]
pub struct UuidResponse {
    /// The uuid
    pub uuid: Uuid,
}

/// A path with an UUID
#[derive(Deserialize, Serialize, IntoParams, Debug, Clone, Copy)]
pub struct PathUuid {
    /// The uuid
    pub uuid: Uuid,
}

/// Query parameters for paginated data
#[derive(Copy, Clone, Debug, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct PageParams {
    /// Number of items to retrieve
    #[param(example = 50, minimum = 1)]
    pub limit: u64,

    /// Position in the whole list to start retrieving from
    #[param(example = 0)]
    pub offset: u64,
}

#[allow(missing_docs)]
mod utoipa_fix {
    use serde::Deserialize;
    use serde::Serialize;
    use utoipa::ToSchema;

    use crate::api::handler::attack_results::schema::FullDnsTxtScanResult;
    use crate::api::handler::attack_results::schema::FullOsDetectionResult;
    use crate::api::handler::attack_results::schema::FullQueryCertificateTransparencyResult;
    use crate::api::handler::attack_results::schema::FullServiceDetectionResult;
    use crate::api::handler::attack_results::schema::FullUdpServiceDetectionResult;
    use crate::api::handler::attack_results::schema::SimpleBruteforceSubdomainsResult;
    use crate::api::handler::attack_results::schema::SimpleDnsResolutionResult;
    use crate::api::handler::attack_results::schema::SimpleHostAliveResult;
    use crate::api::handler::attack_results::schema::SimpleQueryUnhashedResult;
    use crate::api::handler::domains::schema::FullDomain;
    use crate::api::handler::files::schema::FullFile;
    use crate::api::handler::hosts::schema::FullHost;
    use crate::api::handler::ports::schema::FullPort;
    use crate::api::handler::services::schema::FullService;
    use crate::api::handler::workspaces::schema::SearchEntry;
    use crate::api::handler::workspaces::schema::SearchResultEntry;

    /// Response containing paginated data
    #[derive(Serialize, Deserialize, Default, ToSchema, Clone)]
    #[aliases(
        DomainResultsPage = Page<FullDomain>,
        HostResultsPage = Page<FullHost>,
        ServiceResultsPage = Page<FullService>,
        PortResultsPage = Page<FullPort>,
        BruteforceSubdomainsResultsPage = Page<SimpleBruteforceSubdomainsResult>,
        QueryCertificateTransparencyResultsPage = Page<FullQueryCertificateTransparencyResult>,
        QueryUnhashedResultsPage = Page<SimpleQueryUnhashedResult>,
        HostAliveResultsPage = Page<SimpleHostAliveResult>,
        ServiceDetectionResultsPage = Page<FullServiceDetectionResult>,
        UdpServiceDetectionResultsPage = Page<FullUdpServiceDetectionResult>,
        DnsResolutionResultsPage = Page<SimpleDnsResolutionResult>,
        DnsTxtScanResultsPage = Page<FullDnsTxtScanResult>,
        OsDetectionResultsPage = Page<FullOsDetectionResult>,
        SearchResultPage = Page<SearchResultEntry>,
        SearchesResultPage = Page<SearchEntry>,
        FullFilesPage = Page<FullFile>,
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

/// The type of a tag
#[derive(
    Serialize, Deserialize, ToSchema, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash,
)]
pub enum TagType {
    /// Workspace tag
    Workspace,
    /// Global tag
    Global,
}

/// A simple tag
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleTag {
    /// The uuid of the tag
    pub uuid: Uuid,
    /// The name of the tag
    pub name: String,
    /// The color of the tag
    pub color: Color,
    /// The type of the tag
    pub tag_type: TagType,
}

/// This type holds all possible error types that can be returned by the API.
///
/// Numbers between 1000 and 1999 (inclusive) are client errors that can be handled by the client.
/// Numbers between 2000 and 2999 (inclusive) are server errors.
#[derive(Serialize_repr, Deserialize_repr, ToSchema, Debug, PartialOrd, PartialEq)]
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
    /// The streamed request body failed
    PayloadError = 2006,
    /// The uploaded image file is invalid
    InvalidImage = 2007,
}

/// Representation of an error response
///
/// `status_code` holds the error code, `message` a human readable description of the error
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ApiErrorResponse {
    /// The error code
    pub status_code: ApiStatusCode,
    /// A human readable description of the error
    #[schema(example = "Error message will be here")]
    pub message: String,
}

impl ApiErrorResponse {
    /// Create a new error response
    pub fn new(status_code: ApiStatusCode, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }
}
