use rorm::Patch;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::models::OauthClient;

/// Create a new oauth application
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct CreateAppRequest {
    /// The name of the application
    pub name: String,

    /// The redirect url of the application
    pub redirect_uri: String,
}

/// Update an oauth application
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct UpdateAppRequest {
    /// The name of the application
    pub name: Option<String>,

    /// The redirect url of the application
    pub redirect_uri: Option<String>,
}

/// A simple (secret-less) version of a workspace
#[derive(Serialize, Deserialize, JsonSchema, Patch, Debug, Clone)]
#[rorm(model = "OauthClient")]
pub struct SimpleOauthClient {
    /// The uuid of the client
    pub uuid: Uuid,
    /// The name of the client
    pub name: String,
    /// The redirect url of the client
    pub redirect_uri: String,
}

/// A complete version of a workspace
#[derive(Serialize, Deserialize, JsonSchema, Patch, Debug, Clone)]
#[rorm(model = "OauthClient")]
pub struct FullOauthClient {
    /// The uuid of the client
    pub uuid: Uuid,
    /// The name of the client
    pub name: String,
    /// The redirect url of the client
    pub redirect_uri: String,
    /// The secret of the client
    pub secret: String,
}

/// List all oauth applications
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ListOauthApplications {
    /// The list of applications
    pub apps: Vec<FullOauthClient>,
}
