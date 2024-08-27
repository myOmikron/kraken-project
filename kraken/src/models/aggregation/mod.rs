use std::fmt;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use rorm::field;
use rorm::prelude::BackRef;
use rorm::prelude::ForeignModel;
use rorm::DbEnum;
use rorm::Model;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::GlobalTag;
use crate::models::Workspace;
use crate::models::WorkspaceTag;

mod convert;
#[cfg(feature = "bin")]
mod operations;

/// A representation of an OS type
#[derive(DbEnum, Copy, Clone, Debug, ToSchema, Serialize, Deserialize)]
pub enum OsType {
    /// The OS type is currently unknown
    Unknown,
    /// Linux based OS
    Linux,
    /// Windows based OS
    Windows,
    /// Apple based OS
    Apple,
    /// Android based OS
    Android,
    /// FreeBSD based OS
    FreeBSD,
}

/// The certainty of a host
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug, PartialOrd, PartialEq)]
pub enum HostCertainty {
    /// 3rd party historical data
    Historical = 0,
    /// 3rd party data
    SupposedTo = 1,
    /// The host has responded either by HostAlive, Port or Service Detection or something similar
    Verified = 2,
}

/// A representation of an host.
///
/// Will be collected from all results that yield IP addresses
#[derive(Model)]
pub struct Host {
    /// The primary key of a host
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The IP address of the host.
    ///
    /// If the host has multiple addresses, create a [Host] for each and link them.
    #[rorm(index)]
    pub ip_addr: IpNetwork,

    /// The type of OS of this host
    pub os_type: OsType,

    /// Response time in ms
    pub response_time: Option<i32>,

    /// The ports of a host
    pub ports: BackRef<field!(Port::F.host)>,

    /// The services of a host
    pub services: BackRef<field!(Service::F.host)>,

    /// The domains of a host
    pub domains: BackRef<field!(DomainHostRelation::F.host)>,

    /// The http services of a host
    pub http_services: BackRef<field!(HttpService::F.host)>,

    /// A comment to the host
    #[rorm(max_length = 1024)]
    pub comment: String,

    /// The certainty of this host
    pub certainty: HostCertainty,

    /// Workspace tags of the host
    pub workspace_tags: BackRef<field!(HostWorkspaceTag::F.host)>,

    /// Global tags of the host
    pub global_tags: BackRef<field!(HostGlobalTag::F.host)>,

    /// A reference to the workspace this host is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between [GlobalTag] and [Host]
#[derive(Model)]
pub struct HostGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The host this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub host: ForeignModel<Host>,
}

/// M2M relation between [WorkspaceTag] and [Host]
#[derive(Model)]
pub struct HostWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The host this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub host: ForeignModel<Host>,
}

/// The certainty a service is detected
#[derive(Debug, Copy, Clone, ToSchema, Deserialize, Serialize, DbEnum, PartialOrd, PartialEq)]
pub enum ServiceCertainty {
    /// 3rd party historical data
    Historical = 0,
    /// 3rd party data
    SupposedTo = 1,
    /// May be a certain service
    MaybeVerified = 2,
    /// Service is definitely correct
    DefinitelyVerified = 3,
    /// No specific service detected, generic fallback payload got a response though
    UnknownService = 4,
}

/// A detected service on a host
#[derive(Model)]
pub struct Service {
    /// Primary key of a service
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the service
    #[rorm(index, max_length = 255)]
    pub name: String,

    /// Optional version of the service
    #[rorm(index, max_length = 255)]
    pub version: Option<String>,

    /// The certainty the service is detected correct
    pub certainty: ServiceCertainty,

    /// The host this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub host: ForeignModel<Host>,

    /// The port this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub port: Option<ForeignModel<Port>>,

    /// The transport protocols the service responds to.
    ///
    /// By "transport protocols" we mean protocols layered above the [`PortProtocol`] which are not applications yet.
    /// For example: TLS
    ///
    /// This integer is a bitset whose interpretation depends on the `port`'s `protocol`.
    #[rorm(default = 0)] // = Unknown
    pub protocols: i16,

    /// A comment to the service
    #[rorm(max_length = 1024)]
    pub comment: String,

    /// Workspace tags of the service
    pub workspace_tags: BackRef<field!(ServiceWorkspaceTag::F.service)>,

    /// Global tags of the service
    pub global_tags: BackRef<field!(ServiceGlobalTag::F.service)>,

    /// A reference to the workspace this service is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between [GlobalTag] and [Service]
#[derive(Model)]
pub struct ServiceGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The service this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub service: ForeignModel<Service>,
}

/// M2M relation between [WorkspaceTag] and [Service]
#[derive(Model)]
pub struct ServiceWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The service this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub service: ForeignModel<Service>,
}

/// A protocol of a port
#[derive(DbEnum, ToSchema, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PortProtocol {
    /// Unknown protocol
    Unknown,
    /// tcp
    Tcp,
    /// udp
    Udp,
    /// sctp
    Sctp,
}

