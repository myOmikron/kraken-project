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
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::ops::ControlFlow;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use std::time::Duration;

use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;
use dehashed_rs::SearchType;
use hickory_resolver::Name;
use ipnetwork::IpNetwork;
use kraken_proto::push_attack_service_client::PushAttackServiceClient;
use kraken_proto::PushAttackRequest;
use log::error;
use log::info;
use rorm::cli;
use rorm::Database;
use rorm::DatabaseConfiguration;
use tokio::sync::mpsc;
use tokio::task;
use tonic::transport::Endpoint;
use uuid::Uuid;

use crate::backlog::start_backlog;
use crate::config::Config;
use crate::config::DB;
use crate::modules::bruteforce_subdomains::BruteforceSubdomain;
use crate::modules::bruteforce_subdomains::BruteforceSubdomainsSettings;
use crate::modules::certificate_transparency::CertificateTransparency;
use crate::modules::certificate_transparency::CertificateTransparencySettings;
use crate::modules::dehashed;
use crate::modules::dns::txt::DnsTxtScan;
use crate::modules::dns::txt::DnsTxtScanSettings;
use crate::modules::host_alive::icmp_scan::IcmpScan;
use crate::modules::host_alive::icmp_scan::IcmpScanSettings;
use crate::modules::os_detection::tcp_fingerprint::fingerprint_tcp;
use crate::modules::os_detection::OsDetection;
use crate::modules::os_detection::OsDetectionSettings;
use crate::modules::service_detection::tcp::TcpServiceDetection;
use crate::modules::service_detection::tcp::TcpServiceDetectionSettings;
use crate::modules::service_detection::udp::UdpServiceDetection;
use crate::modules::service_detection::udp::UdpServiceDetectionSettings;
use crate::modules::testssl::TestSSL;
use crate::modules::testssl::TestSSLSettings;
use crate::modules::whois;
use crate::modules::Attack;
use crate::modules::StreamedAttack;
use crate::rpc::start_rpc_server;
use crate::utils::input;
use crate::utils::kraken_endpoint;

pub mod backlog;
pub mod config;
pub mod models;
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

    /// Run `testssl.sh`
    TestSSL {
        /// The ip address to scan
        ip: IpAddr,

        /// The port to scan
        #[clap(default_value_t = 443)]
        port: u16,

        /// Domain to scan
        domain: Option<String>,
    },
}

