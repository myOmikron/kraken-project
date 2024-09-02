use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use crate::modules::finding_factory::schema::FindingFactoryIdentifier;

/// The list of all finding factory entries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetFindingFactoryEntriesResponse {
    /// Map of all active finding factory identifiers and their finding definitions
    pub entries: HashMap<FindingFactoryIdentifier, SimpleFindingDefinition>,
}

/// The request to set a finding factory entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetFindingFactoryEntryRequest {
    /// The identifier whose entry to set
    pub identifier: FindingFactoryIdentifier,

    /// Set's the entry's finding definition or `None` to disable an entry
    pub finding_definition: Option<Uuid>,
}
