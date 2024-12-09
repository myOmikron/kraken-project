use kraken::api::handler::workspace_invitations::schema::WorkspaceInvitationList;
use kraken::api::handler::workspaces::schema::InviteToWorkspaceRequest;
use uuid::Uuid;

use crate::error::KrakenError;
use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// List of open invitations the logged-in user has received
    pub async fn get_all_invitations(&self) -> KrakenResult<WorkspaceInvitationList> {
        self.get("api/v1/invitations").send().await
    }

    /// Accept an open invitation to a workspace
    pub async fn accept_invitation(&self, invitation: Uuid) -> Result<(), KrakenError> {
        self.post(&format!("api/v1/invitations/{invitation}/accept"))
            .send()
            .await
    }

    /// Decline an open invitation to a workspace
    pub async fn decline_invitation(&self, invitation: Uuid) -> KrakenResult<()> {
        self.post(&format!("api/v1/invitations/{invitation}/decline"))
            .send()
            .await
    }

    /// Create a new invitation to a given workspace
    pub async fn create_invitation(&self, workspace: Uuid, user: Uuid) -> KrakenResult<Uuid> {
        self.post(&format!("api/v1/workspaces/{workspace}/invitations"))
            .body(InviteToWorkspaceRequest { user })
            .send()
            .await
    }
}
