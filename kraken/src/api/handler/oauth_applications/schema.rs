use rorm::Patch;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::OauthClient;

/// Create a new oauth application
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct CreateAppRequest {
    /// The name of the application
    #[schema(example = "Trustworthy application")]
    pub name: String,

    /// The redirect url of the application
    #[schema(example = "http://127.0.0.1:8080")]
    pub redirect_uri: String,
}

/// Update an oauth application
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct UpdateAppRequest {
    /// The name of the application
    #[schema(example = "Trustworthy application")]
    pub name: Option<String>,

    /// The redirect url of the application
    #[schema(example = "http://127.0.0.1:8080")]
    pub redirect_uri: Option<String>,
}

/// A simple (secret-less) version of a workspace
#[derive(Serialize, Deserialize, ToSchema, Patch, Debug, Clone)]
#[rorm(model = "OauthClient")]
pub struct SimpleOauthClient {
    /// The uuid of the client
    pub uuid: Uuid,
    /// The name of the client
    #[schema(example = "Trustworthy application")]
    pub name: String,
    /// The redirect url of the client
    #[schema(example = "http://127.0.0.1:8080")]
    pub redirect_uri: String,
}

/// A complete version of a workspace
#[derive(Serialize, Deserialize, ToSchema, Patch, Debug, Clone)]
#[rorm(model = "OauthClient")]
pub struct FullOauthClient {
    /// The uuid of the client
    pub uuid: Uuid,
    /// The name of the client
    #[schema(example = "Trustworthy application")]
    pub name: String,
    /// The redirect url of the client
    #[schema(example = "http://127.0.0.1:8080")]
    pub redirect_uri: String,
    /// The secret of the client
    #[schema(example = "IPSPL29BSDw5HFir5LYamdlm6SiaBdwx")]
    pub secret: String,
}

/// List all oauth applications
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ListOauthApplications {
    /// The list of applications
    pub apps: Vec<FullOauthClient>,
}
