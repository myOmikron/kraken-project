/// ApiErrorResponse : Representation of an error response  `status_code` holds the error code, `message` a human readable description of the error
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ApiErrorResponse {
    #[serde(rename = "status_code")]
    pub status_code: ApiStatusCode,
    #[serde(rename = "message")]
    pub message: String,
}

/// ApiStatusCode : This type holds all possible error types that can be returned by the API.  Numbers between 1000 and 1999 (inclusive) are client errors that can be handled by the client. Numbers between 2000 and 2999 (inclusive) are server errors.
///
/// This type holds all possible error types that can be returned by the API.  Numbers between 1000 and 1999 (inclusive) are client errors that can be handled by the client. Numbers between 2000 and 2999 (inclusive) are server errors.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum ApiStatusCode {
    /// Login failed
    #[serde(rename = "1000")]
    LoginFailed = 1000,
    /// Not found
    #[serde(rename = "1001")]
    NotFound = 1001,
    /// Invalid content type
    #[serde(rename = "1002")]
    InvalidContentType = 1002,
    /// Invalid json
    #[serde(rename = "1003")]
    InvalidJson = 1003,
    /// Payload overflow
    #[serde(rename = "1004")]
    PayloadOverflow = 1004,

    /// User is unauthenticated
    #[serde(rename = "1005")]
    Unauthenticated = 1005,
    /// User is missing a required second factor
    #[serde(rename = "1006")]
    Missing2fa = 1006,
    /// The user is missing privileges
    #[serde(rename = "1007")]
    MissingPrivileges = 1007,
    /// No security key is available, but it is required
    #[serde(rename = "1008")]
    NoSecurityKeyAvailable = 1008,
    /// User already exists
    #[serde(rename = "1009")]
    UserAlreadyExists = 1009,
    /// Invalid username
    #[serde(rename = "1010")]
    InvalidUsername = 1010,
    /// Invalid address
    #[serde(rename = "1011")]
    InvalidAddress = 1011,
    /// Address already exists
    #[serde(rename = "1012")]
    AddressAlreadyExists = 1012,
    /// Name already exists
    #[serde(rename = "1013")]
    NameAlreadyExists = 1013,
    /// Invalid uuid
    #[serde(rename = "1014")]
    InvalidUuid = 1014,
    /// The given workspace is not valid
    #[serde(rename = "1015")]
    InvalidWorkspace = 1015,
    /// Received an empty json request.
    ///
    /// Mostly happens in update endpoints without supplying an update
    #[serde(rename = "1016")]
    EmptyJson = 1016,
    /// Invalid password
    #[serde(rename = "1017")]
    InvalidPassword = 1017,
    /// Invalid leech
    #[serde(rename = "1018")]
    InvalidLeech = 1018,
    /// Username is already occupied
    #[serde(rename = "1019")]
    UsernameAlreadyExists = 1019,
    /// Invalid name specified
    #[serde(rename = "1020")]
    InvalidName = 1020,
    /// Invalid query limit
    #[serde(rename = "1021")]
    InvalidQueryLimit = 1021,
    /// Invalid port
    #[serde(rename = "1022")]
    InvalidPort = 1022,
    /// Empty targets
    #[serde(rename = "1023")]
    EmptyTargets = 1023,
    /// Path already exists
    #[serde(rename = "1024")]
    PathAlreadyExists = 1024,

    /// Internal server error
    #[serde(rename = "2000")]
    InternalServerError = 2000,
    /// An database error occurred
    #[serde(rename = "2001")]
    DatabaseError = 2001,
    /// An error occurred while interacting with the user session
    #[serde(rename = "2002")]
    SessionError = 2002,
    /// Webauthn error
    #[serde(rename = "2003")]
    WebauthnError = 2003,
    /// Dehashed is not available due to missing credentials
    #[serde(rename = "2004")]
    DehashedNotAvailable = 2004,
    /// There's no leech available
    #[serde(rename = "2005")]
    NoLeechAvailable = 2005,
}
