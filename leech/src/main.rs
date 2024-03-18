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
use std::error::Error;
use std::io::Write;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use chrono::Datelike;
use chrono::Timelike;
use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use dehashed_rs::SearchType;
use ipnetwork::IpNetwork;
use itertools::Itertools;
use kraken_proto::push_attack_request;
use kraken_proto::push_attack_service_client::PushAttackServiceClient;
use kraken_proto::shared::CertEntry;
use kraken_proto::CertificateTransparencyResponse;
use kraken_proto::PushAttackRequest;
use log::error;
use log::info;
use log::warn;
use prost_types::Timestamp;
use rorm::cli;
use rorm::Database;
use rorm::DatabaseConfiguration;
use rorm::DatabaseDriver;
use tokio::sync::mpsc;
use tokio::task;
use trust_dns_resolver::Name;
use uuid::Uuid;

use crate::backlog::start_backlog;
use crate::config::get_config;
use crate::config::Config;
use crate::modules::bruteforce_subdomains::bruteforce_subdomains;
use crate::modules::bruteforce_subdomains::BruteforceSubdomainResult;
use crate::modules::bruteforce_subdomains::BruteforceSubdomainsSettings;
use crate::modules::certificate_transparency::query_ct_api;
use crate::modules::certificate_transparency::CertificateTransparencySettings;
use crate::modules::dehashed;
use crate::modules::dns::txt::start_dns_txt_scan;
use crate::modules::dns::txt::DnsTxtScanSettings;
use crate::modules::host_alive::icmp_scan::start_icmp_scan;
use crate::modules::host_alive::icmp_scan::IcmpScanSettings;
use crate::modules::os_detection::os_detection;
use crate::modules::os_detection::tcp_fingerprint::fingerprint_tcp;
use crate::modules::os_detection::OsDetectionSettings;
use crate::modules::service_detection;
use crate::modules::service_detection::tcp::start_tcp_service_detection;
use crate::modules::service_detection::tcp::TcpServiceDetectionResult;
use crate::modules::service_detection::tcp::TcpServiceDetectionSettings;
use crate::modules::whois;
use crate::rpc::start_rpc_server;
use crate::utils::input;
use crate::utils::kraken_endpoint;

pub mod backlog;
pub mod config;
pub mod logging;
pub mod models;
pub mod modules;
pub mod rpc;
pub mod utils;

