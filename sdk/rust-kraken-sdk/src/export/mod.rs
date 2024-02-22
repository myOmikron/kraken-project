use kraken::api::handler::common::schema::ApiErrorResponse;
use kraken::api::handler::common::schema::ApiStatusCode;
use kraken::api::handler::data_export::schema::AggregatedWorkspace;
use reqwest::Client;
use reqwest::Url;
use uuid::Uuid;

use crate::error::KrakenError;
use crate::KrakenResult;

/// The client for the export api in kraken
#[derive(Clone)]
pub struct KrakenExportClient {
    base_url: Url,
    bearer: String,
    client: Client,
}

impl KrakenExportClient {
    /// Create a new default [KrakenExportClient].
    ///
    /// The access token can be retrieved by completing the oauth flow of kraken
    pub fn new(base_url: Url, access_token: String) -> Self {
        Self::new_with_client(base_url, access_token, Client::new())
    }

    /// Create a new [KrakenExportClient] using your own [Client]
    ///
    /// The access token can be retrieved by completing the oauth flow of kraken
    pub fn new_with_client(base_url: Url, access_token: String, client: Client) -> Self {
        Self {
            base_url,
            bearer: format!("Bearer {access_token}"),
            client,
        }
    }
}

impl KrakenExportClient {
    /// Export a workspace
    pub async fn export_workspace(&self, workspace: Uuid) -> KrakenResult<AggregatedWorkspace> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/export/workspace/{workspace}"))
            .expect("Valid url");
        let res = self
            .client
            .get(url)
            .header("Authorization", self.bearer.clone())
            .send()
            .await?;

        let status = res.status();
        let txt = res.text().await?;
        if !status.is_success() {
            return if status == 400 || status == 500 {
                let Ok(err) = serde_json::from_str(&txt) else {
                    return Err(KrakenError::DeserializeError(txt));
                };
                let err: ApiErrorResponse = err;

                if err.status_code == ApiStatusCode::Unauthenticated {
                    return Err(KrakenError::AuthenticationFailed);
                }

                Err(KrakenError::ApiError(err))
            } else {
                Err(KrakenError::DeserializeError(txt))
            };
        }

        let Ok(deserialized) = serde_json::from_str(&txt) else {
            return Err(KrakenError::DeserializeError(txt));
        };

        Ok(deserialized)
    }
}
