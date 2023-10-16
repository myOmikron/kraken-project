mod operations;

use rorm::prelude::*;
use uuid::Uuid;

/// A file deployed on every leech used for the bruteforce subdomains attack
#[derive(Model)]
pub struct WordList {
    /// Primary key of the wordlist
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The wordlist's name displayed in a select button
    #[rorm(max_length = 255, unique)]
    pub name: String,

    /// A description explaining the wordlist's intended use case
    #[rorm(max_length = 1024)]
    pub description: String,

    /// The file path the wordlist is deployed under on each leech
    #[rorm(max_length = 255, unique)]
    pub path: String,
}
