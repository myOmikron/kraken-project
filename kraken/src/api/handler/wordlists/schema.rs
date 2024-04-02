use rorm::Patch;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::models::WordList;

/// Arguments for creating a new wordlist
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct CreateWordlistRequest {
    /// The wordlist's name to be displayed select buttons
    pub name: String,
    /// A description explaining the wordlist's intended use case
    pub description: String,
    /// The file path the wordlist is deployed under on each leech
    pub path: String,
}

/// Arguments for updating an existing wordlist
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct UpdateWordlistRequest {
    /// The wordlist's name to be displayed select buttons
    pub name: Option<String>,
    /// A description explaining the wordlist's intended use case
    pub description: Option<String>,
    /// The file path the wordlist is deployed under on each leech
    pub path: Option<String>,
}

/// A wordlist without its `path` field
#[derive(Serialize, Deserialize, JsonSchema, Patch, Debug, Clone)]
#[rorm(model = "WordList")]
pub struct SimpleWordlist {
    /// The primary key of the wordlist
    pub uuid: Uuid,
    /// The wordlist's name to be displayed select buttons
    pub name: String,
    /// A description explaining the wordlist's intended use case
    pub description: String,
}

/// A wordlist including its `path` field only meant for admins
#[derive(Serialize, Deserialize, JsonSchema, Patch, Debug, Clone)]
#[rorm(model = "WordList")]
pub struct FullWordlist {
    /// The primary key of the wordlist
    pub uuid: Uuid,
    /// The wordlist's name to be displayed select buttons
    pub name: String,
    /// A description explaining the wordlist's intended use case
    pub description: String,
    /// The file path the wordlist is deployed under on each leech
    pub path: String,
}

/// Response containing all wordlists including their `path` fields
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ListWordlistsAdmin {
    /// List of all wordlists including their `path` fields
    pub wordlists: Vec<FullWordlist>,
}

/// Response containing all wordlists
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ListWordlists {
    /// List of all wordlists
    pub wordlists: Vec<SimpleWordlist>,
}