/// The certainty states of a port
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug, PartialOrd, PartialEq)]
pub enum PortCertainty {
    /// 3rd party historical data
    Historical = 0,
    /// 3rd party data
    SupposedTo = 1,
    /// The host has responded either by HostAlive, Port or Service Detection or something similar
    Verified = 2,
}

/// A port
#[derive(Model)]
pub struct Port {
    /// Primary key of a port
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Port number
    #[rorm(index)]
    pub port: i32,

    /// Port protocol
    pub protocol: PortProtocol,

    /// The certainty of this port
    pub certainty: PortCertainty,

    /// The host this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub host: ForeignModel<Host>,

    /// The services that link to this port
    pub services: BackRef<field!(Service::F.port)>,

    /// The http services of a port
    pub http_services: BackRef<field!(HttpService::F.port)>,

    /// A comment to the port
    #[rorm(max_length = 1024)]
    pub comment: String,

    /// Workspace tags of the port
    pub workspace_tags: BackRef<field!(PortWorkspaceTag::F.port)>,

    /// Global tags of the port
    pub global_tags: BackRef<field!(PortGlobalTag::F.port)>,

    /// A reference to the workspace this port is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between [GlobalTag] and [Port]
#[derive(Model)]
pub struct PortGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The port this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub port: ForeignModel<Port>,
}

/// M2M relation between [WorkspaceTag] and [Port]
#[derive(Model)]
pub struct PortWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The port this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub port: ForeignModel<Port>,
}

/// The certainty of a domain
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug, PartialOrd, PartialEq)]
pub enum DomainCertainty {
    /// The domain was not found through DNS
    Unverified = 0,
    /// Domain was verified through DNS
    Verified = 1,
}

/// A domain
#[derive(Model)]
pub struct Domain {
    /// The primary key of a domain
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The domain that was found
    #[rorm(index, max_length = 255)]
    pub domain: String,

    /// The certainty of this domain entry
    pub certainty: DomainCertainty,

    /// A comment to the domain
    #[rorm(max_length = 1024)]
    pub comment: String,

    /// Domains resolving to this host
    pub hosts: BackRef<field!(DomainHostRelation::F.domain)>,

    /// Domains pointing to this one
    pub sources: BackRef<field!(DomainDomainRelation::F.destination)>,

    /// Domains, this one resolves to
    pub destinations: BackRef<field!(DomainDomainRelation::F.source)>,

    /// The http services of a domain
    pub http_services: BackRef<field!(HttpService::F.domain)>,

    /// Workspace tags of the domain
    pub workspace_tags: BackRef<field!(DomainWorkspaceTag::F.domain)>,

    /// Global tags of the domain
    pub global_tags: BackRef<field!(DomainGlobalTag::F.domain)>,

    /// A reference to the workspace this domain is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between two [domains](Domain)
#[derive(Model)]
pub struct DomainDomainRelation {
    /// The primary key of this relation
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The source address
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub source: ForeignModel<Domain>,

    /// The destination address
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub destination: ForeignModel<Domain>,

    /// A reference to the workspace for faster querying
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

/// M2M relation between a [Domain] and a [Host]
#[derive(Model)]
pub struct DomainHostRelation {
    /// The primary key of this relation
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The source domain
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub domain: ForeignModel<Domain>,

    /// The destination host
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub host: ForeignModel<Host>,

    /// Does this relation exist directly as a dns record or is it the result of a chain of `CNAME`s?
    ///
    /// If this flag is set to `true`, the domain directly points to the host via an `A` or `AAAA` record.
    /// If it is `false`, the domain redirects to another via `CNAME` which eventually resolves to the host.
    pub is_direct: bool,

    /// A reference to the workspace for faster querying
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

/* This enum won't be actually used, but stays for now as reminder and collection of which relations will need implementations

/// The type of a relation
#[derive(DbEnum)]
pub enum RelationType {
    /// Relation to an IPv4 address
    A,
    /// Relation to an IPv6 address
    AAAA,
    /// Relation to another domain
    CNAME,
    /// Relation from an SPF record
    SPF,
    /// Relation from an SRV record
    SRV,
    /// Relation from an TXT record
    TXT,
    /// Relation from an NS record
    NS,
    /// Relation from an SOA record
    SOA,
    /// Relation from an MX record
    MX,
    /// Relation from an PTR record
    PTR,
    /// Relation from an TLSA record
    TLSA,
    /// Relation from an CAA record
    CAA,
    /// Relation from an DMARC record
    DMARC,
}
*/

/// M2M relation between [GlobalTag] and [Domain]
#[derive(Model)]
pub struct DomainGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The domain this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub domain: ForeignModel<Domain>,
}

/// M2M relation between [WorkspaceTag] and [Domain]
#[derive(Model)]
pub struct DomainWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The domain this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub domain: ForeignModel<Domain>,
}

/// An HTTP Service
///
/// This aggregation is intended to hold information regarding
/// an HTTP service (e.g. nginx or wordpress)
// Unique over name + base_path + host + port + domain + tls + sni_required
#[derive(Model)]
pub struct HttpService {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The name of the http service
    #[rorm(max_length = 255)]
    pub name: String,

