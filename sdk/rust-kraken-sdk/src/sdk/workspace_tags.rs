use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::workspace_tags::schema::{
    CreateWorkspaceTagRequest, ListWorkspaceTags, UpdateWorkspaceTag,
};
use uuid::Uuid;

use crate::sdk::utils::KrakenRequest;
use crate::{KrakenClient, KrakenResult};

impl KrakenClient {
    /// Retrieve all workspace tags
    pub async fn get_all_workspace_tags(&self, workspace: Uuid) -> KrakenResult<ListWorkspaceTags> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/tags"))
            .expect("Valid url");

        self.make_request(KrakenRequest::get(url).build()).await
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

        let UuidResponse { uuid } = self
            .make_request(KrakenRequest::post(url).body(create).build())
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
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/tags/{tag}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::put(url).body(update).build())
            .await
    }

    /// Delete a workspace tag.
    pub async fn delete_workspace_tag(&self, workspace: Uuid, tag: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/tags/{tag}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::delete(url).build()).await
    }
}
