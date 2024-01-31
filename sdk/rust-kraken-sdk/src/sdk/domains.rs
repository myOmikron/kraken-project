use kraken::api::handler::common::schema::{DomainResultsPage, PageParams, UuidResponse};
use kraken::api::handler::domains::schema::{
    CreateDomainRequest, DomainRelations, FullDomain, GetAllDomainsQuery, UpdateDomainRequest,
};
use uuid::Uuid;

use crate::sdk::utils::KrakenRequest;
use crate::{KrakenClient, KrakenResult};

impl KrakenClient {
    /// Manually add a domain
    pub async fn add_domain(&self, workspace: Uuid, domain: String) -> KrakenResult<Uuid> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains"))
            .expect("Valid url");

        let uuid: UuidResponse = self
            .make_request(
                KrakenRequest::post(url)
                    .body(CreateDomainRequest { domain })
                    .build(),
            )
            .await?;

        Ok(uuid.uuid)
    }

    /// Retrieve a page of all domains of a workspace
    pub async fn get_all_domains(
        &self,
        workspace: Uuid,
        page: PageParams,
    ) -> KrakenResult<DomainResultsPage> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains/all"))
            .expect("valid url");

        let domains = self
            .make_request(
                KrakenRequest::post(url)
                    .body(GetAllDomainsQuery {
                        page,
                        host: None,
                        domain_filter: None,
                        global_filter: None,
                    })
                    .build(),
            )
            .await?;

        Ok(domains)
    }

    /// Retrieve a specific domain
    pub async fn get_domain(&self, workspace: Uuid, domain: Uuid) -> KrakenResult<FullDomain> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::get(url).build()).await
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

        self.make_request(KrakenRequest::put(url).body(update).build())
            .await?;

        Ok(())
    }

    /// Delete a domain
    pub async fn delete_domain(&self, workspace: Uuid, domain: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/domains/{domain}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::delete(url).build())
            .await?;

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

        self.make_request(KrakenRequest::get(url).build()).await
    }
}
