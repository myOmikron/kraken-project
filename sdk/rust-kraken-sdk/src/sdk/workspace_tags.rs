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
        self.get(&format!("api/v1/workspaces/{workspace}/tags"))
            .send()
            .await
    }

    /// Create a workspace tag.
    pub async fn create_workspace_tag(
        &self,
        workspace: Uuid,
        create: CreateWorkspaceTagRequest,
    ) -> KrakenResult<Uuid> {
        let UuidResponse { uuid } = self
            .post(&format!("api/v1/workspaces/{workspace}/tags"))
            .body(create)
            .send()
            .await?;

        Ok(uuid)
    }

    /// Update a workspace tag.
    pub async fn update_workspace_tag(
        &self,
        workspace: Uuid,
        tag: Uuid,
        update: UpdateWorkspaceTag,
    ) -> KrakenResult<()> {
        self.put(&format!("api/v1/workspaces/{workspace}/tags/{tag}"))
            .body(update)
            .send()
            .await
    }

    /// Delete a workspace tag.
    pub async fn delete_workspace_tag(&self, workspace: Uuid, tag: Uuid) -> KrakenResult<()> {
        self.delete(&format!("api/v1/workspaces/{workspace}/tags/{tag}"))
            .send()
            .await
    }
}
