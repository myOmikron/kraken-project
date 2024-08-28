//! This module holds all the information regarding attacks

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use rorm::prelude::*;
use rorm::Model;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[cfg(feature = "bin")]
pub(crate) use crate::models::attack::operations::*;
#[cfg(feature = "bin")]
pub(crate) use crate::models::attack::patches::*;
use crate::models::OsType;
use crate::models::ServiceCertainty;
use crate::models::User;
use crate::models::Workspace;

mod convert;
#[cfg(feature = "bin")]
mod operations;
#[cfg(feature = "bin")]
mod patches;

/// The type of an attack
#[derive(Debug, Copy, Clone, DbEnum, ToSchema, Serialize, Deserialize)]
pub enum AttackType {
    /// First variant to be mapped for 0
    Undefined,
    /// Bruteforce subdomains via DNS requests
    BruteforceSubdomains,

    /// Effectively deleted, but postgres can't delete enum variants
    #[serde(skip)]
    TcpPortScan,

    /// Query certificate transparency
    QueryCertificateTransparency,
    /// Query the unhashed API
    QueryUnhashed,
    /// Check if a host is reachable via icmp
    HostAlive,
    /// Detect the service that is running on a port
    ServiceDetection,
    /// Detect UDP services running on a host
    UdpServiceDetection,
    /// Resolve domain names
    DnsResolution,
    /// Resolve domain names
    DnsTxtScan,
    /// Scan udp ports
    UdpPortScan,
    /// Bruteforce your way through an http service
    ForcedBrowsing,
    /// Detect a host's OS
    OSDetection,
    /// Detect a service's version
    VersionDetection,
    /// Detect an anti port scan system
    AntiPortScanningDetection,
    /// Run `testssl.sh` to check a servers TLS configuration
    TestSSL,
}

/// Representation of an attack
///
/// If the attack is still running, `finished_at` is `None`.
/// If `error` is not `None`, the attack has finished with errors.
#[derive(Model, Clone)]
pub struct Attack {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [type](AttackType) of the attack.
    pub attack_type: AttackType,

    /// The user that started this attack
    pub started_by: ForeignModel<User>,

    /// The workspace this attack was started from
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this attack has finished
    pub finished_at: Option<DateTime<Utc>>,

    /// Contains an error message if the attack didn't finish successfully
    #[rorm(max_length = 255)]
    pub error: Option<String>,

    /// The point in time, this attack was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The type of DNS Record
#[derive(Copy, Clone, Debug, DbEnum, Deserialize, Serialize, ToSchema)]
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

/// Representation of a [Bruteforce Subdomain](AttackType::BruteforceSubdomains) attack's result
pub type BruteforceSubdomainsResult = DnsRecordResult;

/// Representation of a [DNS resolution](AttackType::DnsResolution) attack's result
pub type DnsResolutionResult = DnsRecordResult;

/// Generic representation of a DNS result
#[derive(Model)]
pub struct DnsRecordResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The source address
    #[rorm(max_length = 255)]
    pub source: String,

    /// The destination address
    #[rorm(max_length = 255)]
    pub destination: String,

    /// The type of [DNS record type](DnsRecordType)
    pub dns_record_type: DnsRecordType,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// Representation of a [dns txt scan](AttackType::DnsTxtScan) attack's result.
/// Collection of detailed txt record info entries.
#[derive(Model)]
pub struct DnsTxtScanAttackResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The source domain
    #[rorm(max_length = 255)]
    pub domain: String,

