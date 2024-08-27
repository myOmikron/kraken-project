use kraken::api::handler::finding_affected::schema::CreateFindingAffectedRequest;
use kraken::api::handler::finding_affected::schema::FullFindingAffected;
use kraken::api::handler::finding_affected::schema::UpdateFindingAffectedRequest;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Add a new affected object to a finding
    pub async fn create_finding_affected(
        &self,
        workspace: Uuid,
        finding: Uuid,
        request: CreateFindingAffectedRequest,
    ) -> KrakenResult<()> {
        self.post(&format!(
            "api/v1/workspace/{workspace}/findings/{finding}/affected"
        ))
        .body(request)
        .send()
        .await
    }

    /// Get an object affected by a finding
    pub async fn get_finding_affected(
        &self,
        workspace: Uuid,
        finding: Uuid,
        affected: Uuid,
    ) -> KrakenResult<FullFindingAffected> {
        self.get(&format!(
            "api/v1/workspace/{workspace}/findings/{finding}/affected/{affected}"
        ))
        .send()
        .await
    }

    /// Update the details of an affected object
    pub async fn update_finding_affected(
        &self,
        workspace: Uuid,
        finding: Uuid,
        affected: Uuid,
        request: UpdateFindingAffectedRequest,
    ) -> KrakenResult<()> {
        self.put(&format!(
            "api/v1/workspace/{workspace}/findings/{finding}/affected/{affected}"
        ))
        .body(request)
        .send()
        .await
    }

    /// Remove an affected object from a finding
    pub async fn delete_finding_affected(
        &self,
        workspace: Uuid,
        finding: Uuid,
        affected: Uuid,
    ) -> KrakenResult<()> {
        self.delete(&format!(
            "api/v1/workspace/{workspace}/findings/{finding}/affected/{affected}"
        ))
        .send()
        .await
    }
}
