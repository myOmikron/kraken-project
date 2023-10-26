use rorm::prelude::ForeignModel;
use rorm::Patch;
use uuid::Uuid;

use crate::models::{Search, User, Workspace};

#[derive(Patch)]
#[rorm(model = "Search")]
pub(crate) struct SearchInsert {
    pub(crate) uuid: Uuid,
    pub(crate) started_by: ForeignModel<User>,
    pub(crate) workspace: ForeignModel<Workspace>,
    pub(crate) search_term: String,
}
