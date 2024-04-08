//! The aggregator that inserts and updates aggregates models

use ipnetwork::IpNetwork;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::api::handler::services::schema::ServiceProtocols;
use crate::models::DomainCertainty;
use crate::models::HostCertainty;
use crate::models::OsType;
use crate::models::PortCertainty;
use crate::models::PortProtocol;
use crate::models::ServiceCertainty;

mod domain;
mod host;
mod http_service;
mod port;
mod service;

struct DomainAggregationData {
    workspace: Uuid,
    domain: String,
    certainty: DomainCertainty,
    user: Uuid,
}
struct HostAggregationData {
    workspace: Uuid,
    ip_addr: IpNetwork,
    certainty: HostCertainty,
    os_type: Option<OsType>,
}
struct PortAggregationData {
    workspace: Uuid,
    host: Uuid,
    port: u16,
    protocol: PortProtocol,
    certainty: PortCertainty,
}
struct ServiceAggregationData {
    workspace: Uuid,
    host: Uuid,
    port: Option<Uuid>,
    protocols: Option<ServiceProtocols>,
    name: String,
    certainty: ServiceCertainty,
}
struct HttpServiceAggregationData {
    workspace: Uuid,
    name: String,
    host: Uuid,
    port: Uuid,
    domain: Option<Uuid>,
    base_path: String,
    tls: bool,
    sni_required: bool,
}

/// This is a facade to only allow one instance writing to the database per aggregation model
///
/// On construction, 4 tasks will be spawned:
/// - domains
/// - hosts
/// - ports
/// - services
///
/// by using the methods of this struct, you can push updates (or inserts) to this workers
/// that will execute them in order and than return the corresponding `UUID`.
pub struct Aggregator {
    domain: mpsc::Sender<(
        DomainAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
    host: mpsc::Sender<(
        HostAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
    port: mpsc::Sender<(
        PortAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
    service: mpsc::Sender<(
        ServiceAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
    http_service: mpsc::Sender<(
        HttpServiceAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
}

impl Default for Aggregator {
    /// Create a new instance of the aggregator
    ///
    /// This will initiate 4 tasks that will handle all incoming requests for the
    fn default() -> Self {
        let (domain_tx, domain_rx) = mpsc::channel(1);
        let (host_tx, host_rx) = mpsc::channel(1);
        let (port_tx, port_rx) = mpsc::channel(1);
        let (service_tx, service_rx) = mpsc::channel(1);
        let (http_service_tx, http_service_rx) = mpsc::channel(1);

        tokio::spawn(domain::run_domain_aggregator(domain_rx));
        tokio::spawn(host::run_host_aggregator(host_rx));
        tokio::spawn(port::run_port_aggregator(port_rx));
        tokio::spawn(service::run_service_aggregator(service_rx));
        tokio::spawn(http_service::run_http_service_aggregator(http_service_rx));

        Self {
            domain: domain_tx,
            host: host_tx,
            port: port_tx,
            service: service_tx,
            http_service: http_service_tx,
        }
    }
}

impl Aggregator {
    /// Insert an aggregated domain if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    ///
    /// The `user` is required to start dns resolution attacks implicitly.
    pub async fn aggregate_domain(
        &self,
        workspace: Uuid,
        domain: &str,
        certainty: DomainCertainty,
        user: Uuid,
    ) -> Result<Uuid, rorm::Error> {
        let (tx, rx) = oneshot::channel();

        // If we can't send to the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        self
            .domain
            .send((
                DomainAggregationData {
                    workspace,
                    user,
                    certainty,
                    domain: domain.to_string(),
                },
                tx,
            ))
            .await
            .expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        // If we can't receive the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        let aggregation_result = rx.await.expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        aggregation_result
    }

    /// Insert an aggregated service if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate_service(
        &self,
        workspace: Uuid,
        host: Uuid,
        port: Option<Uuid>,
        protocols: Option<ServiceProtocols>,
        name: &str,
        certainty: ServiceCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let (tx, rx) = oneshot::channel();

        // If we can't send to the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        self
            .service
            .send((
                ServiceAggregationData {
                    workspace,
                    host,
                    port,
                    protocols,
                    name: name.to_string(),
                    certainty,
                },
                tx,
            ))
            .await
            .expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        // If we can't receive the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        let aggregation_result = rx.await.expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        aggregation_result
    }

    /// Insert an aggregated host if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate_host(
        &self,
        workspace: Uuid,
        ip_addr: IpNetwork,
        certainty: HostCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let (tx, rx) = oneshot::channel();

        // If we can't send to the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        self
            .host
            .send((
                HostAggregationData {
                    workspace,
                    certainty,
                    ip_addr,
                    os_type: None
                },
                tx,
            ))
            .await
            .expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        // If we can't receive the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        let aggregation_result = rx.await.expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        aggregation_result
    }

    /// Insert an aggregated host if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate_host_os(
        &self,
        workspace: Uuid,
        ip_addr: IpNetwork,
        certainty: HostCertainty,
        os_type: OsType,
    ) -> Result<Uuid, rorm::Error> {
        let (tx, rx) = oneshot::channel();

        // If we can't send to the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        self
            .host
            .send((
                HostAggregationData {
                    workspace,
                    certainty,
                    ip_addr,
                    os_type: Some(os_type)
                },
                tx,
            ))
            .await
            .expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        // If we can't receive the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        let aggregation_result = rx.await.expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        aggregation_result
    }

    /// Insert an aggregated port if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate_port(
        &self,
        workspace: Uuid,
        host: Uuid,
        port: u16,
        protocol: PortProtocol,
        certainty: PortCertainty,
    ) -> Result<Uuid, rorm::Error> {
        let (tx, rx) = oneshot::channel();

        // If we can't send to the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        self
            .port
            .send((
                PortAggregationData {
                    workspace,
                    host,
                    port,
                    protocol,
                    certainty,
                },
                tx,
            ))
            .await
            .expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        // If we can't receive the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        let aggregation_result = rx.await.expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        aggregation_result
    }

    /// Insert an aggregated http service if it doesn't exist yet or
    /// update it if its information is not as precise
    /// and return its primary key.
    pub async fn aggregate_http_service(
        &self,
        workspace: Uuid,
        name: String,
        host: Uuid,
        port: Uuid,
        domain: Option<Uuid>,
        base_path: String,
        tls: bool,
        sni_required: bool,
    ) -> Result<Uuid, rorm::Error> {
        let (tx, rx) = oneshot::channel();

        // If we can't send to the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        self
            .http_service
            .send((
                HttpServiceAggregationData {
                    workspace,
                    name,
                    host,
                    port,
                    domain,
                    base_path,
                    tls,
                    sni_required,
                },
                tx,
            ))
            .await
            .expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        // If we can't receive the channel, somethings really messed up
        #[allow(clippy::expect_used)]
        let aggregation_result = rx.await.expect("This should never fail, if you ever encounter this error, please open an issue with the stacktrace.");

        aggregation_result
    }
}
