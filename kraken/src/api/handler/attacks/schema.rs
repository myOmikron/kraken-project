use std::ops::RangeInclusive;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::models::AttackType;

/// The settings of a subdomain bruteforce request
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
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

/// Single port or a range of ports
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
#[serde(untagged)]
pub enum PortOrRange {
    /// A single port
    #[schema(example = 8000)]
    Port(u16),
    /// In inclusive range of ports
    #[schema(value_type = String, example = "1-1024")]
    Range(
        #[serde(
            deserialize_with = "deserialize_port_range",
            serialize_with = "serialize_port_range"
        )]
        RangeInclusive<u16>,
    ),
}

/// Host Alive check request
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
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
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
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
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct QueryDehashedRequest {
    /// The query to send to dehashed
    #[schema(value_type = Query)]
    pub query: dehashed_rs::Query,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// The request to start a service detection
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct ServiceDetectionRequest {
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

    /// The time to wait until a connection is considered failed.
    ///
    /// The timeout is specified in milliseconds.
    #[schema(example = 3000)]
    pub connect_timeout: u64,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
    ///
    /// The timeout is specified in milliseconds.
    #[schema(example = 3000)]
    pub receive_timeout: u64,

    /// The interval that should be wait between retries on a port.
    ///
    /// The interval is specified in milliseconds.
    #[schema(example = 100)]
    pub retry_interval: u64,

    /// The number of times the connection should be retried if it failed.
    #[schema(example = 2)]
    pub max_retries: u32,

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

/// The request to start a service detection
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct UdpServiceDetectionRequest {
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

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// OS detection request
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct OsDetectionRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The ip addresses / networks or domains to scan
    #[schema(value_type = Vec<String>, example = json!(["10.13.37.1", "10.13.37.0/24", "google.com"]))]
    pub targets: Vec<DomainOrNetwork>,

    /// set to skip open port detection and use this port for TCP fingerprinting
    pub fingerprint_port: Option<u32>,

    /// set to perform OS detection through SSH header
    pub ssh_port: Option<u32>,

    /// timeout for TCP fingerprint detection task, in ms
    pub fingerprint_timeout: u64,

    /// timeout for establishing an SSH connection, if ssh_port is set, in ms
    pub ssh_connect_timeout: u64,

    /// timeout for the full SSH os detection task, in ms
    pub ssh_timeout: u64,

    /// If fingerprint_port is not set, timeout for each port how long to wait for ACKs
    pub port_ack_timeout: u64,

    /// If fingerprint_port is not set, maximum parallel TCP SYN requests
    pub port_parallel_syns: u32,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,

    /// The concurrent task limit
    #[schema(example = 5000)]
    pub concurrent_limit: u32,
}

/// Request to resolve domains
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
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

/// Request to do DNS TXT scanning & parsing
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct DnsTxtScanRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The domains to resolve
    #[schema(value_type = Vec<String>, example = json!(["example.com", "example.org"]))]
    pub targets: Vec<String>,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
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

/// Serializes a string as `{start}-{end}` where `start` and `end` are both `u16`
pub fn serialize_port_range<S>(port_or_range: &RangeInclusive<u16>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!(
        "{start}-{end}",
        start = port_or_range.start(),
        end = port_or_range.end()
    ))
}
