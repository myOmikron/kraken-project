//! The schema for creating and managing bearer tokens

use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// The request to create a new bearer token
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateBearerTokenRequest {
    /// A descriptive name of the token
    pub name: String,
}

/// A full representation of a bearer token
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct FullBearerToken {
    /// The primary key of the token
    pub uuid: Uuid,
    /// The name that is used for identification
    pub name: String,
    /// The token that is used in the authorization header
    pub token: String,
}

/// A list of bearer tokens
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ListBearerTokens {
    /// List of tokens
    pub tokens: Vec<FullBearerToken>,
}
