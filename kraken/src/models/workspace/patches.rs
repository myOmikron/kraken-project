use rorm::prelude::ForeignModel;
use rorm::Patch;
use uuid::Uuid;

use crate::models::Workspace;
use crate::models::WorkspaceNotes;

#[derive(Patch)]
#[rorm(model = "WorkspaceNotes")]
pub(crate) struct WorkspaceNotesInsert {
    pub(crate) uuid: Uuid,
    pub(crate) notes: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}
