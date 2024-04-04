use std::net::IpAddr;
use std::num::NonZeroU16;

use ipnetwork::IpNetwork;
use kraken::api::handler::common::schema::ServiceResultsPage;
use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::services::schema::CreateServiceRequest;
use kraken::api::handler::services::schema::FullService;
use kraken::api::handler::services::schema::GetAllServicesQuery;
use kraken::api::handler::services::schema::ServiceRelations;
use kraken::api::handler::services::schema::UpdateServiceRequest;
use kraken::models::ManualServiceCertainty;
use kraken::models::ServiceProtocols;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Add a service manually to kraken
    pub async fn add_service(
        &self,
        workspace: Uuid,
        name: String,
        certainty: ManualServiceCertainty,
        ip_addr: IpAddr,
        port: Option<(NonZeroU16, ServiceProtocols)>,
    ) -> KrakenResult<Uuid> {
        let uuid: UuidResponse = self
            .post(&format!("api/v1/workspaces/{workspace}/services"))
            .body(CreateServiceRequest {
                name,
                certainty,
                host: IpNetwork::from(ip_addr),
                port: port.map(|x| x.0.get()),
                protocols: port.map(|x| x.1),
            })
            .send()
            .await?;

        Ok(uuid.uuid)
    }

    /// Retrieve all services of a workspace
    pub async fn get_all_services(
        &self,
        workspace: Uuid,
        query: GetAllServicesQuery,
    ) -> KrakenResult<ServiceResultsPage> {
        self.post(&format!("api/v1/workspaces/{workspace}/services/all"))
            .body(query)
            .send()
            .await
    }

    /// Get a single service
    pub async fn get_service(&self, workspace: Uuid, service: Uuid) -> KrakenResult<FullService> {
        self.get(&format!("api/v1/workspaces/{workspace}/services/{service}"))
            .send()
            .await
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
        self.put(&format!("api/v1/workspaces/{workspace}/services/{service}"))
            .body(update)
            .send()
            .await
    }

    /// Delete a service
    pub async fn delete_service(&self, workspace: Uuid, service: Uuid) -> KrakenResult<()> {
        self.delete(&format!("api/v1/workspaces/{workspace}/services/{service}"))
            .send()
            .await
    }

    /// List all direct relations to the service
    pub async fn get_service_relations(
        &self,
        workspace: Uuid,
        service: Uuid,
    ) -> KrakenResult<ServiceRelations> {
        self.get(&format!(
            "api/v1/workspaces/{workspace}/services/{service}/relations"
        ))
        .send()
        .await
    }
}
