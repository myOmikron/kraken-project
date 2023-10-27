use std::collections::HashMap;

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use uuid::Uuid;

/// The aggregated results of a workspace
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedWorkspace {
    /// The hosts found by this workspace
    pub hosts: HashMap<Uuid, AggregatedHost>,

    /// The ports found by this workspace
    pub ports: HashMap<Uuid, AggregatedPort>,

    /// The services found by this workspace
    pub services: HashMap<Uuid, AggregatedService>,

    /// The domains found by this workspace
    pub domains: HashMap<Uuid, AggregatedDomain>,

    /// All m2m relations which are not inlined
    pub relations: HashMap<Uuid, AggregatedRelation>,
}

/// A representation of an host.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedHost {
    /// The host's uuid
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
    pub ports: Vec<Uuid>,

    /// The services of a host
    pub services: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainHost`]
    pub domains: Vec<Uuid>,

    /// A comment to the host
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this host was encountered
    pub created_at: DateTime<Utc>,
}

/// An open port on a host
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedPort {
    /// The port's uuid
    pub uuid: Uuid,

    /// Port number
    pub port: u16,

    /// Port protocol
    pub protocol: PortProtocol,

    /// The host this service is attached to
    pub host: Uuid,

    /// The services that link to this port
    pub services: Vec<Uuid>,

    /// A comment to the port
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this port was encountered
    pub created_at: DateTime<Utc>,
}

/// A detected service on a host
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedService {
    /// The service's uuid
    pub uuid: Uuid,

    /// Name of the service
    pub name: String,

    /// Optional version of the service
    pub version: Option<String>,

    /// The host this service is attached to
    pub host: Uuid,

    /// The port this service is attached to
    pub port: Option<Uuid>,

    /// A comment to the service
    pub comment: String,

    /// The certainty the service was detected
    pub certainty: Certainty,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this service was encountered
    pub created_at: DateTime<Utc>,
}

/// A domain
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedDomain {
    /// The domain's uuid
    pub uuid: Uuid,

    /// The domain that was found
    pub domain: String,

    /// Uuids to [`AggregatedRelation::DomainHost`]
    pub hosts: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainDomain`] where this domain is the `destination`
    pub sources: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainDomain`] where this domain is the `source`
    pub destinations: Vec<Uuid>,

    /// A comment to the domain
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this domain was encountered
    pub created_at: DateTime<Utc>,
}

/// Set of global and local tags
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedTags {
    /// Global tags
    pub global_tags: Vec<String>,

    /// Tags which are local to the workspace
    pub local_tags: Vec<String>,
}

/// An m2m relation
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum AggregatedRelation {
    /// A DNS relation between two domains
    DomainDomain {
        /// The source domain pointing to the other domain
        source: Uuid,

        /// The destination domain which is pointed to by the other domain
        destination: Uuid,
    },
    /// A DNS relation between a domain and a host
    DomainHost {
        /// The domain resolving to a host
        domain: Uuid,

        /// The host resolved to by a domain
        host: Uuid,

        /// Does this relation exist directly as a dns record or is it the result of a chain of `CNAME`s?
        ///
        /// If this flag is set to `true`, the domain directly points to the host via an `A` or `AAAA` record.
        /// If it is `false`, the domain redirects to another via `CNAME` which eventually resolves to the host.
        is_direct: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
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

/// A protocol of a port
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
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

/// The certainty a service is detected
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Certainty {
    /// Service is not detected
    Unknown,
    /// May be a certain service
    Maybe,
    /// Service is definitely correct
    Definitely,
}
