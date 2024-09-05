use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::common::de_optional;
use crate::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use crate::modules::finding_factory::schema::FindingFactoryIdentifier;

/// The list of all finding factory entries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetFindingFactoryEntriesResponse {
    /// Map of all active finding factory identifiers and their finding definitions
    pub entries: HashMap<FindingFactoryIdentifier, FullFindingFactoryEntry>,
}

/// Settings mapping an identifier to a finding definition
///
/// An identifier is an enum variant which identifies one kind of issue,
/// the finding factory might create a finding for.
///
/// If the finding factory detects an issue it will look up its identifier's finding definition
/// and create a finding using this definition (if it found any).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FullFindingFactoryEntry {
    /// Identifies the issue a finding could be created for.
    pub identifier: FindingFactoryIdentifier,

    /// The finding definition to create a finding with, if the identifier's associated issue is found.
    pub finding: Option<SimpleFindingDefinition>,
}

/// The request to set a finding factory entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFindingFactoryEntryRequest {
    /// The identifier whose entry to set
    pub identifier: FindingFactoryIdentifier,

    /// Set's the entry's finding definition or `None` to disable an entry
    #[serde(
        default,
        deserialize_with = "de_optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub finding_definition: Option<Option<Uuid>>,
}
