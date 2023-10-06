use rorm::prelude::ForeignModel;
use rorm::Model;
use uuid::Uuid;

pub use crate::models::tag::operations::Color;
use crate::models::Workspace;

mod operations;

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

/// A tag that can be applied to any aggregated data.
///
/// It is only valid in a specific workspace
#[derive(Model)]
pub struct WorkspaceTag {
    /// The primary key of a workspace tag
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the workspace tag
    #[rorm(max_length = 255)]
    pub name: String,

    /// The color of the tag, converted from hex
    pub color: i32,

    /// The workspace this tag is assigned to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}
