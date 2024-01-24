use kraken::api::handler::attacks::schema::{
    BruteforceSubdomainsRequest, DnsResolutionRequest, DnsTxtScanRequest, HostsAliveRequest,
    ListAttacks, QueryCertificateTransparencyRequest, ScanTcpPortsRequest, ServiceDetectionRequest,
    SimpleAttack, UdpServiceDetectionRequest,
};
use kraken::api::handler::common::schema::UuidResponse;
use serde::Serialize;
use uuid::Uuid;

use crate::sdk::utils::KrakenRequest;
use crate::{KrakenClient, KrakenResult};

impl KrakenClient {
    /// Get all attacks the user has access to
    pub async fn get_all_attacks(&self) -> KrakenResult<ListAttacks> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/attacks").expect("Valid url");

        self.make_request(KrakenRequest::get(url).build()).await
    }

    /// Get all attacks in a specific workspace
    pub async fn get_all_workspace_attacks(&self, workspace: Uuid) -> KrakenResult<ListAttacks> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/workspaces/{workspace}/attacks"))
            .expect("Valid url");

        self.make_request(KrakenRequest::get(url).build()).await
    }

    /// Retrieve a single attack
    pub async fn get_attack(&self, attack: Uuid) -> KrakenResult<SimpleAttack> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/attacks/{attack}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::get(url).build()).await
    }

    /// Delete an attack
    pub async fn delete_attack(&self, attack: Uuid) -> KrakenResult<()> {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/attacks/{attack}"))
            .expect("Valid url");

        self.make_request(KrakenRequest::delete(url).build())
            .await?;

        Ok(())
    }

    async fn start_attack<REQ>(&self, attack: &str, req: REQ) -> KrakenResult<Uuid>
    where
        REQ: Serialize,
    {
        #[allow(clippy::expect_used)]
        let url = self
            .base_url
            .join(&format!("api/v1/attacks/{attack}"))
            .expect("Valid url");

        let uuid: UuidResponse = self
            .make_request(KrakenRequest::post(url).body(req).build())
            .await?;

        Ok(uuid.uuid)
    }

    /// Start the Bruteforce Subdomains attack
    pub async fn attack_bruteforce_subdomains(
        &self,
        req: BruteforceSubdomainsRequest,
    ) -> KrakenResult<Uuid> {
        self.start_attack("bruteforceSubdomains", req).await
    }

    /// Start the DNS resolution attack
    pub async fn attack_dns_resolution(&self, req: DnsResolutionRequest) -> KrakenResult<Uuid> {
        self.start_attack("dnsResolution", req).await
    }

    /// Start the hosts alive attack
    pub async fn attack_hosts_alive(&self, req: HostsAliveRequest) -> KrakenResult<Uuid> {
        self.start_attack("hostsAlive", req).await
    }

    /// Start the certificate transparency attack
    pub async fn attack_certificate_transparency(
        &self,
        req: QueryCertificateTransparencyRequest,
    ) -> KrakenResult<Uuid> {
        self.start_attack("queryCertificateTransparency", req).await
    }

    /// Start the tcp portscan attack
    pub async fn attack_tcp_portscan(&self, req: ScanTcpPortsRequest) -> KrakenResult<Uuid> {
        self.start_attack("scanTcpPorts", req).await
    }

    /// Start the service detection attack
    pub async fn attack_service_detection(
        &self,
        req: ServiceDetectionRequest,
    ) -> KrakenResult<Uuid> {
        self.start_attack("serviceDetection", req).await
    }

    /// Start the tcp portscan attack
    pub async fn attack_udp_service_detection(
        &self,
        req: UdpServiceDetectionRequest,
    ) -> KrakenResult<Uuid> {
        self.start_attack("udpServiceDetection", req).await
    }

    ///
    pub async fn attack_dns_txt_scan(&self, req: DnsTxtScanRequest) -> KrakenResult<Uuid> {
        self.start_attack("dnsTxtScan", req).await
    }
}
