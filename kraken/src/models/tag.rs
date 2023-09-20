use rorm::{Model, Patch};
use uuid::Uuid;

/// A global tag that can be applied to any aggregated data.
///
/// The difference between global and workspace tags is the visibility.
/// Global tags can be accessed from every workspace while workspace tags can be only accessed from
/// the workspace they were created in.
#[derive(Model, Debug)]
pub struct GlobalTag {
    /// The primary key of a global tag
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the global tag
    #[rorm(max_length = 255, unique)]
    pub name: String,

    /// The color of the tag, converted from hex
    pub color: i32,
}

#[derive(Patch)]
#[rorm(model = "GlobalTag")]
pub(crate) struct GlobalTagInsert {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) color: i32,
}
