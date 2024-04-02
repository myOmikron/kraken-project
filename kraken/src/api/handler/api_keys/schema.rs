use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

/// Request to create a new api key
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct CreateApiKeyRequest {
    /// A descriptive name helping the user to identify the key
    pub name: String,
}

/// The request to update an api key
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct UpdateApiKeyRequest {
    /// A descriptive name helping the user to identify the key
    pub name: String,
}

/// A representation of a full api key
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct FullApiKey {
    /// The key's identifier
    pub uuid: Uuid,

    /// A descriptive name helping the user to identify the key
    pub name: String,

    /// The actual key's value
    pub key: String,
}

/// The response that contains all api keys
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct ListApiKeys {
    /// The list of api keys
    pub keys: Vec<FullApiKey>,
}
