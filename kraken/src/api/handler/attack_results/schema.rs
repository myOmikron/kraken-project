use std::net::IpAddr;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::hosts::schema::OsType;
use crate::api::handler::services::schema::ServiceCertainty;

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

/// Representation of an OS detection result
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullOsDetectionResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The ip address a port was found on
    #[schema(value_type = String, example = "127.0.0.1")]
    pub host: IpNetwork,

    /// The detected operating system
    pub os: OsType,

    /// Optional human-readable hints, newline separated (\n)
    pub hints: String,

    /// Optional detected version numbers, separated by OR (`" OR "`)
    pub version: String,
}

/// The type of DNS Record
#[derive(Copy, Clone, Debug, Deserialize, Serialize, ToSchema)]
pub enum DnsRecordType {
    /// [A](crate::rpc::rpc_definitions::shared::dns_record::Record::A) record type
    A,
    /// [Aaaa](crate::rpc::rpc_definitions::shared::dns_record::Record::Aaaa) record type
    Aaaa,
    /// [Caa](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Caa,
    /// [Cname](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Cname,
    /// [Mx](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Mx,
    /// [Tlsa](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Tlsa,
    /// [Txt](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Txt,
}

/// The type of DNS TXT scan result for service hints
#[derive(Copy, Clone, Debug, Deserialize, Serialize, ToSchema)]
pub enum DnsTxtScanServiceHintType {
    /// Domain owner might have or use a google account
    HasGoogleAccount,
    /// Domain owner might have or use a globalsign account
    HasGlobalsignAccount,
    /// Domain owner might have or use globalsign smime email service
    HasGlobalsignSMime,
    /// Domain owner might have or use a docusign account
    HasDocusignAccount,
    /// Domain owner might have or use a apple account
    HasAppleAccount,
    /// Domain owner might have or use a facebook account
    HasFacebookAccount,
    /// Domain owner might have or use a hubspot account
    HasHubspotAccount,
    /// Domain owner might have or use a microsoft account with MS Dyancmis 365
    HasMSDynamics365,
    /// Domain owner might have or use a stripe account
    HasStripeAccount,
    /// Domain owner might have or use a onetrust sso
    HasOneTrustSso,
    /// Domain owner might have or use a brevo account
    HasBrevoAccount,
    /// Can manage Atlassian accounts with emails with this domain
    OwnsAtlassianAccounts,
    /// Can manage Zoom accounts with emails with this domain
    OwnsZoomAccounts,
    /// E-Mail might be managed by ProtonMail
    EmailProtonMail,
}

/// The type of DNS TXT scan result for SPF rules
#[derive(Copy, Clone, Debug, Deserialize, Serialize, ToSchema)]
pub enum DnsTxtScanSpfType {
    /// SPF part: 'all' directive, no other fields set.
    All,
    /// SPF part: 'include:DOMAIN' directive, sets `DnsTxtScanSpfEntry::spf_domain`.
    /// Directive to tell SPF parsers to lookup the referenced DNS entry.
    Include,
    /// SPF part: 'a[:DOMAIN][/32][//128]' directive, sets `DnsTxtScanSpfEntry::spf_domain`.
    /// Directive that allows the A/AAAA IPs under the specified domain to send mails.
    A,
    /// SPF part: 'mx[:DOMAIN][/32][//128]' directive, sets `DnsTxtScanSpfEntry::spf_domain`.
    /// Directive that allows the MX IP under the specified domain to send mails.
    Mx,
    /// SPF part: 'ptr[:DOMAIN]' directive, sets `DnsTxtScanSpfEntry::spf_domain`.
    /// Deprecated, but may allow PTR IPs under the specified domain to send mails.
    Ptr,
    /// SPF part: 'ip4:IP' and 'ip6:IP' directive, sets `DnsTxtScanSpfEntry::spf_ip`.
    /// Allows the exact given IPs or networks to send mails.
    Ip,
    /// SPF part: 'exists:DOMAIN', sets `DnsTxtScanSpfEntry::spf_domain`.
    /// Only allows sending mails if the given DOMAIN resolves to any address.
    Exists,
    /// SPF modifier: 'redirect=DOMAIN', sets `DnsTxtScanSpfEntry::spf_domain`.
    /// Query the given DOMAIN in case no match rules.
    Redirect,
    /// SPF modifier: 'exp=DOMAIN', sets `DnsTxtScanSpfEntry::spf_domain`.
    /// Query the given DOMAIN to see human readable text explaining the SPF rules.
    Explanation,
    /// SPF modifier: 'KEY=VALUE'.
    /// Syntax for future modifiers. Doesn't set domain or ip.
    Modifier,
}

