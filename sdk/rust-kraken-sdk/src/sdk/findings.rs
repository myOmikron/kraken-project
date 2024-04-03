use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::findings::schema::CreateFindingRequest;
use kraken::api::handler::findings::schema::FullFinding;
use kraken::api::handler::findings::schema::ListFindings;
use kraken::api::handler::findings::schema::SimpleFinding;
use kraken::api::handler::findings::schema::UpdateFindingRequest;
use uuid::Uuid;

use crate::sdk::utils::KrakenRequest;
use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Creates a new finding
    pub async fn create_finding(
        &self,
        workspace: Uuid,
        request: CreateFindingRequest,
    ) -> KrakenResult<Uuid> {
        let response: UuidResponse = self
            .post(&format!("api/v1/workspace/{workspace}/findings"))
            .body(request)
            .send()
            .await?;
        Ok(response.uuid)
    }

    /// Gets a workspace's findings
    pub async fn get_all_findings(&self, workspace: Uuid) -> KrakenResult<Vec<SimpleFinding>> {
        let response: ListFindings = self
            .get(&format!("api/v1/workspace/{workspace}/findings"))
            .send()
            .await?;
        Ok(response.findings)
    }

    /// Gets a single finding
    pub async fn get_finding(&self, workspace: Uuid, finding: Uuid) -> KrakenResult<FullFinding> {
        self.get(&format!("api/v1/workspace/{workspace}/findings/{finding}"))
            .send()
            .await
    }

    /// Updates a finding
    pub async fn update_finding(
        &self,
        workspace: Uuid,
        finding: Uuid,
        request: UpdateFindingRequest,
    ) -> KrakenResult<()> {
        self.put(&format!("api/v1/workspace/{workspace}/findings/{finding}"))
            .body(request)
            .send()
            .await
    }

    /// Deletes a finding
    pub async fn delete_finding(&self, workspace: Uuid, finding: Uuid) -> KrakenResult<()> {
        self.delete(&format!("api/v1/workspace/{workspace}/findings/{finding}"))
            .send()
            .await
    }
}
