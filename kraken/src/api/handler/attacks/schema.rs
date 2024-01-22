use std::net::IpAddr;
use std::ops::RangeInclusive;

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::models::AttackType;

/// The settings of a subdomain bruteforce request
#[derive(Deserialize, ToSchema)]
pub struct BruteforceSubdomainsRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// Domain to construct subdomains for
    #[schema(example = "example.com")]
    pub domain: String,

    /// The wordlist to use
    pub wordlist_uuid: Uuid,

    /// The concurrent task limit
    #[schema(example = 100)]
    pub concurrent_limit: u32,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// The settings to configure a tcp port scan
#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct ScanTcpPortsRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The ip addresses / networks or domains to scan
    #[schema(value_type = Vec<String>, example = json!(["10.13.37.1", "10.13.37.0/24", "google.com"]))]
    pub targets: Vec<DomainOrNetwork>,

    /// List of single ports and port ranges
    ///
    /// If no values are supplied, 1-65535 is used as default
    #[serde(default)]
    pub ports: Vec<PortOrRange>,

    /// The interval that should be wait between retries on a port.
    ///
    /// The interval is specified in milliseconds.
    #[schema(example = 100)]
    pub retry_interval: u64,

    /// The number of times the connection should be retried if it failed.
    #[schema(example = 2)]
    pub max_retries: u32,

    /// The time to wait until a connection is considered failed.
    ///
    /// The timeout is specified in milliseconds.
    #[schema(example = 3000)]
    pub timeout: u64,

    /// The concurrent task limit
    #[schema(example = 5000)]
    pub concurrent_limit: u32,

    /// Skips the initial icmp check.
    ///
    /// All hosts are assumed to be reachable
    #[schema(example = false)]
    pub skip_icmp_check: bool,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// Single port or a range of ports
#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(untagged)]
pub enum PortOrRange {
    /// A single port
    #[schema(example = 8000)]
    Port(u16),
    /// In inclusive range of ports
    #[schema(value_type = String, example = "1-1024")]
    Range(#[serde(deserialize_with = "deserialize_port_range")] RangeInclusive<u16>),
}

/// Host Alive check request
#[derive(Deserialize, ToSchema)]
pub struct HostsAliveRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The ip addresses / networks or domains to scan
    #[schema(value_type = Vec<String>, example = json!(["10.13.37.1", "10.13.37.0/24", "google.com"]))]
    pub targets: Vec<DomainOrNetwork>,

    /// The time to wait until a host is considered down.
    ///
    /// The timeout is specified in milliseconds.
    #[schema(example = 3000)]
    pub timeout: u64,

    /// The concurrent task limit
    #[schema(example = 30)]
    pub concurrent_limit: u32,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// The settings to configure a certificate transparency request
#[derive(Deserialize, ToSchema)]
pub struct QueryCertificateTransparencyRequest {
    /// Domain to query certificates for
    #[schema(example = "example.com")]
    pub target: String,

    /// Should expired certificates be included as well
    #[schema(example = true)]
    pub include_expired: bool,

    /// The number of times the query should be retried if it failed.
    #[schema(example = 3)]
    pub max_retries: u32,

    /// The interval that should be waited between retries.
    ///
    /// The interval is specified in milliseconds.
    #[schema(example = 500)]
    pub retry_interval: u64,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// The request to query the dehashed API
#[derive(ToSchema, Deserialize)]
pub struct QueryDehashedRequest {
    /// The query to send to dehashed
    #[schema(value_type = Query)]
    pub query: dehashed_rs::Query,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// The request to start a service detection
#[derive(Debug, ToSchema, Deserialize, Serialize)]
pub struct ServiceDetectionRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The ip address the service listens on
    #[schema(value_type = String, example = "10.13.37.1")]
    pub address: IpAddr,

    /// The port the service listens on
    #[schema(example = 443)]
    pub port: u16,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
    ///
    /// The timeout is specified in milliseconds.
    #[schema(example = 3000)]
    pub timeout: u64,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// The request to start a service detection
#[derive(Debug, ToSchema, Deserialize, Serialize)]
pub struct UdpServiceDetectionRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The ip address the service listens on
    #[schema(value_type = String, example = "10.13.37.1")]
    pub address: IpAddr,