/// All available subcommands
#[derive(Subcommand)]
pub enum Command {
    /// Start the leech as a server
    Server,
    /// Execute a command via CLI
    Execute {
        /// Specify an alternative path to the config file
        #[clap(long = "config-path")]
        #[clap(default_value = "/etc/leech/config.toml")]
        config_path: PathBuf,

        /// Specifies the verbosity of the output
        #[clap(short = 'v', global = true, action = ArgAction::Count)]
        verbosity: u8,

        /// Push the results to a workspace in kraken
        #[clap(long)]
        push: Option<Uuid>,

        /// Api key to authenticate when pushing
        #[clap(long)]
        api_key: Option<String>,

        /// Output the results as json
        #[clap(long)]
        json: bool,

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
#[clap(version)]
pub struct Cli {
    /// Subcommands
    #[clap(subcommand)]
    commands: Command,
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.commands {
        Command::Execute { .. } => {}
        Command::Migrate { .. } | Command::Server => {
            if let Err(errors) = config::load_env() {
                for error in errors {
                    eprintln!("{error}");
                }
                return Err("Failed to load configuration".into());
            }
        }
    }

    match cli.commands {
        Command::Migrate { migration_dir } => migrate(migration_dir).await?,
        Command::Server => {
            env_logger::init();

            let db = get_db().await?;
            let backlog = start_backlog(db).await?;

            start_rpc_server(backlog).await?;
        }
        Command::Execute {
            config_path,
            json,
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

            let push = if let Some(workspace) = push {
                if !config_path.exists() {
                    return Err("The config file does not exist at the specified path".into());
                }

                if !config_path.is_file() {
                    return Err("The config file is a directory".into());
                }

                let config: Config = toml::from_str(
                    &fs::read_to_string(&config_path)
                        .map_err(|e| format!("io error while reading the config file: {e}"))?,
                )
                .map_err(|e| format!("The config file contains invalid TOML: {e}"))?;
                let endpoint = kraken_endpoint(&config.kraken)?;

                let api_key = if let Some(api_key) = api_key {
                    api_key
                } else {
                    print!("Please enter your api key: ");
                    std::io::stdout().flush().unwrap();
                    input()
                        .await?
                        .ok_or_else(|| "Can't push to kraken without api key".to_string())?
                };
                Some((endpoint, workspace, api_key))
            } else {
                None
            };

            {
                match command {
                    RunCommand::BruteforceSubdomains {
                        target,
                        wordlist_path,
                        concurrent_limit,
                    } => {
                        run_streamed_attack::<BruteforceSubdomain>(
                            BruteforceSubdomainsSettings {
                                domain: target.to_string(),
                                wordlist_path,
                                concurrent_limit: u32::from(concurrent_limit),
                            },
                            push,
                            json,
                        )
                        .await?;
                    }
                    RunCommand::DnsTxt { target } => {
                        run_streamed_attack::<DnsTxtScan>(
                            DnsTxtScanSettings {
                                domains: Vec::from([target.to_string()]),
                            },
                            push,
                            json,
                        )
                        .await?;
                    }
                    RunCommand::CertificateTransparency {
                        target,
                        include_expired,
                        max_retries,
                        retry_interval,
                    } => {
                        run_normal_attack::<CertificateTransparency>(
                            CertificateTransparencySettings {
                                target,
                                include_expired,
                                max_retries,
                                retry_interval: Duration::from_millis(retry_interval as u64),
                            },
                            push,
                            json,
                        )
                        .await?;
                    }
                    RunCommand::IcmpScanner {
                        targets,
                        timeout,
                        concurrent_limit,
                    } => {
                        run_streamed_attack::<IcmpScan>(
                            IcmpScanSettings {
                                addresses: targets
                                    .iter()
                                    .map(|s| IpNetwork::from_str(s))
                                    .collect::<Result<_, _>>()?,
                                timeout: Duration::from_millis(timeout as u64),
                                concurrent_limit: u32::from(concurrent_limit),
                            },
                            push,
                            json,
                        )
                        .await?;
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
                        run_streamed_attack::<TcpServiceDetection>(
                            TcpServiceDetectionSettings {
                                addresses: targets
                                    .iter()
                                    .map(|s| IpNetwork::from_str(s))
                                    .collect::<Result<_, _>>()?,
                                ports: utils::parse_ports(&ports, Some(1..=u16::MAX))?,
                                connect_timeout: Duration::from_millis(connect_timeout as u64),
                                receive_timeout: Duration::from_millis(receive_timeout as u64),
                                max_retries,
                                retry_interval: Duration::from_millis(retry_interval as u64),
                                concurrent_limit,
                                skip_icmp_check,
                                just_scan,
                            },
                            push,
                            json,
                        )
                        .await?;
                    }
                    RunCommand::ServiceDetectionUdp {
                        targets,
                        ports,
                        timeout,
                        port_retries,
                        retry_interval,
                        concurrent_limit,
                    } => {
                        run_streamed_attack::<UdpServiceDetection>(
                            UdpServiceDetectionSettings {
                                addresses: targets
                                    .iter()
                                    .map(|s| IpNetwork::from_str(s))
                                    .collect::<Result<_, _>>()?,
                                ports: utils::parse_ports(&ports, Some(1..=u16::MAX))?,
                                max_retries: port_retries,
                                retry_interval: Duration::from_millis(retry_interval),
                                timeout: Duration::from_millis(timeout),
                                concurrent_limit: u32::from(concurrent_limit),
                            },
                            push,
                            json,
                        )
                        .await?;
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
                        timeout,
                        port_timeout,
                        ssh_port,
                    } => {
                        run_streamed_attack::<OsDetection>(
                            OsDetectionSettings {
                                addresses: vec![ip.into()],
                                fingerprint_port: None,
                                fingerprint_timeout: Duration::from_millis(timeout),
                                ssh_port: Some(ssh_port),
                                ssh_connect_timeout: Duration::from_millis(timeout) / 2,
                                ssh_timeout: Duration::from_millis(timeout),
                                port_ack_timeout: Duration::from_millis(port_timeout),
                                port_parallel_syns: 8,
                                concurrent_limit: 0,
                            },
                            push,
                            json,
                        )
                        .await?;
                    }
                    RunCommand::TestSSL { domain, ip, port } => {
                        run_normal_attack::<TestSSL>(
                            TestSSLSettings {
                                domain,
                                ip,
                                port,
                                ..Default::default()
                            },
                            push,
                            json,
                        )
                        .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn migrate(migration_dir: String) -> Result<(), Box<dyn Error>> {
    cli::migrate::run_migrate_custom(
        cli::config::DatabaseConfig {
            last_migration_table_name: None,
            driver: DB.clone(),
        },
        migration_dir,
        false,
        None,
    )
    .await?;
    Ok(())
}

async fn get_db() -> Result<Database, String> {
    // TODO: make driver configurable...?
    let db_config = DatabaseConfiguration {
        driver: DB.clone(),
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

async fn run_normal_attack<A: Attack>(
    settings: A::Settings,
    push: Option<(Endpoint, Uuid, String)>,
    json: bool,
) -> Result<(), Box<dyn Error>> {
    let output = A::execute(settings).await?;

    if json {
        println!("{}", serde_json::to_string(&output)?);
    } else {
        A::print_output(&output);
    }

    if let Some((endpoint, workspace, api_key)) = push {
        if ask_push_confirmation(&output)?.is_continue() {
            let mut kraken = PushAttackServiceClient::connect(endpoint).await?;
            kraken
                .push_attack(PushAttackRequest {
                    workspace_uuid: workspace.to_string(),
                    api_key,
                    response: Some(A::wrap_for_push(A::encode_output(output))),
                })
                .await?;
        }
    }

    Ok(())
}

async fn run_streamed_attack<A: StreamedAttack>(
    settings: A::Settings,
    push: Option<(Endpoint, Uuid, String)>,
    json: bool,
) -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel::<A::Output>(1);

    let should_collect = push.is_some() || json;
    let collector = task::spawn(async move {
        let mut outputs = Vec::new();
        while let Some(output) = rx.recv().await {
            if !json {
                A::print_output(&output);
            }
            if should_collect {
                outputs.push(output);
            }
        }
        outputs
    });

    A::execute(settings, tx).await?;
    let outputs = collector.await?;

    if json {
        println!("{}", serde_json::to_string(&outputs)?);
    }

    if let Some((endpoint, workspace, api_key)) = push {
        if ask_push_confirmation(&outputs)?.is_continue() {
            let mut kraken = PushAttackServiceClient::connect(endpoint).await?;
            kraken
                .push_attack(PushAttackRequest {
                    workspace_uuid: workspace.to_string(),
                    api_key,
                    response: Some(A::wrap_for_push(
                        outputs.into_iter().map(A::encode_output).collect(),
                    )),
                })
                .await?;
        }
    }

    Ok(())
}

fn ask_push_confirmation(data: &impl fmt::Debug) -> io::Result<ControlFlow<()>> {
    let pager = env::var("PAGER")
        .ok()
        .or_else(|| {
            Path::new("/usr/bin/pager")
                .exists()
                .then_some("/usr/bin/pager".to_string())
        })
        .unwrap_or_else(|| "less".to_string());

    loop {
        print!("Do you want to push these results? [y/N/p/pp/?]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input = input.trim().to_ascii_lowercase();

        match input.as_str() {
            "" | "n" => return Ok(ControlFlow::Break(())),
            "y" => return Ok(ControlFlow::Continue(())),
            "p" | "pp" => {
                let mut process = std::process::Command::new(&pager)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::inherit())
                    .spawn()?;

                if input.len() == 1 {
                    write!(process.stdin.take().unwrap(), "{data:?}")?;
                } else {
                    write!(process.stdin.take().unwrap(), "{data:#?}")?;
                }

                process.wait()?;
            }
            "?" => {
                println!("y  - yes");
                println!("n  - no");
                println!("p  - print data to push");
                println!("pp - pretty print data to push");
                println!("?  - show this message");
            }
            _ => println!("Unknown option"),
        }
    }
}
