//! The schema of the module `finding_definitions` are here

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::FindingSeverity;

/// The request to create a new finding definition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFindingDefinitionRequest {
    /// Name of the new finding definition
    ///
    /// This must be unique
    pub name: String,
    /// The severity of the finding
    pub severity: FindingSeverity,
    /// Short summary of the finding
    pub summary: String,
    /// Optional linked CVE
    pub cve: Option<String>,
    /// The full description of the finding
    ///
    /// This should include the cause of the finding
    pub description: String,
    /// The impact of the finding in general.
    ///
    /// The impact for a specific environment should be described in the linked finding
    pub impact: String,
    /// How is remediation possible in general?
    ///
    /// For example when using weak ciphers, the easiest remediation might be to just
    /// rework the process of creating a new certificate and use safer parameters
    pub remediation: String,
    /// Any references to get more information about the definition of a finding.
    ///
    /// Can link to resources like Mitre's Attack or CME explanations, etc.
    pub references: String,
}

/// The full definition of a finding
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FullFindingDefinition {
    /// The uuid of a finding definition
    pub uuid: Uuid,
    /// Name of the new finding definition
    pub name: String,
    /// The severity of the finding
    pub severity: FindingSeverity,
    /// Short summary of the finding
    pub summary: String,
    /// Optional linked CVE
    pub cve: Option<String>,
    /// The full description of the finding
    pub description: String,
    /// The impact of the finding in general.
    pub impact: String,
    /// How to remediate the finding
    pub remediation: String,
    /// Any references to get more information about the definition of a finding.
    pub references: String,
    /// The point in time this finding definition was created
    pub created_at: DateTime<Utc>,
}

/// The simple definition of a finding
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimpleFindingDefinition {
    /// The uuid of a finding definition
    pub uuid: Uuid,
    /// Name of the new finding definition
    pub name: String,
    /// The severity of the finding
    pub severity: FindingSeverity,
    /// Short summary of the finding
    pub summary: String,
    /// The point in time this finding definition was created
    pub created_at: DateTime<Utc>,
}

/// A list of simple definition of a finding
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListFindingDefinitions {
    /// The finding definitions
    pub finding_definitions: Vec<SimpleFindingDefinition>,
}
