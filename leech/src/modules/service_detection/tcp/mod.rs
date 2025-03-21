use std::error::Error;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use ipnetwork::IpNetwork;
use itertools::Itertools;
use kraken_proto::any_attack_response;
use kraken_proto::any_attack_response::Response;
use kraken_proto::push_attack_request;
use kraken_proto::shared;
use kraken_proto::RepeatedServiceDetectionResponse;
use kraken_proto::ServiceCertainty;
use kraken_proto::ServiceDetectionRequest;
use kraken_proto::ServiceDetectionResponse;
use log::info;
use log::warn;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

use self::detection::detect_service;
pub use self::oneshot::OneShotTcpSettings;
pub use self::oneshot::ProbeTcpResult;
pub use self::oneshot::ProbeTlsResult;
use self::scanner::is_port_open;
use crate::modules::host_alive::icmp_scan::start_icmp_scan;
use crate::modules::host_alive::icmp_scan::IcmpScanSettings;
use crate::modules::service_detection::error::ResultExt;
use crate::modules::service_detection::DynError;
use crate::modules::service_detection::DynResult;
use crate::modules::service_detection::Service;
use crate::modules::StreamedAttack;
use crate::utils::IteratorExt;

mod detection;
mod oneshot;
mod scanner;

/// Attack scanning for open tcp ports and detecting their services
pub struct TcpServiceDetection;
#[tonic::async_trait]
impl StreamedAttack for TcpServiceDetection {
    type Settings = TcpServiceDetectionSettings;
    type Output = TcpServiceDetectionResult;
    // thanks std
    // Arc<dyn Error> does implement Error while Box<dyn Error> doesn't
    type Error = Arc<dyn Error + Send + Sync + 'static>;

    async fn execute(
        settings: Self::Settings,
        sender: Sender<Self::Output>,
    ) -> Result<(), Self::Error> {
        start_tcp_service_detection(settings, sender)
            .await
            .map_err(From::from)
    }

    type Request = ServiceDetectionRequest;

    fn get_attack_uuid(request: &Self::Request) -> &str {
        &request.attack_uuid
    }

    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status> {
        let mut ports = request
            .ports
            .into_iter()
            .map(RangeInclusive::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if ports.is_empty() {
            ports.push(1..=u16::MAX);
        }

        let concurrent_limit = NonZeroU32::new(request.concurrent_limit)
            .ok_or_else(|| Status::invalid_argument("concurrent_limit can't be zero"))?;

        Ok(TcpServiceDetectionSettings {
            addresses: request
                .targets
                .into_iter()
                .map(IpNetwork::try_from)
                .collect::<Result<_, _>>()?,
            ports,
            connect_timeout: Duration::from_millis(request.connect_timeout),
            receive_timeout: Duration::from_millis(request.receive_timeout),
            max_retries: request.max_retries,
            retry_interval: Duration::from_millis(request.retry_interval),
            concurrent_limit,
            skip_icmp_check: request.skip_icmp_check,
            just_scan: false,
        })
    }

    type Response = ServiceDetectionResponse;
    fn encode_output(
        TcpServiceDetectionResult {
            tls_service,
            tcp_service,
            addr,
        }: Self::Output,
    ) -> Self::Response {
        let mut response = ServiceDetectionResponse {
            address: Some(shared::Address::from(addr.ip())),
            port: addr.port() as u32,
            // The following are updated in the 2 match statements below
            is_tls: true,
            tcp_certainty: ServiceCertainty::Unknown as _,
            tcp_services: Vec::new(),
            tls_certainty: ServiceCertainty::Unknown as _,
            tls_services: Vec::new(),
        };
        match tcp_service {
            Service::Unknown => (),
            Service::Maybe(services) => {
                response.tcp_certainty = ServiceCertainty::Maybe as _;
                response.tcp_services = services.into_iter().map(str::to_string).collect();
            }
            Service::Definitely(service) => {
                response.tcp_certainty = ServiceCertainty::Definitely as _;
                response.tcp_services = vec![service.to_string()];
            }
        }
        match tls_service {
            None => {
                response.is_tls = false;
            }
            Some(Service::Unknown) => (),
            Some(Service::Maybe(services)) => {
                response.tls_certainty = ServiceCertainty::Maybe as _;
                response.tls_services = services.into_iter().map(str::to_string).collect();
            }
            Some(Service::Definitely(service)) => {
                response.tls_certainty = ServiceCertainty::Definitely as _;
                response.tls_services = vec![service.to_string()];
            }
        }
        response
    }

    fn print_output(output: &Self::Output) {
        info!("Open port found: {}", output.addr,);
        info!("It's running: {:?} (TCP)", output.tcp_service);
        info!("It's running: {:?} (TLS over TCP)", output.tls_service);
    }

    fn wrap_for_backlog(response: Self::Response) -> Response {
        any_attack_response::Response::ServiceDetection(response)
    }

    fn wrap_for_push(responses: Vec<Self::Response>) -> push_attack_request::Response {
        push_attack_request::Response::ServiceDetection(RepeatedServiceDetectionResponse {
            responses,
        })
    }
}

/// Settings for a service detection
#[derive(Clone, Debug)]
pub struct TcpServiceDetectionSettings {
    /// Ip addresses / networks to scan
    pub addresses: Vec<IpNetwork>,

