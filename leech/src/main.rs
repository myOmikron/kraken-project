//! # Leeches
//! Leeches are the workers of kraken.
//!
//! They provide a gRPC server to receive requests from kraken and respond with results.
//! If this connection is lost somehow, they will store the results in a local database
//! and will try to connect to the kraken gRPC server to send the missing data.
//!
//! You can also use the leech as a cli utility without a kraken attached for manual
//! execution and testing. See the subcommand `run` for further information.
#![warn(missing_docs)]
#![cfg_attr(
    feature = "rorm-main",
    allow(dead_code, unused_variables, unused_imports)
)]

use std::env;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use clap::{ArgAction, Parser, Subcommand};
use ipnet::IpNet;
use itertools::Itertools;
use log::{error, info};
use tokio::sync::mpsc;
use tokio::task;
use trust_dns_resolver::Name;

use crate::config::get_config;
use crate::modules::bruteforce_subdomains::{
    bruteforce_subdomains, BruteforceSubdomainResult, BruteforceSubdomainsSettings,
};
use crate::modules::certificate_transparency::{
    query_ct_api, query_ct_db, CertificateTransparencySettings,
};
use crate::modules::port_scanner::{start_tcp_con_port_scan, TcpPortScannerSettings};
use crate::rpc::start_rpc_server;

pub mod config;
pub mod modules;
pub mod rpc;
pub mod utils;

/// The execution commands
#[derive(Subcommand)]
pub enum RunCommand {
    /// Bruteforce subdomains via DNS
    BruteforceSubdomains {
        /// Valid domain name
        target: Name,
        /// Path to a wordlist that can be used for subdomain enumeration.
        ///
        /// The entries in the wordlist are assumed to be line seperated.
        #[clap(short = 'w', long = "wordlist")]
        wordlist_path: PathBuf,
        /// The concurrent task limit
        #[clap(long)]
        #[clap(default_value_t = NonZeroU32::new(50).unwrap())]
        concurrent_limit: NonZeroU32,
    },
    /// Retrieve domains through certificate transparency
    CertificateTransparency {
        /// Valid domain name
        target: String,
        /// Whether expired certificates should be included
        #[clap(long)]
        #[clap(default_value_t = false)]
        include_expired: bool,
        /// Use the database instead of the API
        #[clap(long)]
        #[clap(default_value_t = false)]
        db: bool,
    },
    /// A simple port scanning utility
    PortScanner {
        /// Valid IPv4 or IPv6 addresses or networks in CIDR notation
        #[clap(required(true))]
        targets: Vec<String>,
        /// A single port, multiple, comma seperated ports or (inclusive) port ranges
        ///
        /// If no values are supplied, 1-65535 is used as default
        #[clap(short = 'p')]
        ports: Vec<String>,
        /// Valid IPv4 or IPv6 addresses or networks in CIDR notation
        #[clap(long)]
        exclude: Vec<String>,
        /// The time to wait until a connection is considered failed.
        ///
        /// The timeout is specified in milliseconds.
        #[clap(long)]
        #[clap(default_value_t = 1000)]
        timeout: u16,
        /// The concurrent task limit
        #[clap(long)]
        #[clap(default_value_t = NonZeroU32::new(1000).unwrap())]
        concurrent_limit: NonZeroU32,
        /// The number of times the connection should be retried if it failed.
        #[clap(long)]
        #[clap(default_value_t = 6)]
        max_retries: u8,
        /// The interval that should be wait between retries on a port.
        ///
        /// The interval is specified in milliseconds.
        #[clap(long)]
        #[clap(default_value_t = 100)]
        retry_interval: u16,
        /// Skips the initial icmp check.
        ///
        /// All hosts are assumed to be reachable.
        #[clap(long)]
        #[clap(default_value_t = false)]
        skip_icmp_check: bool,
    },
}

/// All available subcommands
#[derive(Subcommand)]
pub enum Command {
    /// Start the leech as a server
    Server,
    /// Execute a command via CLI
    Execute {
        /// Specifies the verbosity of the output
        #[clap(short = 'v', global = true, action = ArgAction::Count)]
        verbosity: u8,

        /// the subcommand to execute
        #[clap(subcommand)]
        command: RunCommand,
    },
}

