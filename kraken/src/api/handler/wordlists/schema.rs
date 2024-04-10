use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Arguments for creating a new wordlist
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct CreateWordlistRequest {
    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: String,
    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: String,
    /// The file path the wordlist is deployed under on each leech
    #[schema(example = "/opt/wordlists/Discovery/DNS/subdomains-top1million-5000.txt")]
    pub path: String,
}

/// Arguments for updating an existing wordlist
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct UpdateWordlistRequest {
    /// The primary key of the wordlist to update
    pub uuid: Uuid,
    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: Option<String>,
    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: Option<String>,
    /// The file path the wordlist is deployed under on each leech
    #[schema(example = "/opt/wordlists/Discovery/DNS/subdomains-top1million-5000.txt")]
    pub path: Option<String>,
}

/// A wordlist without its `path` field
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[cfg_attr(feature = "bin", derive(rorm::Patch))]
#[cfg_attr(feature = "bin", rorm(model = "crate::models::WordList"))]
pub struct SimpleWordlist {
    /// The primary key of the wordlist
    pub uuid: Uuid,
    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: String,
    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: String,
}

/// A wordlist including its `path` field only meant for admins
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[cfg_attr(feature = "bin", derive(rorm::Patch))]
#[cfg_attr(feature = "bin", rorm(model = "crate::models::WordList"))]
pub struct FullWordlist {
    /// The primary key of the wordlist
    pub uuid: Uuid,
    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: String,
    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: String,
    /// The file path the wordlist is deployed under on each leech
    #[schema(example = "/opt/wordlists/Discovery/DNS/subdomains-top1million-5000.txt")]
    pub path: String,
}

/// Response containing all wordlists including their `path` fields
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ListWordlistsAdmin {
    /// List of all wordlists including their `path` fields
    pub wordlists: Vec<FullWordlist>,
}

/// Response containing all wordlists
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ListWordlists {
    /// List of all wordlists
    pub wordlists: Vec<SimpleWordlist>,
}