    /// Indicates what kind of collection this is / what items will be attached to this.
    /// `ServiceHints` means children should be fetched from `DnsTxtScanServiceHintEntry`
    /// `Spf` means children should be fetched from `DnsTxtScanSpfEntry`
    pub collection_type: DnsTxtScanSummaryType,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// Indicates what children the DnsTxtScanAttackResult has
#[derive(Copy, Clone, Debug, DbEnum, Deserialize, Serialize, ToSchema)]
pub enum DnsTxtScanSummaryType {
    /// Site verifications, domain keys, etc. that indicate possibly used services
    ServiceHints,
    /// SPF records controlling how email is supposed to be handled.
    Spf,
}

/// The type of DNS TXT scan result for service hints
#[derive(Copy, Clone, Debug, DbEnum, Deserialize, Serialize, ToSchema)]
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
#[derive(Copy, Clone, Debug, DbEnum, Deserialize, Serialize, ToSchema)]
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

/// Part of a DnsTxtScanAttackResult of type ServiceHints
#[derive(Model)]
pub struct DnsTxtScanServiceHintEntry {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The result collection this is a part of
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub collection: ForeignModel<DnsTxtScanAttackResult>,

    /// The TXT record part that was matched for this scan result
    #[rorm(max_length = 255)]
    pub rule: String,

    /// The type of this result part.
    pub txt_type: DnsTxtScanServiceHintType,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// Part of a DnsTxtScanAttackResult of type Spf
#[derive(Model)]
pub struct DnsTxtScanSpfEntry {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The result collection this is a part of
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub collection: ForeignModel<DnsTxtScanAttackResult>,

    /// String representation of the SPF rule part representing this entry
    #[rorm(max_length = 255)]
    pub rule: String,

    /// The type of this result part.
    pub spf_type: DnsTxtScanSpfType,

    /// Set when txt_type is SpfIp, designates allowed / disallowed IPs that may
    /// send emails. None if this is not an SPF entry.
    pub spf_ip: Option<IpNetwork>,

    /// A domain to look up more rules in as referenced in the SPF part or that
    /// may be allowed as sender. None if this is not an SPF entry.
    #[rorm(max_length = 255)]
    pub spf_domain: Option<String>,

    /// For SPF domains, the IP prefix / subnet mask length how many of the
    /// resolved IPs should match. (CIDR)
    pub spf_domain_ipv4_cidr: Option<i32>,
    /// ditto
    pub spf_domain_ipv6_cidr: Option<i32>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// Representation of a [dehashed query](AttackType::Dehashed) result
#[derive(Model)]
pub struct DehashedQueryResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// ID of the entry
    pub dehashed_id: i64,
    /// An email address, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub email: Option<String>,
    /// An username, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub username: Option<String>,
    /// A password, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub password: Option<String>,
    /// An hashed password, may be [None] if the result didn't include this field
    #[rorm(max_length = 8192)]
    pub hashed_password: Option<String>,
    /// An ip address, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub ip_address: Option<IpNetwork>,
    /// A name, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub name: Option<String>,
    /// A vin, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub vin: Option<String>,
    /// An address, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub address: Option<String>,
    /// A phone, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub phone: Option<String>,
    /// A database name, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub database_name: Option<String>,
}

/// A value name in a [AttackType::QueryCertificateTransparency] result
#[derive(Model)]
pub struct CertificateTransparencyValueName {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// a single value
    #[rorm(max_length = 255)]
    pub value_name: String,

    /// The result this value is originating from
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub ct_result: ForeignModel<CertificateTransparencyResult>,
}

/// Representation of a [AttackType::QueryCertificateTransparency] result
#[derive(Model)]
pub struct CertificateTransparencyResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The name of the issuer
    #[rorm(max_length = 255)]
    pub issuer_name: String,
    /// The common name of the certificate
    #[rorm(max_length = 255)]
    pub common_name: String,
    /// The values of the certificate
    pub value_names: BackRef<field!(CertificateTransparencyValueName::F.ct_result)>,
    /// The start date of the certificate
    pub not_before: Option<DateTime<Utc>>,
    /// The end date of the certificate
    pub not_after: Option<DateTime<Utc>>,
    /// The serial number of the certificate
    #[rorm(max_length = 255)]
    pub serial_number: String,
}

/// Representation of a [Host Alive](AttackType::HostAlive) attack's result
#[derive(Model)]
pub struct HostAliveResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// A host that responded
    pub host: IpNetwork,
}

