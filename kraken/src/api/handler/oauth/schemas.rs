//! Different request and response types as defined in [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749)

use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The client constructs the request URI by adding the following
/// parameters to the query component of the authorization endpoint URI
/// using the "application/x-www-form-urlencoded" format
#[derive(Deserialize)]
pub(crate) struct AuthRequest {
    /// Value MUST be set to "code".
    pub response_type: String,

    /// The client identifier as described in [Section 2.2](https://www.rfc-editor.org/rfc/rfc6749#section-2.2).
    pub client_id: Uuid,

    /// As described in [Section 3.1.2](https://www.rfc-editor.org/rfc/rfc6749#section-3.1.2).
    pub redirect_uri: Option<String>,

    /// The scope of the access request as described by [Section 3.3](https://www.rfc-editor.org/rfc/rfc6749#section-3.3).
    pub scope: Option<String>,

    /// An opaque value used by the client to maintain
    /// state between the request and callback.  The authorization
    /// server includes this value when redirecting the user-agent back
    /// to the client.  The parameter SHOULD be used for preventing
    /// cross-site request forgery as described in [Section 10.12](https://www.rfc-editor.org/rfc/rfc6749#section-10.12).
    pub state: Option<String>,
    //code_challenge: String,
    //code_challenge_method: CodeChallengeMethod
}

#[derive(Serialize, Debug)]
pub(crate) struct AuthError {
    pub error: AuthErrorType,
    pub state: Option<String>,
    pub error_description: Option<&'static str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthErrorType {
    /// The request is missing a required parameter, includes an
    /// invalid parameter value, includes a parameter more than
    /// once, or is otherwise malformed.
    InvalidRequest,

    /// The client is not authorized to request an
    /// authorization code using this method.
    UnauthorizedClient,

    /// The resource owner or authorization server denied the request.
    AccessDenied,

    /// The authorization server does not support obtaining an
    /// authorization code using this method.
    UnsupportedResponseType,

    /// The requested scope is invalid, unknown, or malformed.
    InvalidScope,

    /// The authorization server encountered an unexpected
    /// condition that prevented it from fulfilling the request.
    /// (This error code is needed because a 500
    /// Internal Server Error HTTP status code cannot be returned
    /// to the client via an HTTP redirect.)
    ServerError,

    /// The authorization server is currently unable to handle
    /// the request due to a temporary overloading or maintenance
    /// of the server.  (This error code is needed because a 503
    /// Service Unavailable HTTP status code cannot be returned
    /// to the client via an HTTP redirect.)
    TemporarilyUnavailable,
}

#[derive(Deserialize)]
pub(crate) struct TokenRequest {
    /// Must be "authorization_code"
    pub grant_type: GrantType,
    pub code: Uuid,
    pub redirect_uri: String,
    pub client_id: Uuid,
    pub client_secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GrantType {
    AuthorizationCode,
}

#[derive(Serialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,

    #[serde(serialize_with = "duration_seconds")]
    pub expires_in: Duration,
}

fn duration_seconds<S: serde::ser::Serializer>(
    duration: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(duration.as_secs())
}
