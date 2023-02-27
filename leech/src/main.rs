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

use std::net::IpAddr;
use std::num::{NonZeroU16, NonZeroUsize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use clap::{Parser, Subcommand};
use ipnet::IpNet;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use trust_dns_resolver::Name;

use crate::config::get_config;
use crate::modules::bruteforce_subdomains::{bruteforce_subdomains, BruteforceSubdomainsSettings};
use crate::modules::certificate_transparency::{
    query_ct_api, query_ct_db, CertificateTransparencySettings,
};
use crate::modules::port_scanner::{start_tcp_con_port_scan, TcpPortScannerSettings};

pub mod config;
pub mod modules;

pub(crate) struct Regexes {
    pub(crate) ports: Regex,
}

static RE: Lazy<Regexes> = Lazy::new(|| Regexes {
    ports: Regex::new(r#"^(?P<range>\d*-\d*)$|^(?P<single>\d+)$|^$"#).unwrap(),
});

#[deny(missing_docs)]
#[derive(Subcommand)]
enum RunCommand {
    /// Bruteforce subdomains via DNS
    BruteforceSubdomains {
        /// Valid domain name
        target: Name,
        /// Path to a wordlist that can be used for subdomain enumeration.
        ///
        /// The entries in the wordlist are assumed to be line seperated.
        #[clap(short = 'w', long = "wordlist")]
        wordlist_path: PathBuf,
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
        #[clap(default_value_t = NonZeroUsize::new(1000).unwrap())]
        concurrent_limit: NonZeroUsize,
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
        /// Skips the initial ping check.
        ///
        /// All hosts are assumed to be reachable.
        #[clap(long)]
        #[clap(default_value_t = false)]
        skip_ping_check: bool,
    },
}

#[derive(Subcommand)]
enum Command {
    Server,
    Execute {
        #[clap(subcommand)]
        command: RunCommand,
    },
}

#[derive(Parser)]
struct Cli {
    #[clap(long = "config-path")]
    #[clap(help = "Specify an alternative path to the config file")]
    #[clap(default_value_t = String::from("/etc/leech/config.toml"))]
    config_path: String,

    #[clap(subcommand)]
    commands: Command,
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.commands {
        Command::Server => {
            let _config = get_config(&cli.config_path).map_err(|e| e.to_string())?;
        }
        Command::Execute { command } => match command {
            RunCommand::BruteforceSubdomains {
                target,
                wordlist_path,
            } => {
                bruteforce_subdomains(BruteforceSubdomainsSettings {
                    domain: target.to_string(),
                    wordlist_path,
                })
                .await?
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
                skip_ping_check,
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

                let mut parsed_ports = vec![];

                if ports.is_empty() {
                    parsed_ports.extend(1..=u16::MAX);
                }

                for port in ports {
                    let port_parts = port.split(',');
                    for part in port_parts {
                        if let Some(captures) = RE.ports.captures(part) {
                            if let Some(c) = captures.get(0) {
                                if c.as_str().is_empty() {
                                    continue;
                                }
                            }
                            if let Some(m) = captures.name("range") {
                                let mut start = 1;
                                let mut end = u16::MAX;
                                for (idx, content) in m.as_str().split('-').into_iter().enumerate()
                                {
                                    match idx {
                                        0 => {
                                            if content.is_empty() {
                                                start = 1;
                                            } else if let Ok(v) = NonZeroU16::from_str(content) {
                                                start = u16::from(v);
                                            } else {
                                                return Err(format!("Invalid port: {content}"));
                                            }
                                        }
                                        1 => {
                                            if content.is_empty() {
                                                end = u16::MAX;
                                            } else if let Ok(v) = NonZeroU16::from_str(content) {
                                                end = u16::from(v);
                                            } else {
                                                return Err(format!("Invalid port: {content}"));
                                            }
                                        }
                                        _ => unreachable!(""),
                                    }
                                }

                                if end < start {
                                    return Err(format!("Invalid port range: {end} < {start}"));
                                }

                                for port in start..=end {
                                    parsed_ports.push(port);
                                }
                            } else if let Some(m) = captures.name("single") {
                                if let Ok(v) = NonZeroU16::from_str(m.as_str()) {
                                    parsed_ports.push(u16::from(v));
                                } else {
                                    return Err(format!("Invalid port: {}", m.as_str()));
                                }
                            }
                        } else {
                            return Err(format!("Invalid port declaration found: {part}"));
                        }
                    }
                }

                parsed_ports.sort();
                parsed_ports.dedup();

                start_tcp_con_port_scan(TcpPortScannerSettings {
                    addresses,
                    port_range: parsed_ports,
                    timeout: Duration::from_millis(timeout as u64),
                    skip_ping_check,
                    max_retries,
                    retry_interval: Duration::from_millis(retry_interval as u64),
                    concurrent_limit: usize::from(concurrent_limit),
                })
                .await;
            }
        },
    }

    Ok(())
}
