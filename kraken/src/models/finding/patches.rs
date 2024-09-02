use rorm::prelude::ForeignModel;
use rorm::Patch;
use uuid::Uuid;

use crate::models::Finding;
use crate::models::FindingDefinition;
use crate::models::FindingDetails;
use crate::models::FindingSeverity;
use crate::models::Workspace;

#[derive(Patch)]
#[rorm(model = "FindingDefinition")]
pub(crate) struct InsertFindingDefinition {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) summary: String,
    pub(crate) severity: FindingSeverity,
    pub(crate) cve: Option<String>,
    pub(crate) description: String,
    pub(crate) impact: String,
    pub(crate) remediation: String,
    pub(crate) references: String,
}

#[derive(Patch)]
#[rorm(model = "Finding")]
pub(crate) struct InsertFinding {
    pub(crate) uuid: Uuid,
    pub(crate) definition: ForeignModel<FindingDefinition>,
    pub(crate) severity: FindingSeverity,
    pub(crate) details: ForeignModel<FindingDetails>,
    pub(crate) workspace: ForeignModel<Workspace>,
}
