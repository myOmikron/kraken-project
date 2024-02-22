use kraken::api::handler::workspace_invitations::schema::WorkspaceInvitationList;
use uuid::Uuid;

use crate::error::KrakenError;
use crate::sdk::utils::KrakenRequest;
use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// List of open invitations the logged-in user has received
    pub async fn get_all_invitations(&self) -> KrakenResult<WorkspaceInvitationList> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/invitations").expect("Valid url");

        self.make_request(KrakenRequest::get(url).build()).await
    }

    /// Accept an open invitation to a workspace
    pub async fn accept_invitation(&self, invitation: Uuid) -> Result<(), KrakenError> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/invitations/{invitation}/accept"))
            .expect("Valid url");

        self.make_request(KrakenRequest::post(url).build()).await?;

        Ok(())
    }

    /// Decline an open invitation to a workspace
    pub async fn decline_invitation(&self, invitation: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/invitations/{invitation}/decline"))
            .expect("Valid url");

        self.make_request(KrakenRequest::post(url).build()).await?;

        Ok(())
    }
}
