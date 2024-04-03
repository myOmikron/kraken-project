use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::workspace_tags::schema::CreateWorkspaceTagRequest;
use kraken::api::handler::workspace_tags::schema::ListWorkspaceTags;
use kraken::api::handler::workspace_tags::schema::UpdateWorkspaceTag;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Retrieve all workspace tags
    pub async fn get_all_workspace_tags(&self, workspace: Uuid) -> KrakenResult<ListWorkspaceTags> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/tags"))
            .expect("Valid url");

        self.get(url).send().await
    }

    /// Create a workspace tag.
    pub async fn create_workspace_tag(
        &self,
        workspace: Uuid,
        create: CreateWorkspaceTagRequest,
    ) -> KrakenResult<Uuid> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/tags"))
            .expect("Valid url");

        let UuidResponse { uuid } = self.post(url).body(create).send().await?;

        Ok(uuid)
    }

    /// Update a workspace tag.
    pub async fn update_workspace_tag(
        &self,
        workspace: Uuid,
        tag: Uuid,
        update: UpdateWorkspaceTag,
    ) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/tags/{tag}"))
            .expect("Valid url");

        self.put(url).body(update).send().await
    }

    /// Delete a workspace tag.
    pub async fn delete_workspace_tag(&self, workspace: Uuid, tag: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/tags/{tag}"))
            .expect("Valid url");

        self.delete(url).send().await
    }
}
