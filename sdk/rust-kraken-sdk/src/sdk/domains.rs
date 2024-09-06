use kraken::api::handler::common::schema::DomainResultsPage;
use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::domains::schema::CreateDomainRequest;
use kraken::api::handler::domains::schema::DomainRelations;
use kraken::api::handler::domains::schema::FullDomain;
use kraken::api::handler::domains::schema::GetAllDomainsQuery;
use kraken::api::handler::domains::schema::UpdateDomainRequest;
use kraken::api::handler::findings::schema::ListFindings;
use kraken::api::handler::findings::schema::SimpleFinding;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Manually add a domain
    pub async fn add_domain(&self, workspace: Uuid, domain: String) -> KrakenResult<Uuid> {
        let uuid: UuidResponse = self
            .post(&format!("api/v1/workspaces/{workspace}/domains"))
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
        self.post(&format!("api/v1/workspaces/{workspace}/domains/all"))
            .body(query)
            .send()
            .await
    }

    /// Retrieve a specific domain
    pub async fn get_domain(&self, workspace: Uuid, domain: Uuid) -> KrakenResult<FullDomain> {
        self.get(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .send()
            .await
    }

    /// Update a domain
    pub async fn update_domain(
        &self,
        workspace: Uuid,
        domain: Uuid,
        update: UpdateDomainRequest,
    ) -> KrakenResult<()> {
        self.put(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .body(update)
            .send()
            .await
    }

    /// Delete a domain
    pub async fn delete_domain(&self, workspace: Uuid, domain: Uuid) -> KrakenResult<()> {
        self.delete(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .send::<()>()
            .await?;

        Ok(())
    }

    /// Get all relations for a domain
    pub async fn get_domain_relations(
        &self,
        workspace: Uuid,
        domain: Uuid,
    ) -> KrakenResult<DomainRelations> {
        self.get(&format!(
            "api/v1/workspaces/{workspace}/domains/{domain}/relations"
        ))
        .send()
        .await
    }

    /// List all findings affecting the domain
    pub async fn get_domain_findings(
        &self,
        workspace: Uuid,
        domain: Uuid,
    ) -> KrakenResult<Vec<SimpleFinding>> {
        let list: ListFindings = self
            .get(&format!(
                "api/v1/workspaces/{workspace}/domains/{domain}/findings"
            ))
            .send()
            .await?;
        Ok(list.findings)
    }
}
