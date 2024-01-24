use std::net::IpAddr;
use std::num::NonZeroU16;

use ipnetwork::IpNetwork;
use kraken::api::handler::common::schema::{PageParams, ServiceResultsPage, UuidResponse};
use kraken::api::handler::services::schema::{
    CreateServiceRequest, FullService, GetAllServicesQuery, ServiceRelations, UpdateServiceRequest,
};
use kraken::models::{ManualServiceCertainty, PortProtocol};
use uuid::Uuid;

use crate::sdk::utils::KrakenRequest;
use crate::{KrakenClient, KrakenResult};

impl KrakenClient {
    /// Add a service manually to kraken
    pub async fn add_service(
        &self,
        workspace: Uuid,
        name: String,
        certainty: ManualServiceCertainty,
        ip_addr: IpAddr,
        port: Option<(NonZeroU16, PortProtocol)>,
    ) -> KrakenResult<Uuid> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/services"))
            .expect("Valid Url");

        let uuid: UuidResponse = self
            .make_request(
                KrakenRequest::post(url)
                    .body(CreateServiceRequest {
                        name,
                        certainty,
                        host: IpNetwork::from(ip_addr),
                        port: port.map(|x| x.0.get()),
                        protocol: port.map(|x| x.1),
                    })
                    .build(),
            )
            .await?;

        Ok(uuid.uuid)
    }

    /// Retrieve all services of a workspace
    pub async fn get_all_services(
        &self,
        workspace: Uuid,
        page: PageParams,
    ) -> KrakenResult<ServiceResultsPage> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/services/all"))
            .expect("Valid Url");

        self.make_request(
            KrakenRequest::post(url)
                .body(GetAllServicesQuery {
                    page,
                    host: None,
                    global_filter: None,
                    service_filter: None,
                })
                .build(),
        )
        .await
    }

    /// Get a single service
    pub async fn get_service(&self, workspace: Uuid, service: Uuid) -> KrakenResult<FullService> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/services/{service}"))
            .expect("Valid Url");

        self.make_request(KrakenRequest::get(url).build()).await
    }

    /// Update a service
    ///
    /// At least one of the Options must be set in `update`
    pub async fn update_service(
        &self,
        workspace: Uuid,
        service: Uuid,
        update: UpdateServiceRequest,
    ) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/services/{service}"))
            .expect("Valid Url");

        self.make_request(KrakenRequest::put(url).body(update).build())
            .await
    }

    /// Delete a service
    pub async fn delete_service(&self, workspace: Uuid, service: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/services/{service}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::delete(url).build())
            .await?;

        Ok(())
    }

    /// List all direct relations to the service
    pub async fn get_service_relations(
        &self,
        workspace: Uuid,
        service: Uuid,
    ) -> KrakenResult<ServiceRelations> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!(
                "api/v1/workspaces/{workspace}/services/{service}/relations"
            ))
            .expect("Valid Url");

        self.make_request(KrakenRequest::get(url).build()).await
    }
}
