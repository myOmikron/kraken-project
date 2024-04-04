use std::net::IpAddr;
use std::num::NonZeroU16;

use ipnetwork::IpNetwork;
use kraken::api::handler::common::schema::PortResultsPage;
use kraken::api::handler::common::schema::UuidResponse;
use kraken::api::handler::findings::schema::ListFindings;
use kraken::api::handler::findings::schema::SimpleFinding;
use kraken::api::handler::ports::schema::CreatePortRequest;
use kraken::api::handler::ports::schema::FullPort;
use kraken::api::handler::ports::schema::GetAllPortsQuery;
use kraken::api::handler::ports::schema::PortRelations;
use kraken::api::handler::ports::schema::UpdatePortRequest;
use kraken::models::ManualPortCertainty;
use kraken::models::PortProtocol;
use uuid::Uuid;

use crate::KrakenClient;
use crate::KrakenResult;

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
        let uuid: UuidResponse = self
            .post(&format!("api/v1/workspaces/{workspace}/ports"))
            .body(CreatePortRequest {
                ip_addr: IpNetwork::from(ip_addr),
                port: port.get(),
                certainty,
                protocol,
            })
            .send()
            .await?;

        Ok(uuid.uuid)
    }

    /// Get all ports of a workspace
    pub async fn get_all_ports(
        &self,
        workspace: Uuid,
        query: GetAllPortsQuery,
    ) -> KrakenResult<PortResultsPage> {
        self.post(&format!("api/v1/workspaces/{workspace}/ports/all"))
            .body(query)
            .send()
            .await
    }

    /// Get all information about a single port
    pub async fn get_port(&self, workspace: Uuid, port: Uuid) -> KrakenResult<FullPort> {
        self.get(&format!("api/v1/workspaces/{workspace}/ports/{port}"))
            .send()
            .await
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
        self.put(&format!("api/v1/workspaces/{workspace}/ports/{port}"))
            .body(update)
            .send()
            .await
    }

    /// Delete a port
    pub async fn delete_port(&self, workspace: Uuid, port: Uuid) -> KrakenResult<()> {
        self.delete(&format!("api/v1/workspaces/{workspace}/ports/{port}"))
            .send()
            .await
    }

    /// Retrieve all direct relations of a port
    pub async fn get_port_relations(
        &self,
        workspace: Uuid,
        port: Uuid,
    ) -> KrakenResult<PortRelations> {
        self.get(&format!(
            "api/v1/workspaces/{workspace}/ports/{port}/relations"
        ))
        .send()
        .await
    }

    /// List all findings affecting the port
    pub async fn get_port_findings(
        &self,
        workspace: Uuid,
        port: Uuid,
    ) -> KrakenResult<Vec<SimpleFinding>> {
        let list: ListFindings = self
            .get(&format!(
                "api/v1/workspaces/{workspace}/ports/{port}/findings"
            ))
            .send()
            .await?;
        Ok(list.findings)
    }
}
