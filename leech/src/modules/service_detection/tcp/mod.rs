use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::ops::RangeInclusive;
use std::time::Duration;

use futures::StreamExt;
use ipnetwork::IpNetwork;
use itertools::Itertools;
use log::info;
use log::warn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;

use self::detection::detect_service;
pub use self::oneshot::OneShotTcpSettings;
use self::scanner::is_port_open;
use crate::modules::host_alive::icmp_scan::start_icmp_scan;
use crate::modules::host_alive::icmp_scan::IcmpScanSettings;
use crate::modules::service_detection::error::ResultExt;
use crate::modules::service_detection::DynError;
use crate::modules::service_detection::DynResult;
use crate::modules::service_detection::Service;
use crate::utils::IteratorExt;

mod detection;
mod oneshot;
mod scanner;

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
#[derive(Debug, Clone)]
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
                } else if let Some(result) =
                    detect_service(socket, settings.receive_timeout, settings.connect_timeout)
                        .await?
                {
                    tx.send(result).await?;
                };
            }
            Ok::<(), DynError>(())
        })
        .await?;

    Ok(())
}
