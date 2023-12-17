use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to create a new api key
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct CreateApiKeyRequest {
    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    pub name: String,
}

/// The request to update an api key
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct UpdateApiKeyRequest {
    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    pub name: String,
}

/// A representation of a full api key
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct FullApiKey {
    /// The key's identifier
    pub uuid: Uuid,

    /// A descriptive name helping the user to identify the key
    #[schema(example = "Leech on my local machine")]
    pub name: String,

    /// The actual key's value
    #[schema(example = "fsn83r0jfis84nfthw...")]
    pub key: String,
}

/// The response that contains all api keys
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct ListApiKeys {
    /// The list of api keys
    pub keys: Vec<FullApiKey>,
}
