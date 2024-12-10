use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::workspaces::schema::CreateWorkspaceRequest;
use kraken::api::handler::workspaces::schema::FullWorkspace;
use kraken::api::handler::workspaces::schema::ListWorkspaces;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Retrieve all workspaces the user has access to
    pub async fn get_all_workspaces(&self) -> KrakenResult<ListWorkspaces> {
        self.get("api/v1/workspaces").send().await
    }

    /// Retrieve a workspace by its uuid
    pub async fn get_workspace(&self, workspace: Uuid) -> KrakenResult<FullWorkspace> {
        self.get(&format!("api/v1/workspaces/{workspace}"))
            .send()
            .await
    }

    /// Create a new workspace in kraken
    pub async fn create_workspace(
        &self,
        name: String,
        description: Option<String>,
    ) -> KrakenResult<UuidResponse> {
        self.post("api/v1/workspaces")
            .body(CreateWorkspaceRequest { name, description })
            .send()
            .await
    }

    /// Archive an existing workspace
    pub async fn archive_workspace(&self, workspace: Uuid) -> KrakenResult<()> {
        self.post(&format!("api/v1/workspaces/{workspace}/archive"))
            .send()
            .await
    }
}
