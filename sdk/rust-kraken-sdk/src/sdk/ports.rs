use std::net::IpAddr;
use std::num::NonZeroU16;

use ipnetwork::IpNetwork;
use kraken::api::handler::common::schema::{PortResultsPage, UuidResponse};
use kraken::api::handler::ports::schema::{
    CreatePortRequest, FullPort, PortRelations, UpdatePortRequest,
};
use kraken::models::{ManualPortCertainty, PortProtocol};
use uuid::Uuid;

use crate::sdk::utils::KrakenRequest;
use crate::{KrakenClient, KrakenResult};

impl KrakenClient {
    /// Add a port manually to kraken
    pub async fn add_port(
        &self,
        workspace: Uuid,
        ip_addr: IpAddr,
        port: NonZeroU16,
        protocol: PortProtocol,
        certainty: ManualPortCertainty,
    ) -> KrakenResult<Uuid> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/ports"))
            .expect("Valid url");

        let uuid: UuidResponse = self
            .make_request(
                KrakenRequest::post(url)
                    .body(CreatePortRequest {
                        ip_addr: IpNetwork::from(ip_addr),
                        port: port.get(),
                        certainty,
                        protocol,
                    })
                    .build(),
            )
            .await?;

        Ok(uuid.uuid)
    }

    /// Get all ports of a workspace
    pub async fn get_all_ports(&self, workspace: Uuid) -> KrakenResult<PortResultsPage> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/ports/all"))
            .expect("Valid Url");

        self.make_request(KrakenRequest::post(url).build()).await
    }

    /// Get all information about a single port
    pub async fn get_port(&self, workspace: Uuid, port: Uuid) -> KrakenResult<FullPort> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/ports/{port}"))
            .expect("Valid Url");

        self.make_request(KrakenRequest::get(url).build()).await
    }

    /// Update a port
    ///
    /// There must be at least one `update`
    pub async fn update_port(
        &self,
        workspace: Uuid,
        port: Uuid,
        update: UpdatePortRequest,
    ) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/ports/{port}"))
            .expect("Valid Url");

        self.make_request(KrakenRequest::put(url).body(update).build())
            .await?;

        Ok(())
    }

    /// Delete a port
    pub async fn delete_port(&self, workspace: Uuid, port: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/ports/{port}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::delete(url).build())
            .await?;

        Ok(())
    }

    /// Retrieve all direct relations of a port
    pub async fn get_port_relations(
        &self,
        workspace: Uuid,
        port: Uuid,
    ) -> KrakenResult<PortRelations> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!(
                "api/v1/workspaces/{workspace}/ports/{port}/relations"
            ))
            .expect("Valid Url");

        self.make_request(KrakenRequest::get(url).build()).await
    }
}