    /// The port ranges to scan
    pub ports: Vec<RangeInclusive<u16>>,

    /// The duration to wait until a connection is considered failed.
    pub connect_timeout: Duration,

    /// The duration to wait when receiving the service's response during detection.
    pub receive_timeout: Duration,

    /// Defines how many times a connection should be retried if it failed the last time
    pub max_retries: u32,

    /// The interval to wait in between the retries
    pub retry_interval: Duration,

    /// Maximum of concurrent tasks that should be spawned
    pub concurrent_limit: NonZeroU32,

    /// If set to true, there won't be an initial icmp check.
    ///
    /// All hosts are assumed to be reachable.
    pub skip_icmp_check: bool,

    /// Just runs the initial port scanner without the service detection
    pub just_scan: bool,
}

/// A found open port and the potentially detected service
#[derive(Debug, Serialize, Clone)]
pub struct TcpServiceDetectionResult {
    /// The socket address found to be open
    pub addr: SocketAddr,

    /// The potentially detected tcp service
    ///
    /// This is not optional, because if the port it not speaking TCP at all,
    /// then what is the point of a result?
    /// Note it might be `Service::Unknown`.
    pub tcp_service: Service,

    /// The potentially detected tls service
    ///
    /// This is optional, because the TCP port might not speak TLS.
    pub tls_service: Option<Service>,
}

/// Scan for open tcp ports and detect the service running on them
/// by recognizing their banner and their responses to certain payloads.
pub async fn start_tcp_service_detection(
    settings: TcpServiceDetectionSettings,
    tx: Sender<TcpServiceDetectionResult>,
) -> DynResult<()> {
    // Increase the NO_FILE limit if necessary
    rlimit::increase_nofile_limit(100_000).context("rlimit::increase_nofile_limit")?;

    let addresses = if settings.skip_icmp_check {
        info!("Skipping icmp check");
        settings.addresses
    } else {
        let (tx, rx) = mpsc::channel(1);
        let icmp_settings = IcmpScanSettings {
            addresses: settings.addresses,
            timeout: Duration::from_millis(1000),
            concurrent_limit: settings.concurrent_limit.get(),
        };
        let icmp_scan = tokio::spawn(start_icmp_scan(icmp_settings, tx));
        let addresses: Vec<_> = ReceiverStream::new(rx).map(IpNetwork::from).collect().await;
        icmp_scan.await??;

        if addresses.is_empty() {
            warn!("All hosts are unreachable. Check your targets or disable the icmp check.");
        }

        addresses
    };

    let iter_addresses = addresses
        .iter()
        .cloned()
        .flat_map(|network| network.into_iter());
    let iter_ports = settings.ports.iter().cloned().flatten();
    #[allow(clippy::expect_used)] // The leech will never run on a 16bit machine
    let limit = settings
        .concurrent_limit
        .try_into()
        .expect("u32 should be convertible to usize");
    iter_ports
        .cartesian_product(iter_addresses)
        .try_for_each_concurrent(Some(limit), |(port, ip)| async move {
            let socket = SocketAddr::new(ip, port);
            if is_port_open(
                socket,
                settings.connect_timeout,
                settings.max_retries,
                settings.retry_interval,
            )
            .await
            {
                if settings.just_scan {
                    tx.send(TcpServiceDetectionResult {
                        addr: socket,
                        tcp_service: Service::Unknown,
                        tls_service: None,
                    })
                    .await?;
                } else {
                    tx.send(
                        detect_service(socket, settings.receive_timeout, settings.connect_timeout)
                            .await,
                    )
                    .await?;
                };
            }
            Ok::<(), DynError>(())
        })
        .await?;

    Ok(())
}
