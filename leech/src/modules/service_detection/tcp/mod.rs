mod detection;
mod oneshot;
mod scanner;

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::ops::RangeInclusive;
use std::time::Duration;

use futures::StreamExt;
use ipnetwork::IpNetwork;
use itertools::Itertools;
use log::{info, warn};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;

use self::detection::detect_service;
pub use self::oneshot::OneShotTcpSettings;
use self::scanner::is_port_open;
use crate::modules::host_alive::icmp_scan::{start_icmp_scan, IcmpScanSettings};
use crate::modules::service_detection::error::ResultExt;
use crate::modules::service_detection::{DynError, DynResult, Service};
use crate::utils::IteratorExt;

/// Settings for a service detection
#[derive(Clone, Debug)]
pub struct TcpServiceDetectionSettings {
    /// Ip addresses / networks to scan
    pub addresses: Vec<IpNetwork>,

    /// The port ranges to scan
    pub ports: Vec<RangeInclusive<u16>>,

    /// The duration to wait for a response
    pub timeout: Duration,

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
}

#[derive(Debug, Clone)]
pub struct TcpServiceDetectionResult {
    pub addr: SocketAddr,
    pub service: Service,
}

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
    iter_addresses
        .cartesian_product(iter_ports)
        .try_for_each_concurrent(Some(limit), |(ip, port)| async move {
            let socket = SocketAddr::new(ip, port);
            if is_port_open(
                socket,
                settings.timeout,
                settings.max_retries,
                settings.retry_interval,
            )
            .await
            {
                let service = detect_service(socket, settings.timeout).await?;
                tx.send(TcpServiceDetectionResult {
                    addr: socket,
                    service,
                })
                .await?;
            }
            Ok::<(), DynError>(())
        })
        .await?;

    Ok(())
}
