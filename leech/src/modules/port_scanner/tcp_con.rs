//! This module holds a tcp connect port scanner

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use futures::{stream, StreamExt};
use ipnetwork::IpNetwork;
use itertools::iproduct;
use log::{debug, error, info, trace, warn};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, timeout};

use crate::modules::host_alive::icmp_scan::{start_icmp_scan, IcmpScanSettings};
use crate::modules::port_scanner::error::TcpPortScanError;

/// The settings of a tcp connection port scan
#[derive(Clone, Debug)]
pub struct TcpPortScannerSettings {
    /// The addresses to scan
    pub addresses: Vec<IpAddr>,
    /// The port range to scan
    pub port_range: Vec<u16>,
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

    let addresses;

    if settings.skip_icmp_check {
        info!("Skipping icmp check");
        addresses = settings.addresses;
    } else {
        let (tx, mut rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let mut a = vec![];
            while let Some(addr) = rx.recv().await {
                a.push(addr);
            }
            if a.is_empty() {
                warn!("All hosts are unreachable. Check your targets or disable the icmp check.");
            }
            a
        });

        let icmp_settings = IcmpScanSettings {
            addresses: settings
                .addresses
                .into_iter()
                .map(IpNetwork::from)
                .collect(),
            timeout: Duration::from_millis(1000),
            concurrent_limit: settings.concurrent_limit,
        };

        start_icmp_scan(icmp_settings, tx).await?;

        match handle.await {
            Ok(a) => addresses = a,
            Err(err) => {
                error!("Could not join on icmp rx handle task: {err}");
                return Err(TcpPortScanError::TaskJoin(err));
            }
        }
    }

    let product_it = iproduct!(settings.port_range, addresses);

    stream::iter(product_it)
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
