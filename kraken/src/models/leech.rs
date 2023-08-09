use rorm::{Model, Patch};
use uuid::Uuid;

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
}

#[derive(Patch)]
#[rorm(model = "Leech")]
pub(crate) struct LeechInsert {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) address: String,
    pub(crate) description: Option<String>,
}