    /// Optional version of the http service
    #[rorm(index, max_length = 255)]
    pub version: Option<String>,

    /// The base path of the http service
    #[rorm(max_length = 1024)]
    pub base_path: String,

    /// Marks whether the http service is accessible over TLS
    /// If it is accessible via raw TCP and TLS, two http services
    /// should be created
    pub tls: bool,

    /// Marks whether SNI is required to
    pub sni_required: bool,

    /// An optional domain that is used to access the http service
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub domain: Option<ForeignModel<Domain>>,

    /// The host this http service is running on
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub host: ForeignModel<Host>,

    /// The port this http service is running on
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub port: ForeignModel<Port>,

    /// The option to add some notes to the http service
    #[rorm(max_length = 1024)]
    pub comment: String,

    /// The certainty of this http service
    pub certainty: HttpServiceCertainty,

    /// Workspace tags of the http service
    pub workspace_tags: BackRef<field!(HttpServiceWorkspaceTag::F.http_service)>,

    /// Global tags of the http service
    pub global_tags: BackRef<field!(HttpServiceGlobalTag::F.http_service)>,

    /// A reference to the workspace this http service is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The certainty of a http service
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug, PartialOrd, PartialEq)]
pub enum HttpServiceCertainty {
    /// 3rd party historical data
    Historical = 0,
    /// 3rd party data
    SupposedTo = 1,
    /// One of our attacks verified this service
    Verified = 2,
}

/// M2M relation between [GlobalTag] and [HttpService]
#[derive(Model)]
pub struct HttpServiceGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The HttpService this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub http_service: ForeignModel<HttpService>,
}

/// M2M relation between [WorkspaceTag] and [HttpService]
#[derive(Model)]
pub struct HttpServiceWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The http service this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub http_service: ForeignModel<HttpService>,
}

/// Generic M2M relation between aggregated models (ex: [`Host`])
/// and the sources which contributed to them (ex: [`HostAliveResult`])
#[derive(Model)]
pub struct AggregationSource {
    /// Primary key of this table
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Workspace, the involved parties belong to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The type of the source which contributed to the aggregated model
    pub source_type: SourceType,

    /// The primary key of the source which contributed to the aggregated model
    ///
    /// The table this key is valid in, depends on the `source_type`
    pub source_uuid: Uuid,

    /// The aggregated model's type
    pub aggregated_table: AggregationTable,

    /// The aggregated model's primary key
    ///
    /// The table this key is valid in, depends on the `aggregated_table`
    pub aggregated_uuid: Uuid,
}

/// Enum used in [`AggregationSource`] to identify which table it points to
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug, Eq, PartialEq, Hash)]
pub enum SourceType {
    /// The [`BruteforceSubdomainsResult`] table
    BruteforceSubdomains,

    /// Effectively deleted, but postgres can't delete enum variants
    #[serde(skip)]
    TcpPortScan,

    /// The [`CertificateTransparencyResult`] table
    QueryCertificateTransparency,
    /// The [`DehashedQueryResult`] table
    QueryDehashed,
    /// The [`HostAliveResult`] table
    HostAlive,
    /// The [`ServiceDetectionResult`] table
    ServiceDetection,
    /// The [`UdpServiceDetectionResult`] table
    UdpServiceDetection,
    /// The [`DnsResolutionResult`] table
    DnsResolution,
    /// The [`DnsTxtScanResult`] table
    DnsTxtScan,
    /// The table for the not yet implemented [`AttackType::UdpPortScan`] results
    UdpPortScan,
    /// The table for the not yet implemented [`AttackType::ForcedBrowsing`] results
    ForcedBrowsing,
    /// The [`OsDetectionResult`] table
    OSDetection,
    /// The table for the not yet implemented [`AttackType::VersionDetection`] results
    VersionDetection,
    /// The table for the not yet implemented [`AttackType::AntiPortScanningDetection`] results
    AntiPortScanningDetection,
    /// The [`TestSSLResultHeader`] table
    TestSSL,
    /// The [`ManualDomain`] table
    ManualDomain,
    /// The [`ManualHost`] table
    ManualHost,
    /// The [`ManualPort`] table
    ManualPort,
    /// The [`ManualService`] table
    ManualService,
    /// The [`ManualHttpService`] table
    ManualHttpService,
}

/// Enum used in [`AggregationSource`] to identify which table it points to
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum AggregationTable {
    /// The [`Host`] table
    Host,
    /// The [`Port`] table
    Port,
    /// The [`Service`] table
    Service,
    /// The [`Domain`] table
    Domain,
    /// The [`HttpService`] table
    HttpService,
}

impl fmt::Display for AggregationTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let table = match self {
            AggregationTable::Host => Host::TABLE,
            AggregationTable::Port => Port::TABLE,
            AggregationTable::Service => Service::TABLE,
            AggregationTable::Domain => Domain::TABLE,
            AggregationTable::HttpService => HttpService::TABLE,
        };
        write!(f, "{table}")
    }
}
