use rorm::prelude::*;
use uuid::Uuid;

pub use self::operations::*;

mod operations;

/// The data collectors of kraken
#[derive(Model)]
pub struct Leech {
    /// Primary key of the leech
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the leech
    #[rorm(max_length = 255, unique)]
    pub name: String,

    /// Address of the leech
    #[rorm(max_length = 255, unique)]
    pub address: String,

    /// Optional description of a leech
    #[rorm(max_length = 65535)]
    pub description: Option<String>,

    /// Random string the leech has to present when connecting to kraken during the backlog process.
    ///
    /// It enables blocking incoming connections of outdated leeches whose TLS cert are still valid.
    #[rorm(max_length = 255)]
    pub secret: String,
}