/// The main CLI parser
#[derive(Parser)]
pub struct Cli {
    /// Specify an alternative path to the config file
    #[clap(long = "config-path")]
    #[clap(default_value_t = String::from("/etc/leech/config.toml"))]
    config_path: String,

    /// Subcommands
    #[clap(subcommand)]
    commands: Command,
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.commands {
        Command::Server => {
            let config = get_config(&cli.config_path)?;
            start_rpc_server(&config).await?;
        }
        Command::Execute { command, verbosity } => {
            if env::var("RUST_LOG").is_err() {
                match verbosity {
                    0 => env::set_var("RUST_LOG", "leech=info"),
                    1 => env::set_var("RUST_LOG", "leech=debug"),
                    _ => env::set_var("RUST_LOG", "leech=trace"),
                }
            }
            env_logger::init();

            match command {
                RunCommand::BruteforceSubdomains {
                    target,
                    wordlist_path,
                    concurrent_limit,
                } => {
                    let (tx, mut rx) = mpsc::channel(128);

                    task::spawn(async move {
                        while let Some(res) = rx.recv().await {
                            match res {
                                BruteforceSubdomainResult::A { source, target } => {
                                    info!("Found a record for {source}: {target}");
                                }
                                BruteforceSubdomainResult::Aaaa { source, target } => {
                                    info!("Found aaaa record for {source}: {target}");
                                }
                                BruteforceSubdomainResult::Cname { source, target } => {
                                    info!("Found cname record for {source}: {target}");
                                }
                            };
                        }
                    });

                    let settings = BruteforceSubdomainsSettings {
                        domain: target.to_string(),
                        wordlist_path,
                        concurrent_limit: u32::from(concurrent_limit),
                    };
                    if let Err(err) = bruteforce_subdomains(settings, tx).await {
                        error!("{err}");
                    }
                }
                RunCommand::CertificateTransparency {
                    target,
                    include_expired,
                    db,
                } => {
                    let ct = CertificateTransparencySettings {
                        target,
                        include_expired,
                    };
                    if db {
                        query_ct_db(ct).await;
                    } else {
                        query_ct_api(ct).await;
                    }
                }
                RunCommand::PortScanner {
                    targets,
                    exclude,
                    ports,
                    timeout,
                    concurrent_limit,
                    max_retries,
                    retry_interval,
                    skip_icmp_check,
                } => {
                    let mut addresses = vec![];
                    for target in targets {
                        if let Ok(addr) = IpAddr::from_str(&target) {
                            addresses.push(addr);
                        } else if let Ok(net) = IpNet::from_str(&target) {
                            addresses.extend(net.hosts());
                        } else {
                            return Err(format!("{target} isn't valid ip address or ip net"));
                        }
                    }

                    let mut exclude_addresses = vec![];
                    for ex in exclude {
                        if let Ok(addr) = IpAddr::from_str(&ex) {
                            exclude_addresses.push(addr);
                        } else if let Ok(net) = IpNet::from_str(&ex) {
                            exclude_addresses.extend(net.hosts());
                        } else {
                            return Err(format!("{ex} isn't valid ip address or ip net"));
                        }
                    }

                    let addresses: Vec<IpAddr> = addresses
                        .into_iter()
                        .filter(|addr| !exclude_addresses.contains(addr))
                        .sorted()
                        .dedup()
                        .collect();

                    let mut port_range = vec![];

                    if ports.is_empty() {
                        port_range.extend(1..=u16::MAX);
                    } else {
                        utils::parse_ports(&ports, &mut port_range)?;
                    }

                    let settings = TcpPortScannerSettings {
                        addresses,
                        port_range,
                        timeout: Duration::from_millis(timeout as u64),
                        skip_icmp_check,
                        max_retries,
                        retry_interval: Duration::from_millis(retry_interval as u64),
                        concurrent_limit: u32::from(concurrent_limit),
                    };

                    let (tx, mut rx) = mpsc::channel(128);

                    task::spawn(async move {
                        while let Some(addr) = rx.recv().await {
                            info!("Open port found: {addr}");
                        }
                    });

                    if let Err(err) = start_tcp_con_port_scan(settings, tx).await {
                        error!("{err}");
                    }
                }
            }
        }
    }

    Ok(())
}
