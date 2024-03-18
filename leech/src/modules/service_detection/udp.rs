use std::collections::BTreeMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::time::Duration;

use futures::stream;
use futures::TryStreamExt;
use ipnetwork::IpNetwork;
use itertools::Itertools;
use log::debug;
use log::info;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tokio::time::timeout;

use super::error::UdpServiceScanError;
use super::ProtocolSet;
use super::Service;
use crate::modules::service_detection::generated;
use crate::modules::service_detection::generated::Match;

/// Settings for a service detection
#[derive(Clone)]
pub struct UdpServiceDetectionSettings {
    /// Ip addresses / networks to scan
    pub addresses: Vec<IpNetwork>,

    /// Ports to scan
    pub ports: Vec<RangeInclusive<u16>>,

    /// Number of times packet sending should be retried if no response was received within the `timeout`
    ///
    /// Set to 0 to only try once.
    pub max_retries: u32,

    /// How much time to wait between sending probe payloads (independent of
    /// timeout, aka it can send while another pending probe did not yet receive
    /// data within the timeout interval).
    pub retry_interval: Duration,

    /// The timeout for receiving data on a single sent probe.
    pub timeout: Duration,

    /// Maximum of concurrent tasks that should be spawned
    ///
    /// 0 means, that there should be no limit.
    pub concurrent_limit: u32,
}

impl UdpServiceDetectionSettings {
    async fn probe_udp(
        &self,
        address: IpAddr,
        port: u16,
        payload: &[u8],
    ) -> Result<Option<Vec<u8>>, UdpServiceScanError> {
        let addr = SocketAddr::new(address, port);

        let mut set = JoinSet::new();

        for i in 0..=self.max_retries {
            let offset = self.retry_interval.saturating_mul(i);
            set.spawn(timeout(
                self.timeout.saturating_add(offset),
                single_probe_udp(offset, addr, payload.to_vec()),
            ));
        }

        while let Some(res) = set.join_next().await {
            let res = res?; // propagate JoinError
            if let Ok(res) = res {
                // ignore timeouts
                let res = res?; // propagate single_probe_udp errors
                if let Some(res) = res {
                    // check for single_probe_udp returning None (aka no response)
                    set.abort_all();
                    return Ok(Some(res));
                }
            } else {
                debug!("timeout on udp service probe {}", addr);
            }
        }

        Ok(None)
    }
}

/// Returns the received payload or None in case there was no response. Returns
/// errors in case of other networking errors.
async fn single_probe_udp(
    wait_before: Duration,
    addr: SocketAddr,
    payload: Vec<u8>,
) -> Result<Option<Vec<u8>>, UdpServiceScanError> {
    sleep(wait_before).await;
    let sock = UdpSocket::bind(match addr {
        SocketAddr::V4(_) => "0.0.0.0:0",
        SocketAddr::V6(_) => "[::]:0",
    })
    .await?;
    let mut buf = [0; 4096];
    sock.connect(addr).await?;
    sock.send(&payload).await?;
    // TODO: it's possible the server might respond with ICMP Destination/port unreachable (could handle that here)
    let err = sock.recv(&mut buf).await;
    match err {
        Ok(n) => Ok(Some(buf[0..n].to_vec())),
        // TODO: certain I/O errors should probably just return None here
        Err(e) => Err(UdpServiceScanError::IoError(e)),
    }
}

#[derive(Debug, Clone)]
/// A found open or detected UDP service on a specific port.
pub struct UdpServiceDetectionResult {
    /// The address that the open port was found on.
    pub address: IpAddr,
    /// The port that was found open with the given service.
    pub port: u16,
    /// The service that was found possibly running on this port.
    pub service: Service,
}

/// Detects the UDP service behind a socket by sending it known probes or trying `\r\n\r\n` in case there are no
/// matching probes.
pub async fn start_udp_service_detection(
    settings: &UdpServiceDetectionSettings,
    tx: Sender<UdpServiceDetectionResult>,
) -> Result<(), UdpServiceScanError> {
    let iter_addresses = settings
        .addresses
        .iter()
        .cloned()
        .flat_map(|network| network.into_iter());
    let iter_ports = settings.ports.iter().cloned().flatten();
    stream::iter(
        iter_ports
            .cartesian_product(iter_addresses)
            .map(Ok::<(u16, IpAddr), UdpServiceScanError>),
    )
    .try_for_each_concurrent(
        settings.concurrent_limit as usize,
        move |(port, address)| {
            let tx = tx.clone();
            async move {
                let mut partial_matches = BTreeMap::new();
                for prev in 0..3 {
                    debug!("Starting udp scans prevalence={prev}");
                    for probe in &generated::PROBES.udp_probes[prev] {
                        if let Some(data) = settings.probe_udp(address, port, probe.payload).await?
                        {
                            match probe.is_match(&data) {
                                Match::No => {}
                                Match::Partial => {
                                    partial_matches.insert(probe.service, ProtocolSet::UDP);
                                }
                                Match::Exact => {
                                    debug!(
                                        "Found exact UDP service {} on {address} port {port}",
                                        probe.service
                                    );
                                    tx.send(UdpServiceDetectionResult {
                                        address,
                                        port,
                                        service: Service::Definitely {
                                            service: probe.service,
                                            protocols: ProtocolSet::UDP,
                                        },
                                    })
                                    .await?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }

                if partial_matches.is_empty() {
                    let payload = b"\r\n\r\n";
                    if let Some(_data) = settings.probe_udp(address, port, payload).await? {
                        debug!("Generic CRLF probe on port {port} matched");
                        tx.send(UdpServiceDetectionResult {
                            address,
                            port,
                            service: Service::Unknown,
                        })
                        .await?;
                    }
                } else {
                    debug!("Found maybe UDP services {partial_matches:?} on port {port}");
                    tx.send(UdpServiceDetectionResult {
                        address,
                        port,
                        service: Service::Maybe(partial_matches),
                    })
                    .await?;
                }

                Ok(())
            }
        },
    )
    .await?;

    info!("Finished UDP service detection");

    Ok(())
}