/// The name of a result of a service that was found during a service detection
#[derive(Model)]
pub struct ServiceDetectionName {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The name of found service
    #[rorm(max_length = 255)]
    pub name: String,

    /// The result this service name is linked to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub result: ForeignModel<ServiceDetectionResult>,
}

/// Representation of a [Service Detection](AttackType::ServiceDetection) attack's result
#[derive(Model)]
pub struct ServiceDetectionResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The certainty of the result
    pub certainty: ServiceCertainty,

    /// The ip address a port was found on
    pub host: IpNetwork,

    /// Port number
    pub port: i32,

    /// The found names of the service
    pub service_names: BackRef<field!(ServiceDetectionName::F.result)>,
}

/// The name of a result of a service that was found during a service detection
#[derive(Model)]
pub struct UdpServiceDetectionName {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The name of found service
    #[rorm(max_length = 255)]
    pub name: String,

    /// The result this service name is linked to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub result: ForeignModel<UdpServiceDetectionResult>,
}

/// Representation of a [UDP Service Detection](AttackType::UdpServiceDetection) attack's result
#[derive(Model)]
pub struct UdpServiceDetectionResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// A host that responded
    pub host: IpNetwork,

    /// Port number
    pub port: i32,

    /// The certainty of the result
    pub certainty: ServiceCertainty,

    /// The found names of the service
    pub service_names: BackRef<field!(UdpServiceDetectionName::F.result)>,
}
/// Meta information about a single `testssl.sh` scan's results
///
/// The actual results are stored in [`TestSSLResultFinding`].
#[derive(Model)]
pub struct TestSSLResultHeader {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The domain which was used for SNI and certificate verification
    #[rorm(max_length = 255)]
    pub domain: Option<String>,

    /// The scanned ip address
    pub ip: IpNetwork,

    /// The scanned port
    pub port: i32,

    /// The ip address' rDNS name
    #[rorm(max_length = 255)]
    pub rdns: String,

    /// The detected service
    #[rorm(max_length = 255)]
    pub service: String,
}

/// A single finding reported by `testssl.sh`
///
/// This includes, log messages, extracted information (for example cert parameters) and tests for vulnerabilities / bad options.
#[derive(Model)]
pub struct TestSSLResultFinding {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    // TODO: shouldn't this be associated with the header?
    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The section `testssl.sh` reported this finding under
    pub section: TestSSLSection,

    /// The finding's id (not db id, but `testssl.sh` id)
    #[rorm(max_length = 255)]
    pub key: String,

    /// The finding's value (the value's semantics are highly dependant on the `key` and `testssl_severity`)
    #[rorm(max_length = 1024)]
    pub value: String,

    /// The severity reported by `testssl.sh` (this also includes log levels)
    pub testssl_severity: TestSSLSeverity,

    /// An associated cve
    #[rorm(max_length = 255)]
    pub cve: Option<String>,

    /// An associated cwe category
    #[rorm(max_length = 255)]
    pub cwe: Option<String>,
}

/// A [`TestSSLResultFinding`]'s section
#[derive(
    Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, DbEnum, Deserialize, Serialize, ToSchema,
)]
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

/// A [`TestSSLResultFinding`]'s severity
#[derive(Copy, Clone, Debug, DbEnum)]
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

/// Representation of a [OS Detection](AttackType::OSDetection) attack's result
#[derive(Model)]
pub struct OsDetectionResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// A host that was checked
    pub host: IpNetwork,

    /// The detected operating system or unknown if it wasn't able to precisely detect one.
    /// May contain additional human-readable information in `hints` or version information for known operating systems
    /// inside `version`.
    pub os: OsType,

    /// List of human-readable hints, separated by new-line characters (\n).
    #[rorm(max_length = 2048)]
    pub hints: String,

    /// Detected version for known operating systems. In case multiple possible were found, they
    /// will all be joined here using OR (`" OR "`) as separator.
    ///
    /// For linux this is the distro + distro version, if available.
    ///
    /// For windows this is the major release + additional version information, if available.
    #[rorm(max_length = 255)]
    pub version: String,
}
