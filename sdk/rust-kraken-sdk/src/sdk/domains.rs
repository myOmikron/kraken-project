use kraken::api::handler::common::schema::DomainResultsPage;
use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::domains::schema::CreateDomainRequest;
use kraken::api::handler::domains::schema::DomainRelations;
use kraken::api::handler::domains::schema::FullDomain;
use kraken::api::handler::domains::schema::GetAllDomainsQuery;
use kraken::api::handler::domains::schema::UpdateDomainRequest;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Manually add a domain
    pub async fn add_domain(&self, workspace: Uuid, domain: String) -> KrakenResult<Uuid> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains"))
            .expect("Valid url");

        let uuid: UuidResponse = self
            .post(url)
            .body(CreateDomainRequest { domain })
            .send()
            .await?;

        Ok(uuid.uuid)
    }

    /// Retrieve a page of all domains of a workspace
    pub async fn get_all_domains(
        &self,
        workspace: Uuid,
        query: GetAllDomainsQuery,
    ) -> KrakenResult<DomainResultsPage> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains/all"))
            .expect("valid url");

        self.post(url).body(query).send().await
    }

    /// Retrieve a specific domain
    pub async fn get_domain(&self, workspace: Uuid, domain: Uuid) -> KrakenResult<FullDomain> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .expect("Valid url");

        self.get(url).send().await
    }

    /// Update a domain
    pub async fn update_domain(
        &self,
        workspace: Uuid,
        domain: Uuid,
        update: UpdateDomainRequest,
    ) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .expect("Valid url");

        self.put(url).body(update).send().await
    }

    /// Delete a domain
    pub async fn delete_domain(&self, workspace: Uuid, domain: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .expect("Valid url");

        self.delete(url).send().await?;

        Ok(())
    }

    /// Get all relations for a domain
    pub async fn get_domain_relations(
        &self,
        workspace: Uuid,
        domain: Uuid,
    ) -> KrakenResult<DomainRelations> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!(
                "api/v1/workspaces/{workspace}/domain/{domain}/relations"
            ))
            .expect("Valid url");

        self.get(url).send().await
    }
}