    /// List of single ports and port ranges
    ///
    /// If no values are supplied, 1-65535 is used as default
    #[serde(default)]
    pub ports: Vec<PortOrRange>,

    /// The interval that should be wait between retries on a port.
    ///
    /// The interval is specified in milliseconds.
    #[schema(example = 100)]
    pub retry_interval: u64,

    /// The number of times the connection should be retried if it failed.
    #[schema(example = 2)]
    pub max_retries: u32,

    /// The time to wait until a connection is considered failed.
    ///
    /// The timeout is specified in milliseconds.
    #[schema(example = 3000)]
    pub timeout: u64,

    /// The concurrent task limit
    #[schema(example = 5000)]
    pub concurrent_limit: u32,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// Request to resolve domains
#[derive(Deserialize, ToSchema)]
pub struct DnsResolutionRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The domains to resolve
    #[schema(value_type = Vec<String>, example = json!(["example.com", "example.org"]))]
    pub targets: Vec<String>,

    /// The concurrent task limit
    #[schema(example = 2)]
    pub concurrent_limit: u32,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// Request to run testssl
#[derive(Deserialize, ToSchema)]
pub struct TestSSLRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,

    /// The domain to scan
    pub uri: String,

    /// The host to scan
    #[schema(value_type = String)]
    pub host: IpAddr,

    /// The port to scan
    pub port: u16,

    /// Timeout for TCP handshakes in seconds
    pub connect_timeout: Option<u64>,

    /// Timeout for `openssl` connections in seconds
    pub openssl_timeout: Option<u64>,

    /// Set the `BASICAUTH` header when checking http headers
    pub basic_auth: Option<[String; 2]>,

    /// Run against a STARTTLS enabled protocol
    pub starttls: Option<StartTLSProtocol>,
}

/// A simple version of an attack
#[derive(Clone, Serialize, Deserialize, ToSchema, Debug)]
pub struct SimpleAttack {
    /// The identifier of the attack
    pub uuid: Uuid,
    /// The workspace this attack is attached to
    pub workspace: SimpleWorkspace,
    /// The type of attack
    pub attack_type: AttackType,
    /// The user that has started the attack
    pub started_by: SimpleUser,
    /// If this is None, the attack is still running
    pub finished_at: Option<DateTime<Utc>>,
    /// If this field is set, the attack has finished with an error
    pub error: Option<String>,
    /// The point in time this attack was started
    pub created_at: DateTime<Utc>,
}

/// A list of attacks
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct ListAttacks {
    /// The list of the attacks
    pub attacks: Vec<SimpleAttack>,
}

/// Either an ip address / network or a domain name
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
#[serde(untagged)]
pub enum DomainOrNetwork {
    /// A ip address / network
    #[schema(value_type = String, example = "10.13.37.10")]
    Network(IpNetwork),

    /// A domain name
    #[schema(value_type = String, example = "kraken.test")]
    Domain(String),
}

/// Protocols to select from when using `--starttls`
#[derive(Serialize, Deserialize, ToSchema, Debug, Copy, Clone)]
#[allow(missing_docs)] // The names are pretty unambiguous
pub enum StartTLSProtocol {
    FTP,
    SMTP,
    POP3,
    IMAP,
    XMPP,
    // Telnet, // WIP
    // LDAP,   // Requires `--ssl-native` which is less precise
    // IRC,    // WIP
    LMTP,
    NNTP,
    Postgres,
    MySQL,
}

/// Deserializes a string and parses it as `{start}-{end}` where `start` and `end` are both `u16`
pub fn deserialize_port_range<'de, D>(deserializer: D) -> Result<RangeInclusive<u16>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value
        .split_once('-')
        .and_then(|(start, end)| Some((start.parse::<u16>().ok()?)..=(end.parse::<u16>().ok()?)))
        .ok_or_else(|| {
            <D::Error as serde::de::Error>::invalid_value(serde::de::Unexpected::Str(&value), &"")
        })
}
