//! This module holds a tcp connect port scanner

use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::time::Duration;

use futures::{stream, StreamExt};
use ipnetwork::IpNetwork;
use itertools::Itertools;
use log::{debug, info, trace, warn};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, timeout};
use tokio_stream::wrappers::ReceiverStream;

use crate::modules::host_alive::icmp_scan::{start_icmp_scan, IcmpScanSettings};
use crate::modules::port_scanner::error::TcpPortScanError;

/// The settings of a tcp connection port scan
#[derive(Clone, Debug)]
pub struct TcpPortScannerSettings {
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
    ///
    /// 0 means, that there should be no limit.
    pub concurrent_limit: u32,
    /// If set to true, there won't be an initial icmp check.
    ///
    /// All hosts are assumed to be reachable.
    pub skip_icmp_check: bool,
}

/// Start a TCP port scan with this function
///
/// **Parameter**:
/// - settings: [TcpPortScannerSettings]
/// - `tx`: [Sender] of [SocketAddr]
pub async fn start_tcp_con_port_scan(
    settings: TcpPortScannerSettings,
    tx: Sender<SocketAddr>,
) -> Result<(), TcpPortScanError> {
    info!("Starting tcp port Scan");
    // Increase the NO_FILE limit if necessary
    if let Err(err) = rlimit::increase_nofile_limit(settings.concurrent_limit as u64 + 100) {
        return Err(TcpPortScanError::RiseNoFileLimit(err));
    }

    let addresses = if settings.skip_icmp_check {
        info!("Skipping icmp check");
        settings.addresses
    } else {
        let (tx, rx) = mpsc::channel(1);

        let icmp_settings = IcmpScanSettings {
            addresses: settings.addresses,
            timeout: Duration::from_millis(1000),
            concurrent_limit: settings.concurrent_limit,
        };
        let icmp_scan = tokio::spawn(start_icmp_scan(icmp_settings, tx));
        let addresses = ReceiverStream::new(rx).map(IpNetwork::from).collect().await;
        icmp_scan.await.map_err(TcpPortScanError::TaskJoin)??;
        addresses
    };
    if addresses.is_empty() && settings.skip_icmp_check {
        warn!("All hosts are unreachable. Check your targets or disable the icmp check.");
    }
    let iter_addresses = addresses.iter().flat_map(|network| network.iter());
    let iter_ports = settings.ports.iter().cloned().flatten();

    stream::iter(iter_ports.cartesian_product(iter_addresses))
        .for_each_concurrent(settings.concurrent_limit as usize, move |(port, addr)| {
            let tx = tx.clone();

            async move {
                let s_addr = SocketAddr::new(addr, port);

                for _ in 0..=settings.max_retries {
                    if let Ok(res) = timeout(settings.timeout, TcpStream::connect(s_addr)).await {
                        match res {
                            Ok(mut stream) => {
                                if let Err(err) = stream.shutdown().await {
                                    debug!("Couldn't shut down tcp stream: {err}");
                                }

                                if let Err(err) = tx.send(s_addr).await {
                                    warn!("Could not send result to tx: {err}");
                                }

                                break;
                            }
                            Err(err) => {
                                let err_str = err.to_string();
                                if err_str.contains("refused") {
                                    trace!("Connection refused on {s_addr}: {err}");
                                } else {
                                    warn!("Unknown error: {err}");
                                }
                            }
                        }
                    } else {
                        trace!("Timeout reached");
                    }
                    sleep(settings.retry_interval).await;
                }
            }
        })
        .await;

    info!("Finished tcp port scan");

    Ok(())
}
