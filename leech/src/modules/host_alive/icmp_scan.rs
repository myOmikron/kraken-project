//! This module holds everything regarding icmp scanning

use std::net::IpAddr;
use std::time::Duration;

use futures::stream;
use futures::StreamExt;
use ipnetwork::IpNetwork;
use kraken_proto::any_attack_response;
use kraken_proto::push_attack_request;
use kraken_proto::shared::Address;
use kraken_proto::HostsAliveRequest;
use kraken_proto::HostsAliveResponse;
use kraken_proto::RepeatedHostsAliveResponse;
use log::debug;
use log::error;
use log::info;
use log::trace;
use log::warn;
use rand::random;
use surge_ping::Client;
use surge_ping::PingIdentifier;
use surge_ping::PingSequence;
use surge_ping::SurgeError;
use surge_ping::ICMP;
use tokio::sync::mpsc::Sender;
use tonic::Status;

use crate::modules::host_alive::error::IcmpScanError;
use crate::modules::StreamedAttack;

/// Attack scanning hosts with ICMP
pub struct IcmpScan;
#[tonic::async_trait]
impl StreamedAttack for IcmpScan {
    type Settings = IcmpScanSettings;
    type Output = IpAddr;
    type Error = IcmpScanError;
    async fn execute(
        settings: Self::Settings,
        sender: Sender<Self::Output>,
    ) -> Result<(), Self::Error> {
        start_icmp_scan(settings, sender).await
    }

    type Request = HostsAliveRequest;
    fn get_attack_uuid(request: &Self::Request) -> &str {
        &request.attack_uuid
    }
    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status> {
        if request.targets.is_empty() {
            return Err(Status::invalid_argument("no hosts to check"));
        }

        Ok(IcmpScanSettings {
            concurrent_limit: request.concurrent_limit,
            timeout: Duration::from_millis(request.timeout),
            addresses: request
                .targets
                .into_iter()
                .map(IpNetwork::try_from)
                .collect::<Result<_, _>>()?,
        })
    }

    type Response = HostsAliveResponse;
    fn encode_output(output: Self::Output) -> Self::Response {
        HostsAliveResponse {
            host: Some(Address::from(output)),
        }
    }

    fn print_output(output: &Self::Output) {
        info!("Host up: {output}");
    }

    fn wrap_for_backlog(response: Self::Response) -> any_attack_response::Response {
        any_attack_response::Response::HostsAlive(response)
    }

    fn wrap_for_push(responses: Vec<Self::Response>) -> push_attack_request::Response {
        push_attack_request::Response::HostsAlive(RepeatedHostsAliveResponse { responses })
    }
}

/// The settings of a icmp scan
#[derive(Debug)]
pub struct IcmpScanSettings {
    /// The addresses to scan
    pub addresses: Vec<IpNetwork>,
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
/// - `tx`: [Sender] of [IcmpScanResult]
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

    let ips = settings.addresses.into_iter().flat_map(|x| x.iter());

    stream::iter(ips)
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