/// The technique to use for the port scan
#[derive(Debug, ValueEnum, Copy, Clone)]
pub enum PortScanTechnique {
    /// A tcp connect scan
    TcpCon,
    /// A icmp scan
    Icmp,
}

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
        #[clap(default_value_t = NonZeroU32::new(100).unwrap())]
        concurrent_limit: NonZeroU32,
    },
    /// Parse known TXT DNS entries
    DnsTxt {
        /// Valid domain name
        target: Name,
    },
    /// Retrieve domains through certificate transparency
    CertificateTransparency {
        /// Valid domain name
        target: String,
        /// Whether expired certificates should be included
        #[clap(long)]
        #[clap(default_value_t = false)]
        include_expired: bool,
        /// The number of times the connection should be retried if it failed.
        #[clap(long)]
        #[clap(default_value_t = 6)]
        max_retries: u32,
        /// The interval that should be wait between retries on a port.
        ///
        /// The interval is specified in milliseconds.
        #[clap(long)]
        #[clap(default_value_t = 100)]
        retry_interval: u16,
    },
    /// A simple icmp (ping) scanning utility
    IcmpScanner {
        /// Valid IPv4 or IPv6 addresses or networks in CIDR notation
        #[clap(required(true))]
        targets: Vec<String>,

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
    },
    /// Query the dehashed API
    Dehashed {
        /// The query for the api
        query: String,
    },
    /// Query whois entries
    Whois {
        /// The ip to query information for
        query: IpAddr,
    },
    /// Detect open tcp ports and their services
    ServiceDetectionTcp {
        /// Valid IPv4 or IPv6 addresses or networks in CIDR notation
        #[clap(required(true))]
        targets: Vec<String>,

        /// A single port, multiple, comma seperated ports or (inclusive) port ranges
        ///
        /// If no values are supplied, 1-65535 is used as default
        #[clap(short = 'p')]
        ports: Vec<String>,

        /// The time to wait until a connection is considered failed.
        ///
        /// The timeout is specified in milliseconds.
        #[clap(long)]
        #[clap(default_value_t = 1000)]
        connect_timeout: u16,

        /// The time to wait when receiving the service's response during detection.
        ///
        /// The timeout is specified in milliseconds.
        #[clap(long)]
        #[clap(default_value_t = 1000)]
        receive_timeout: u16,

        /// The concurrent task limit
        #[clap(long)]
        #[clap(default_value_t = NonZeroU32::new(1000).unwrap())]
        concurrent_limit: NonZeroU32,

        /// The number of times the connection should be retried if it failed.
        #[clap(long)]
        #[clap(default_value_t = 6)]
        max_retries: u32,

        /// The interval that should be waited between retries on a port.
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

        /// Just runs the initial port scanner without the service detection
        #[clap(long)]
        #[clap(default_value_t = false)]
        just_scan: bool,
    },
    /// Detect the services running behind on a given address in the given port range
    ServiceDetectionUdp {
        /// Valid IPv4 or IPv6 addresses or networks in CIDR notation
        #[clap(required(true))]
        targets: Vec<String>,

        /// A single port, multiple, comma seperated ports or (inclusive) port ranges
        ///
        /// If no values are supplied, 1-65535 is used as default
        #[clap(short = 'p')]
        ports: Vec<String>,

        /// The interval that should be waited for a response after connecting and sending an optional payload.
        ///
        /// The interval is specified in milliseconds.
        #[clap(long)]
        #[clap(default_value_t = 10000)]
        timeout: u64,

        /// The number of times how often to retry sending a UDP packet
        #[clap(long)]
        #[clap(default_value_t = 3)]
        port_retries: u32,

        /// The time between sending UDP packets if a response isn't being heard
        /// back from in time.
        #[clap(long)]
        #[clap(default_value_t = 1000)]
        retry_interval: u64,

        /// The concurrent task limit
        #[clap(long)]
        #[clap(default_value_t = NonZeroU32::new(1000).unwrap())]
        concurrent_limit: NonZeroU32,
    },
    /// Generate the TCP fingerprint for the specified IP on the specified open and specified closed port.
    TcpFingerprint {
        /// The ip to query information for.
        ip: IpAddr,
        /// A TCP port that must accept connections for a consistent fingerprint.
        #[clap(short = 'p')]
        port: u16,
        /// Timeout in milliseconds after which to give up the connection if it didn't send any reply by then.
        #[clap(default_value_t = 1000)]
        timeout: u64,
    },
    /// OS detection.
    OsDetection {
        /// The ip to query information for.
        ip: IpAddr,
        /// Timeout in milliseconds after which to abort checks and discard all immediate results.
        #[clap(default_value_t = 60000)]
        total_timeout: u64,
        /// Timeout for each probe.
        #[clap(default_value_t = 5000)]
        timeout: u64,
        /// Port for SSH detection
        #[clap(default_value_t = 22)]
        ssh_port: u16,
        /// Timeout in milliseconds for each TCP port how long to wait for SYN/ACK on.
        #[clap(default_value_t = 2000)]
        port_timeout: u64,
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

        /// Push the results to a workspace in kraken
        #[clap(long)]
        push: Option<Uuid>,

        /// Api key to authenticate when pushing
        #[clap(long)]
        api_key: Option<String>,

        /// the subcommand to execute
        #[clap(subcommand)]
        command: RunCommand,
    },
    /// Apply migrations to the database
    Migrate {
        /// The directory where the migration files are located
        migration_dir: String,
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
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.commands {
        Command::Migrate { migration_dir } => migrate(&cli.config_path, migration_dir).await?,
        Command::Server => {
            let config = get_config(&cli.config_path)?;
            logging::setup_logging(&config.logging)?;

            let db = get_db(&config).await?;
            let backlog = start_backlog(db, &config.kraken).await?;

            start_rpc_server(&config, backlog).await?;
        }
        Command::Execute {
            command,
            verbosity,
            push,
            api_key,
        } => {
            if env::var("RUST_LOG").is_err() {
                match verbosity {
                    0 => env::set_var("RUST_LOG", "leech=info"),
                    1 => env::set_var("RUST_LOG", "leech=debug"),
                    _ => env::set_var("RUST_LOG", "leech=trace"),
                }
            }
            env_logger::init();

            if let Some(workspace) = push {
                let config = get_config(&cli.config_path)?;

                let api_key = if let Some(api_key) = api_key {
                    api_key
                } else {
                    print!("Please enter your api key: ");
                    std::io::stdout().flush().unwrap();
                    input()
                        .await?
                        .ok_or_else(|| "Can't push to kraken without api key".to_string())?
                };

                match command {
                    RunCommand::CertificateTransparency {
                        target,
                        include_expired,
                        max_retries,
                        retry_interval,
                    } => {
                        let ct = CertificateTransparencySettings {
                            target,
                            include_expired,
                            max_retries,
                            retry_interval: Duration::from_millis(retry_interval as u64),
                        };

                        let entries = query_ct_api(ct).await?;

                        for x in entries
                            .iter()
                            .flat_map(|e| {
                                let mut name_value = e.name_value.clone();

                                name_value.push(e.common_name.clone());
                                name_value
                            })
                            .sorted()
                            .dedup()
                        {
                            info!("{x}");
                        }

                        info!("Sending results to kraken");

                        let endpoint = kraken_endpoint(&config.kraken)?;
                        let chan = endpoint.connect().await.unwrap();

                        let mut client = PushAttackServiceClient::new(chan);
                        client
                            .push_attack(PushAttackRequest {
                                workspace_uuid: workspace.to_string(),
                                api_key,
                                response: Some(
                                    push_attack_request::Response::CertificateTransparency(
                                        CertificateTransparencyResponse {
                                            entries: entries
                                                .into_iter()
                                                .map(|x| CertEntry {
                                                    value_names: x.name_value,
                                                    common_name: x.common_name,
                                                    serial_number: x.serial_number,
                                                    not_after: x.not_after.map(|ts| {
                                                        Timestamp::date_time_nanos(
                                                            ts.year() as i64,
                                                            ts.month() as u8,
                                                            ts.day() as u8,
                                                            ts.hour() as u8,
                                                            ts.minute() as u8,
                                                            ts.second() as u8,
                                                            ts.nanosecond(),
                                                        )
                                                        .unwrap()
                                                    }),
                                                    not_before: x.not_before.map(|ts| {
                                                        Timestamp::date_time_nanos(
                                                            ts.year() as i64,
                                                            ts.month() as u8,
                                                            ts.day() as u8,
                                                            ts.hour() as u8,
                                                            ts.minute() as u8,
                                                            ts.second() as u8,
                                                            ts.nanosecond(),
                                                        )
                                                        .unwrap()
                                                    }),
                                                    issuer_name: x.issuer_name,
                                                })
                                                .collect(),
                                        },
                                    ),
                                ),
                            })
                            .await
                            .unwrap();

                        info!("Finished sending results to kraken")
                    }
                    _ => todo!("Not supported right now for pushing to kraken"),
                }
            } else {
                match command {
                    RunCommand::BruteforceSubdomains {
                        target,
                        wordlist_path,
                        concurrent_limit,
                    } => {
                        let (tx, mut rx) = mpsc::channel(128);

                        let join_handle = task::spawn(bruteforce_subdomains(
                            BruteforceSubdomainsSettings {
                                domain: target.to_string(),
                                wordlist_path,
                                concurrent_limit: u32::from(concurrent_limit),
                            },
                            tx,
                        ));

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

                        join_handle.await??;
                    }
                    RunCommand::DnsTxt { target } => {
                        let (tx, mut rx) = mpsc::channel(128);

                        let join_handle = task::spawn(start_dns_txt_scan(
                            DnsTxtScanSettings {
                                domains: Vec::from([target.to_string()]),
                            },
                            tx,
                        ));

                        while let Some(res) = rx.recv().await {
                            match res.info {
                                modules::dns::txt::TxtScanInfo::SPF { parts } => {
                                    info!("Found SPF entry for {}:", res.domain);
                                    for part in parts {
                                        info!("  {part}");
                                    }
                                }
                                modules::dns::txt::TxtScanInfo::ServiceHints { hints } => {
                                    info!("Found service hints for {}:", res.domain);
                                    for (_rule, hint) in hints {
                                        info!("  {hint}");
                                    }
                                }
                            };
                        }

                        join_handle.await??;
                    }
                    RunCommand::CertificateTransparency {
                        target,
                        include_expired,
                        max_retries,
                        retry_interval,
                    } => {
                        let ct = CertificateTransparencySettings {
                            target,
                            include_expired,
                            max_retries,
                            retry_interval: Duration::from_millis(retry_interval as u64),
                        };

                        let entries = query_ct_api(ct).await?;
                        for x in entries
                            .into_iter()
                            .flat_map(|mut e| {
                                e.name_value.push(e.common_name);
                                e.name_value
                            })
                            .sorted()
                            .dedup()
                        {
                            info!("{x}");
                        }
                    }
                    RunCommand::IcmpScanner {
                        targets,
                        timeout,
                        concurrent_limit,
                    } => {
                        let addresses = targets
                            .iter()
                            .map(|s| IpNetwork::from_str(s))
                            .collect::<Result<_, _>>()?;

                        let settings = IcmpScanSettings {
                            addresses,
                            timeout: Duration::from_millis(timeout as u64),
                            concurrent_limit: u32::from(concurrent_limit),
                        };
                        let (tx, mut rx) = mpsc::channel(1);

                        task::spawn(async move {
                            while let Some(addr) = rx.recv().await {
                                info!("Host up: {addr}");
                            }
                        });

                        if let Err(err) = start_icmp_scan(settings, tx).await {
                            error!("{err}");
                        }
                    }
                    RunCommand::Dehashed { query } => {
                        let email = match env::var("DEHASHED_EMAIL") {
                            Ok(x) => x,
                            Err(_) => {
                                error!("Missing environment variable DEHASHED_EMAIL");
                                return Err("Missing environment variable DEHASHED_EMAIL".into());
                            }
                        };
                        let api_key = match env::var("DEHASHED_API_KEY") {
                            Ok(x) => x,
                            Err(_) => {
                                error!("Missing environment variable DEHASHED_API_KEY");
                                return Err("Missing environment variable DEHASHED_API_KEY".into());
                            }
                        };

                        match dehashed::query(
                            email,
                            api_key,
                            dehashed_rs::Query::Domain(SearchType::Simple(query)),
                        )
                        .await
                        {
                            Ok(x) => {
                                for entry in x.entries {
                                    info!("{entry:?}");
                                }
                            }
                            Err(err) => error!("{err}"),
                        }
                    }
                    RunCommand::Whois { query } => match whois::query_whois(query).await {
                        Ok(x) => info!("Found result\n{x:#?}"),

                        Err(err) => error!("{err}"),
                    },
                    RunCommand::ServiceDetectionTcp {
                        targets,
                        ports,
                        connect_timeout,
                        receive_timeout,
                        concurrent_limit,
                        max_retries,
                        retry_interval,
                        skip_icmp_check,
                        just_scan,
                    } => {
                        let addresses = targets
                            .iter()
                            .map(|s| IpNetwork::from_str(s))
                            .collect::<Result<_, _>>()?;

                        let (tx, mut rx) = mpsc::channel::<TcpServiceDetectionResult>(1);
                        if just_scan {
                            tokio::spawn(async move {
                                while let Some(result) = rx.recv().await {
                                    info!("Open port found: {}", result.addr);
                                }
                            });
                        } else {
                            tokio::spawn(async move {
                                while let Some(result) = rx.recv().await {
                                    info!("Open port found: {}", result.addr,);
                                    info!("It's running: {:?}", result.service);
                                }
                            });
                        }
                        start_tcp_service_detection(
                            TcpServiceDetectionSettings {
                                addresses,
                                ports: utils::parse_ports(&ports, Some(1..=u16::MAX))?,
                                connect_timeout: Duration::from_millis(connect_timeout as u64),
                                receive_timeout: Duration::from_millis(receive_timeout as u64),
                                max_retries,
                                retry_interval: Duration::from_millis(retry_interval as u64),
                                concurrent_limit,
                                skip_icmp_check,
                                just_scan,
                            },
                            tx,
                        )
                        .await
                        .map_err(|e| e.to_string())?;
                    }
                    RunCommand::ServiceDetectionUdp {
                        targets,
                        ports,
                        timeout,
                        port_retries,
                        retry_interval,
                        concurrent_limit,
                    } => {
                        let addresses = targets
                            .iter()
                            .map(|s| IpNetwork::from_str(s))
                            .collect::<Result<_, _>>()?;

                        let (tx, mut rx) =
                            mpsc::channel::<service_detection::udp::UdpServiceDetectionResult>(1);

                        task::spawn(async move {
                            while let Some(result) = rx.recv().await {
                                info!(
                                    "detected service on {}:{}: {:?}",
                                    result.address, result.port, result.service
                                );
                            }
                        });

                        if let Err(err) = service_detection::udp::start_udp_service_detection(
                            &service_detection::udp::UdpServiceDetectionSettings {
                                addresses,
                                ports: utils::parse_ports(&ports, Some(1..=u16::MAX))?,
                                max_retries: port_retries,
                                retry_interval: Duration::from_millis(retry_interval),
                                timeout: Duration::from_millis(timeout),
                                concurrent_limit: u32::from(concurrent_limit),
                            },
                            tx,
                        )
                        .await
                        {
                            error!("{err}");
                        }
                    }
                    RunCommand::TcpFingerprint { ip, port, timeout } => {
                        let fp = fingerprint_tcp(
                            SocketAddr::new(ip, port),
                            Duration::from_millis(timeout),
                        )
                        .await?;
                        println!("Fingerprint: {fp}");
                    }
                    RunCommand::OsDetection {
                        ip,
                        total_timeout,
                        timeout,
                        port_timeout,
                        ssh_port,
                    } => {
                        let os = tokio::time::timeout(
                            Duration::from_millis(total_timeout),
                            os_detection(OsDetectionSettings {
                                ip_addr: ip,
                                fingerprint_port: None,
                                fingerprint_timeout: Duration::from_millis(timeout),
                                ssh_port: Some(ssh_port),
                                ssh_connect_timeout: Duration::from_millis(timeout) / 2,
                                ssh_timeout: Duration::from_millis(timeout),
                                port_ack_timeout: Duration::from_millis(port_timeout),
                                port_parallel_syns: 8,
                            }),
                        )
                        .await?;

                        match os {
                            Ok(os) => {
                                println!("OS detection result:");
                                println!("- likely OS: {}", os);
                                let hints = os.hints();
                                if !hints.is_empty() {
                                    println!("- hints:");
                                    for hint in hints {
                                        println!("\t- {hint}");
                                    }
                                }
                            }
                            Err(err) => {
                                println!("Failed detecting OS: {err}")
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn migrate(config_path: &str, migration_dir: String) -> Result<(), Box<dyn Error>> {
    let config = get_config(config_path)?;
    cli::migrate::run_migrate_custom(
        cli::config::DatabaseConfig {
            last_migration_table_name: None,
            driver: cli::config::DatabaseDriver::Postgres {
                host: config.database.host,
                port: config.database.port,
                name: config.database.name,
                user: config.database.user,
                password: config.database.password,
            },
        },
        migration_dir,
        false,
        None,
    )
    .await?;
    Ok(())
}

async fn get_db(config: &Config) -> Result<Database, String> {
    // TODO: make driver configurable...?
    let db_config = DatabaseConfiguration {
        driver: DatabaseDriver::Postgres {
            host: config.database.host.clone(),
            port: config.database.port,
            user: config.database.user.clone(),
            password: config.database.password.clone(),
            name: config.database.name.clone(),
        },
        min_connections: 2,
        max_connections: 20,
        disable_logging: Some(true),
        statement_log_level: None,
        slow_statement_log_level: None,
    };

    Database::connect(db_config)
        .await
        .map_err(|e| format!("Error connecting to the database: {e}"))
}
