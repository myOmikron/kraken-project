use kraken::api::handler::api_keys::schema::{
    CreateApiKeyRequest, ListApiKeys, UpdateApiKeyRequest,
};
use uuid::Uuid;

use crate::sdk::utils::KrakenRequest;
use crate::{KrakenClient, KrakenResult};

impl KrakenClient {
    /// Retrieve all api keys
    pub async fn get_all_api_keys(&self) -> KrakenResult<ListApiKeys> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/apiKeys").expect("Valid url");

        self.make_request(KrakenRequest::get(url).build()).await
    }

    /// Create a new api key
    pub async fn create_api_key(&self, name: String) -> KrakenResult<Uuid> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/apiKeys").expect("Valid url");

        self.make_request(
            KrakenRequest::post(url)
                .body(CreateApiKeyRequest { name })
                .build(),
        )
        .await
    }

    /// Update an api key
    pub async fn update_api_key(&self, key: Uuid, name: String) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/apiKeys/{key}"))
            .expect("Valid url");

        self.make_request(
            KrakenRequest::put(url)
                .body(UpdateApiKeyRequest { name })
                .build(),
        )
        .await?;

        Ok(())
    }

    /// Delete an api key
    pub async fn delete_api_key(&self, key: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/apiKeys/{key}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::delete(url).build())
            .await?;

        Ok(())
    }
}
