use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::finding_definitions::schema::CreateFindingDefinitionRequest;
use kraken::api::handler::finding_definitions::schema::FullFindingDefinition;
use kraken::api::handler::finding_definitions::schema::ListFindingDefinitions;
use kraken::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use kraken::api::handler::finding_definitions::schema::UpdateFindingDefinitionRequest;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Create a definition for a finding
    pub async fn create_finding_definition(
        &self,
        request: CreateFindingDefinitionRequest,
    ) -> KrakenResult<Uuid> {
        let response: UuidResponse = self
            .post("api/v1/findingDefinitions")
            .body(request)
            .send()
            .await?;
        Ok(response.uuid)
    }

    /// Retrieve a specific finding definition
    pub async fn get_finding_definition(&self, uuid: Uuid) -> KrakenResult<FullFindingDefinition> {
        self.get(&format!("api/v1/findingDefinitions/{uuid}"))
            .send()
            .await
    }

    /// Retrieve all finding definitions
    pub async fn get_all_finding_definitions(&self) -> KrakenResult<Vec<SimpleFindingDefinition>> {
        let list: ListFindingDefinitions = self.get("api/v1/findingDefinitions").send().await?;
        Ok(list.finding_definitions)
    }

    /// Update a finding definition
    pub async fn update_finding_definition(
        &self,
        uuid: Uuid,
        request: UpdateFindingDefinitionRequest,
    ) -> KrakenResult<()> {
        self.put(&format!("api/v1/findingDefinitions/{uuid}"))
            .body(request)
            .send()
            .await
    }
}
