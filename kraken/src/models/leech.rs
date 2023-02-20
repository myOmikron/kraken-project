use rorm::{Model, Patch};

/// The data collectors of kraken
#[derive(Model)]
pub struct Leech {
    /// Primary key of the leech
    #[rorm(id)]
    pub id: i64,

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
    pub(crate) name: String,
    pub(crate) address: String,
    pub(crate) description: Option<String>,
}
