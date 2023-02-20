//!
//! This module holds all database related definitions of workspace related structs
//!

use rorm::{ForeignModel, Model, Patch};

use crate::models::User;

/**
Representation of a set of connected data.

Workspaces are owned by a user and can be shared with others.
 */
#[derive(Model)]
pub struct Workspace {
    /// Unique identifier of the workspace
    #[rorm(id)]
    pub id: i64,

    /// Name of the workspace
    #[rorm(max_length = 255)]
    pub name: String,

    /// Optional description of the workspace
    #[rorm(max_length = 65535)]
    pub description: Option<String>,

    /// The user that owns this workspace
    pub owner: ForeignModel<User>,

    /// States, if the workspace can be deleted
    #[rorm(default = true)]
    pub deletable: bool,

    /// Timestamp when the workspace was created
    #[rorm(auto_create_time)]
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Patch)]
#[rorm(model = "Workspace")]
pub(crate) struct WorkspaceInsert {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) owner: ForeignModel<User>,
    pub(crate) deletable: bool,
}
