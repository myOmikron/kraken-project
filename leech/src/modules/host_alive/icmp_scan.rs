//! This module holds everything regarding icmp scanning

use std::net::IpAddr;
use std::time::Duration;

use futures::{stream, StreamExt};
use log::{debug, error, info, trace, warn};
use rand::random;
use surge_ping::{Client, PingIdentifier, PingSequence, SurgeError, ICMP};
use tokio::sync::mpsc::Sender;

use crate::modules::host_alive::error::IcmpScanError;

/// The settings of a icmp scan
#[derive(Debug)]
pub struct IcmpScanSettings {
    /// The addresses to scan
    pub addresses: Vec<IpAddr>,
    /// The time wait for a pong
    pub timeout: Duration,
    /// Maximum of concurrent tasks that should be spawned
    ///
    /// 0 means, that there should be no limit.
    pub concurrent_limit: u32,
}

/// Start a ICMP scan.
///
/// Sends hosts that respond with a pong through the `tx` Sender.
///
/// **Parameter**:
/// - `settings`: [IcmpScanSettings]
/// - `tx`: [Sender] of [IpAddr]
pub async fn start_icmp_scan(
    settings: IcmpScanSettings,
    tx: Sender<IpAddr>,
) -> Result<(), IcmpScanError> {
    // Increase the NO_FILE limit if necessary
    if let Err(err) = rlimit::increase_nofile_limit(settings.concurrent_limit as u64 + 100) {
        return Err(IcmpScanError::RiseNoFileLimit(err));
    }

    let conf_v4 = surge_ping::Config::default();
    let conf_v6 = surge_ping::Config::builder().kind(ICMP::V6).build();

    let icmp_v4_client = Client::new(&conf_v4).map_err(IcmpScanError::CreateIcmpClient)?;
    let icmp_v6_client = Client::new(&conf_v6).map_err(IcmpScanError::CreateIcmpClient)?;

    info!("Starting icmp check");

    stream::iter(settings.addresses)
        .for_each_concurrent(settings.concurrent_limit as usize, |addr| {
            let icmp_client = if addr.is_ipv4() {
                icmp_v4_client.clone()
            } else {
                icmp_v6_client.clone()
            };

            let tx = tx.clone();

            async move {
                const PAYLOAD: &[u8] = &[];
                let mut pinger = icmp_client
                    .pinger(addr, PingIdentifier::from(random::<u16>()))
                    .await;

                if let Err(err) = pinger
                    .timeout(settings.timeout)
                    .ping(PingSequence(0), PAYLOAD)
                    .await
                {
                    match err {
                        SurgeError::Timeout { .. } => trace!("Host timeout: {addr}"),
                        _ => error!("ICMP error: {err}"),
                    }
                } else {
                    debug!("Host is up: {addr}");
                    if let Err(err) = tx.send(addr).await {
                        warn!("Could not send result to tx: {err}");
                    }
                }
            }
        })
        .await;

    info!("Finished icmp check");

    Ok(())
}
