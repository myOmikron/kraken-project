use kraken::api::handler::workspaces::schema::FullWorkspace;
use kraken::api::handler::workspaces::schema::ListWorkspaces;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Retrieve all workspaces the user has access to
    pub async fn get_all_workspaces(&self) -> KrakenResult<ListWorkspaces> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/workspaces").expect("Valid url");

        self.get(url).send().await
    }

    /// Retrieve a workspace by its uuid
    pub async fn get_workspace(&self, workspace: Uuid) -> KrakenResult<FullWorkspace> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}"))
            .expect("Valid url");

        self.get(url).send().await
    }
}
