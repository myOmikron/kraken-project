use kraken::api::handler::common::schema::HttpServiceResultsPage;
use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::findings::schema::ListFindings;
use kraken::api::handler::findings::schema::SimpleFinding;
use kraken::api::handler::http_services::schema::CreateHttpServiceRequest;
use kraken::api::handler::http_services::schema::FullHttpService;
use kraken::api::handler::http_services::schema::GetAllHttpServicesQuery;
use kraken::api::handler::http_services::schema::HttpServiceRelations;
use kraken::api::handler::http_services::schema::UpdateHttpServiceRequest;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Add a http service manually to kraken
    pub async fn add_http_service(
        &self,
        workspace: Uuid,
        create: CreateHttpServiceRequest,
    ) -> KrakenResult<Uuid> {
        let uuid: UuidResponse = self
            .post(&format!("api/v1/workspaces/{workspace}/httpServices"))
            .body(create)
            .send()
            .await?;

        Ok(uuid.uuid)
    }

    /// Retrieve all http services of a workspace
    pub async fn get_all_http_services(
        &self,
        workspace: Uuid,
        query: GetAllHttpServicesQuery,
    ) -> KrakenResult<HttpServiceResultsPage> {
        self.post(&format!("api/v1/workspaces/{workspace}/httpServices/all"))
            .body(query)
            .send()
            .await
    }

    /// Get a single http service
    pub async fn get_http_service(
        &self,
        workspace: Uuid,
        http_service: Uuid,
    ) -> KrakenResult<FullHttpService> {
        self.get(&format!(
            "api/v1/workspaces/{workspace}/httpServices/{http_service}"
        ))
        .send()
        .await
    }

    /// Update a http service
    ///
    /// At least one of the Options must be set in `update`
    pub async fn update_http_service(
        &self,
        workspace: Uuid,
        http_service: Uuid,
        update: UpdateHttpServiceRequest,
    ) -> KrakenResult<()> {
        self.put(&format!(
            "api/v1/workspaces/{workspace}/httpServices/{http_service}"
        ))
        .body(update)
        .send()
        .await
    }

    /// Delete a service
    pub async fn delete_http_service(
        &self,
        workspace: Uuid,
        http_service: Uuid,
    ) -> KrakenResult<()> {
        self.delete(&format!(
            "api/v1/workspaces/{workspace}/httpServices/{http_service}"
        ))
        .send()
        .await
    }

    /// List all direct relations to the service
    pub async fn get_http_service_relations(
        &self,
        workspace: Uuid,
        http_service: Uuid,
    ) -> KrakenResult<HttpServiceRelations> {
        self.get(&format!(
            "api/v1/workspaces/{workspace}/httpServices/{http_service}/relations"
        ))
        .send()
        .await
    }

    /// List all findings affecting the service
    pub async fn get_http_service_findings(
        &self,
        workspace: Uuid,
        http_service: Uuid,
    ) -> KrakenResult<Vec<SimpleFinding>> {
        let list: ListFindings = self
            .get(&format!(
                "api/v1/workspaces/{workspace}/httpServices/{http_service}/findings"
            ))
            .send()
            .await?;
        Ok(list.findings)
    }
}
