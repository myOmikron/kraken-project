use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{
    DnsRecordType, DnsTxtScanServiceHintType, DnsTxtScanSpfType, DnsTxtScanSummaryType,
    ServiceCertainty,
};

/// A simple representation of a bruteforce subdomains result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleBruteforceSubdomainsResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The source address
    pub source: String,

    /// The destination address
    pub destination: String,

    /// The type of DNS record
    #[schema(inline)]
    pub dns_record_type: DnsRecordType,
}

/// A simple representation of a tcp port scan result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleTcpPortScanResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The ip address a port was found on
    #[schema(value_type = String, example = "127.0.0.1")]
    pub address: IpNetwork,

    /// The found port
    pub port: u16,
}

/// A simple representation of a query certificate transparency result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullQueryCertificateTransparencyResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The name of the issuer
    pub issuer_name: String,

    /// The common name of the certificate
    pub common_name: String,

    /// The values of the certificate
    pub value_names: Vec<String>,

    /// The start date of the certificate
    pub not_before: Option<DateTime<Utc>>,

    /// The end date of the certificate
    pub not_after: Option<DateTime<Utc>>,

    /// The serial number of the certificate
    pub serial_number: String,
}

/// A simple representation of a query unhashed result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleQueryUnhashedResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// ID of the entry
    pub dehashed_id: i64,

    /// An email address
    pub email: Option<String>,

    /// An username
    pub username: Option<String>,

    /// A password
    pub password: Option<String>,

    /// An hashed password
    pub hashed_password: Option<String>,

    /// An ip address
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip_address: Option<IpNetwork>,

    /// A name
    pub name: Option<String>,

    /// A vin
    pub vin: Option<String>,

    /// An address
    pub address: Option<String>,

    /// A phone number
    pub phone: Option<String>,

    /// A database name
    pub database_name: Option<String>,
}

/// A simple representation of a host alive result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleHostAliveResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// A host that responded
    #[schema(value_type = String, example = "127.0.0.1")]
    pub host: IpNetwork,
}

/// A simple representation of a service detection result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullServiceDetectionResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The certainty of the result
    #[schema(inline)]
    pub certainty: ServiceCertainty,

    /// The found names of the service
    pub service_names: Vec<String>,

    /// The ip address a port was found on
    #[schema(value_type = String, example = "127.0.0.1")]
    pub host: IpNetwork,

    /// Port number
    pub port: u16,
}

/// A simple representation of a service detection result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullUdpServiceDetectionResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The certainty of the result
    #[schema(inline)]
    pub certainty: ServiceCertainty,

    /// The found names of the service
    pub service_names: Vec<String>,

    /// The ip address a port was found on
    #[schema(value_type = String, example = "127.0.0.1")]
    pub host: IpNetwork,

    /// Port number
    pub port: u16,
}

/// A simple representation of a dns resolution result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleDnsResolutionResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The source address
    pub source: String,

    /// The destination address
    pub destination: String,

    /// The type of DNS record
    #[schema(inline)]
    pub dns_record_type: DnsRecordType,
}

/// A simple representation of a dns txt scan result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleDnsTxtScanResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The source address
    pub domain: String,

    /// Indicates the kind of items this result entry has (e.g. ServiceHints or SPF)
    pub collection_type: DnsTxtScanSummaryType,
}

/// The full representation of a dns txt scan result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullDnsTxtScanResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The source address
    pub domain: String,

    /// Indicates the kind of items this result entry has (e.g. ServiceHints or SPF)
    pub collection_type: DnsTxtScanSummaryType,

    /// List of result entries. The kind depends on the `collection_type` in this object.
    pub entries: Vec<DnsTxtScanEntry>,
}

/// A single detailed entry for a given DNS TXT scan result. May be a hint at service usage / ownership or contain
/// parsed SPF rules.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub enum DnsTxtScanEntry {
    /// Just wraps txt_type, the DNS rule is usually more exact to what is actually written in DNS
    ServiceHint {
        /// The primary key
        uuid: Uuid,

        /// The point in time, this entry was produced
        created_at: DateTime<Utc>,

        /// The rule that was matched for this scan result, usually the whole TXT record.
        rule: String,

        /// The type of DNS record
        txt_type: DnsTxtScanServiceHintType,
    },
    /// Wraps a single SPF rule part, the rule is reconstructed from the parsed value
    Spf {
        /// The primary key
        uuid: Uuid,

        /// The point in time, this entry was produced
        created_at: DateTime<Utc>,

        /// A single SPF rule part that was matched for this object.
        rule: String,

        /// The type of DNS record
        spf_type: DnsTxtScanSpfType,

        /// If the txt_type is a SPF type that includes an IP (or whole IP range), it will be set here.
        #[schema(value_type = String, example = "127.0.0.1/24")]
        spf_ip: Option<IpNetwork>,

        /// If the txt_type is a SPF type that includes a domain, it will be set here.
        spf_domain: Option<String>,

        /// If the txt_type is a SPF type that includes a domain, this is its ipv4 CIDR.
        spf_domain_ipv4_cidr: Option<i32>,
        /// If the txt_type is a SPF type that includes a domain, this is its ipv6 CIDR.
        spf_domain_ipv6_cidr: Option<i32>,
    },
}
