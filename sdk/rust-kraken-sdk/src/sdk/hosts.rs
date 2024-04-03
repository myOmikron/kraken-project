use std::net::IpAddr;

use ipnetwork::IpNetwork;
use kraken::api::handler::common::schema::HostResultsPage;
use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::hosts::schema::CreateHostRequest;
use kraken::api::handler::hosts::schema::FullHost;
use kraken::api::handler::hosts::schema::GetAllHostsQuery;
use kraken::api::handler::hosts::schema::HostRelations;
use kraken::api::handler::hosts::schema::UpdateHostRequest;
use kraken::models::ManualHostCertainty;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Add a host
    pub async fn add_host(
        &self,
        workspace: Uuid,
        ip_addr: IpAddr,
        certainty: ManualHostCertainty,
    ) -> KrakenResult<Uuid> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/hosts"))
            .expect("Valid url");

        let uuid: UuidResponse = self
            .post(url)
            .body(CreateHostRequest {
                ip_addr: IpNetwork::from(ip_addr),
                certainty,
            })
            .send()
            .await?;

        Ok(uuid.uuid)
    }

    /// Get all hosts of a workspace
    pub async fn get_all_hosts(
        &self,
        workspace: Uuid,
        query: GetAllHostsQuery,
    ) -> KrakenResult<HostResultsPage> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/hosts/all"))
            .expect("Valid url");

        self.post(url).body(query).send().await
    }

    /// Retrieve a single host
    pub async fn get_host(&self, workspace: Uuid, host: Uuid) -> KrakenResult<FullHost> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/hosts/{host}"))
            .expect("Valid url");

        self.get(url).send().await
    }

    /// Update a host
    ///
    /// At least one field in `update` must be not None
    pub async fn update_host(
        &self,
        workspace: Uuid,
        host: Uuid,
        update: UpdateHostRequest,
    ) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/hosts/{host}"))
            .expect("Valid url");

        self.put(url).body(update).send().await
    }

    /// Delete a host
    pub async fn delete_host(&self, workspace: Uuid, host: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/hosts/{host}"))
            .expect("Valid url");

        self.delete(url).send().await
    }

    /// Get the direct relations of a host
    pub async fn get_host_relations(
        &self,
        workspace: Uuid,
        host: Uuid,
    ) -> KrakenResult<HostRelations> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!(
                "api/v1/workspaces/{workspace}/hosts/{host}/relations"
            ))
            .expect("Valid url");

        self.get(url).send().await
    }
}
