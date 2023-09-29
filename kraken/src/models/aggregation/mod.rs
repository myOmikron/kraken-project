use ipnetwork::IpNetwork;
use rorm::prelude::{BackRef, ForeignModel};
use rorm::{field, DbEnum, Model, Patch};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{GlobalTag, Workspace, WorkspaceTag};

/// A representation of an OS type
#[derive(DbEnum, Copy, Clone, Debug, ToSchema, Serialize)]
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
    pub ip_addr: IpNetwork,

    /// The type of OS of this host
    pub os_type: OsType,

    /// Response time in ms
    pub response_time: Option<i32>,

    /// The ports of a host
    pub ports: BackRef<field!(Port::F.host)>,

    /// The services of a host
    pub services: BackRef<field!(Service::F.host)>,

    /// A comment to the host
    #[rorm(max_length = 255)]
    pub comment: String,

    /// A reference to the workspace this host is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Host")]
pub(crate) struct HostInsert {
    pub(crate) uuid: Uuid,
    pub(crate) ip_addr: IpNetwork,
    pub(crate) os_type: OsType,
    pub(crate) response_time: Option<i32>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
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

/// A detected service on a host
#[derive(Model)]
pub struct Service {
    /// Primary key of a service
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the service
    #[rorm(max_length = 255)]
    pub name: String,

    /// Optional version of the service
    #[rorm(max_length = 255)]
    pub version: Option<String>,

    /// The host this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub host: ForeignModel<Host>,

    /// The port this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub port: Option<ForeignModel<Port>>,

    /// A comment to the service
    #[rorm(max_length = 255)]
    pub comment: String,

    /// A reference to the workspace this service is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Service")]
pub(crate) struct ServiceInsert {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) host: ForeignModel<Host>,
    pub(crate) port: Option<ForeignModel<Port>>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
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
#[derive(DbEnum, ToSchema, Debug, Copy, Clone, Serialize)]
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

/// A port
#[derive(Model)]
pub struct Port {
    /// Primary key of a port
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Port number
    ///
    /// Reinterpret as u16 with to_ne_bytes and from_ne_bytes
    pub port: i16,

    /// Port protocol
    pub protocol: PortProtocol,

    /// The host this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub host: ForeignModel<Host>,

    /// The services that link to this port
    pub services: BackRef<field!(Service::F.port)>,

    /// A comment to the port
    #[rorm(max_length = 255)]
    pub comment: String,

    /// A reference to the workspace this port is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Port")]
pub(crate) struct PortInsert {
    pub(crate) uuid: Uuid,
    pub(crate) port: i16,
    pub(crate) protocol: PortProtocol,
    pub(crate) host: ForeignModel<Host>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
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

/// A domain
#[derive(Model)]
pub struct Domain {
    /// The primary key of a domain
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The domain that was found
    #[rorm(max_length = 255)]
    pub domain: String,

    /// A comment to the domain
    #[rorm(max_length = 255)]
    pub comment: String,

    /// A reference to the workspace this domain is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Domain")]
pub(crate) struct DomainInsert {
    pub(crate) uuid: Uuid,
    pub(crate) domain: String,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}

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
