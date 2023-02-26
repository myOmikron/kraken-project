//! This module holds a port scanning utility.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use futures::{stream, StreamExt};
use itertools::iproduct;
use log::debug;
use rand::random;
use surge_ping::{Client, PingIdentifier, PingSequence, ICMP};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};

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
    pub concurrent_limit: usize,
    /// If set to true, there won't be an initial ping check.
    ///
    /// All hosts are assumed to be reachable.
    pub skip_ping_check: bool,
}

/// Start a TCP port scan with this function
///
/// **Parameter**:
/// - settings: [TcpPortScannerSettings]
pub async fn start_tcp_con_port_scan(settings: TcpPortScannerSettings) {
    let icmp_config_v4 = surge_ping::Config::default();
    let icmp_config_v6 = surge_ping::Config::builder().kind(ICMP::V6).build();

    let icmp_v4_client = match Client::new(&icmp_config_v4) {
        Ok(client) => client,
        Err(err) => {
            println!("Error creating ping client: {err}");
            return;
        }
    };

    let icmp_v6_client = match Client::new(&icmp_config_v6) {
        Ok(client) => client,
        Err(err) => {
            println!("Error creating ping client: {err}");
            return;
        }
    };

    let mut icmp_handles = JoinSet::new();

    for addr in settings.addresses {
        let icmp_client = if addr.is_ipv4() {
            icmp_v4_client.clone()
        } else {
            icmp_v6_client.clone()
        };
        icmp_handles.spawn(async move {
            const PAYLOAD: [u8; 56] = [0; 56];
            let mut pinger = icmp_client
                .pinger(addr, PingIdentifier::from(random::<u16>()))
                .await;

            let mut reachable = false;
            for seq in 0..3 {
                if pinger.ping(PingSequence(seq), &PAYLOAD).await.is_ok() {
                    reachable = true;
                    break;
                }
            }

            match reachable {
                true => Some(addr),
                false => None,
            }
        });
    }

    let mut reachable_hosts = vec![];

    while let Some(Ok(Some(addr))) = icmp_handles.join_next().await {
        reachable_hosts.push(addr);
    }

    let product_it = iproduct!(settings.port_range, reachable_hosts);

    let (tx, mut rx) = mpsc::channel(1000);

    tokio::spawn(async move {
        while let Some(res) = rx.recv().await {
            if let Some(s_addr) = res {
                println!("{s_addr}");
            }
        }
    });

    // Increase the NO_FILE limit if necessary
    if let Err(err) = rlimit::increase_nofile_limit(settings.concurrent_limit as u64 + 100) {
        println!("Could not increase nofile limit: {err}");
        return;
    }

    stream::iter(product_it)
        .for_each_concurrent(settings.concurrent_limit, move |(port, addr)| {
            let tx = tx.clone();

            async move {
                let s_addr = SocketAddr::new(addr, port);

                for _ in 0..=settings.max_retries {
                    if let Ok(res) = timeout(settings.timeout, TcpStream::connect(s_addr)).await {
                        match res {
                            Ok(mut stream) => {
                                stream.shutdown().await.unwrap();
                                tx.send(Some(s_addr)).await.unwrap();
                                break;
                            }
                            Err(err) => {
                                let err_str = err.to_string();
                                if err_str.contains("refused") {
                                    tx.send(None).await.unwrap();
                                } else {
                                    println!("{err}");
                                }
                            }
                        }
                    } else {
                        debug!("Timeout reached");
                        tx.send(None).await.unwrap();
                    }
                    sleep(settings.retry_interval).await;
                }
            }
        })
        .await;
}