/// Indicates what children the DnsTxtScanAttackResult has
#[derive(Copy, Clone, Debug, Deserialize, Serialize, ToSchema)]
pub enum DnsTxtScanSummaryType {
    /// Site verifications, domain keys, etc. that indicate possibly used services
    ServiceHints,
    /// SPF records controlling how email is supposed to be handled.
    Spf,
}

/// The results of a `testssl.sh` scan
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullTestSSLResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The domain which was used for SNI and certificate verification
    pub domain: Option<String>,

    /// The scanned ip address
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip: IpAddr,

    /// The scanned port
    pub port: u16,

    /// The ip address' rDNS name
    pub rdns: String,

    /// The detected service
    pub service: String,

    /// The scan's findings
    ///
    /// This includes, log messages, extracted information (for example cert parameters) and tests for vulnerabilities / bad options.
    pub findings: Vec<TestSSLFinding>,
}

/// A single finding reported by `testssl.sh`
///
/// This includes, log messages, extracted information (for example cert parameters) and tests for vulnerabilities / bad options.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct TestSSLFinding {
    /// The section `testssl.sh` reported this finding under
    pub section: TestSSLSection,

    /// The finding's id (not db id, but `testssl.sh` id)
    pub id: String,

    /// The finding's value (the value's semantics are highly dependant on the `id` and `severity`)
    pub value: String,

    /// The severity reported by `testssl.sh` (this also includes log levels)
    pub severity: TestSSLSeverity,

    /// An associated cve
    pub cve: Option<String>,

    /// An associated cwe category
    pub cwe: Option<String>,

    /// An issue categorized by kraken TODO
    pub issue: (),
}

/// A [`TestSSLResultFinding`]'s severity
#[derive(Deserialize, Serialize, ToSchema, Copy, Clone, Debug)]
pub enum TestSSLSeverity {
    /// A debug level log message
    Debug,
    /// An info level log message
    Info,
    /// A warning level log message
    Warn,
    /// An error level log message
    Fatal,

    /// The test's result doesn't pose an issue
    Ok,
    /// The test's result pose a low priority issue
    Low,
    /// The test's result pose a medium priority issue
    Medium,
    /// The test's result pose a high priority issue
    High,
    /// The test's result pose a critical priority issue
    Critical,
}

/// A [`TestSSLResultFinding`]'s section
#[derive(Deserialize, Serialize, ToSchema, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum TestSSLSection {
    /// Some sanity checks which can't be disabled
    Pretest = 0,

    /// Which tls protocols are supported
    Protocols = 1,

    /// Server implementation bugs and [GREASE](https://www.ietf.org/archive/id/draft-ietf-tls-grease-01.txt)
    Grease = 2,

    /// Which cipher suites are supported
    Ciphers = 3,

    /// Checks robust (perfect) forward secrecy key exchange
    Pfs = 4,

    /// The server's preferences
    ServerPreferences = 5,

    /// The server's defaults
    ServerDefaults = 6,

    /// The http header set by the server
    HeaderResponse = 7,

    /// List of several vulnerabilities
    Vulnerabilities = 8,

    /// Which concrete ciphers are supported
    ///
    /// Depending on the option `testssl` is invoked with,
    /// this is either a list of all ciphers or a list of all cipher per tls protocol.
    CipherTests = 9,

    /// Which browser is able to establish a connection
    BrowserSimulations = 10,
}
