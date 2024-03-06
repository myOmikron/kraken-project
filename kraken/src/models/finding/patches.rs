use rorm::Patch;
use uuid::Uuid;

use crate::models::FindingDefinition;
use crate::models::FindingSeverity;

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
