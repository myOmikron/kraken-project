//! This module holds a port scanning utility.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use futures::{stream, StreamExt};
use itertools::iproduct;
use log::{debug, error, info, trace, warn};
use rand::random;
use surge_ping::{Client, PingIdentifier, PingSequence, SurgeError, ICMP};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};

use crate::modules::port_scanner::error::TcpPortScanError;

pub mod error;

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
    pub max_retries: u8,
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
    let addresses = Arc::new(Mutex::new(vec![]));

    if settings.skip_icmp_check {
        info!("Skipping icmp check");
        addresses.lock().await.extend(settings.addresses);
    } else {
        info!("Starting icmp check");

        let conf_v4 = surge_ping::Config::default();
        let conf_v6 = surge_ping::Config::builder().kind(ICMP::V6).build();

        let icmp_v4_client = Client::new(&conf_v4).map_err(TcpPortScanError::CreateIcmpClient)?;
        let icmp_v6_client = Client::new(&conf_v6).map_err(TcpPortScanError::CreateIcmpClient)?;

        stream::iter(settings.addresses)
            .for_each_concurrent(10, |addr| {
                let icmp_client = if addr.is_ipv4() {
                    icmp_v4_client.clone()
                } else {
                    icmp_v6_client.clone()
                };

                let addresses = addresses.clone();

                async move {
                    const PAYLOAD: &[u8] = &[];
                    let mut pinger = icmp_client
                        .pinger(addr, PingIdentifier::from(random::<u16>()))
                        .await;

                    if let Err(err) = pinger
                        .timeout(Duration::from_millis(1000))
                        .ping(PingSequence(0), PAYLOAD)
                        .await
                    {
                        match err {
                            SurgeError::Timeout { .. } => trace!("Host timeout: {addr}"),
                            _ => error!("ICMP error: {err}"),
                        }
                    } else {
                        debug!("Host is up: {addr}");
                        addresses.lock().await.push(addr);
                    }
                }
            })
            .await;

        info!("Finished icmp check");
    }

    let product_it = iproduct!(settings.port_range, addresses.lock().await.clone());

    // Increase the NO_FILE limit if necessary
    if let Err(err) = rlimit::increase_nofile_limit(settings.concurrent_limit as u64 + 100) {
        return Err(TcpPortScanError::RiseNoFileLimit(err));
    }

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

    Ok(())
}
