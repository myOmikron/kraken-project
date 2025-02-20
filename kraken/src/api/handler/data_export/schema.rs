use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::domains::schema::DomainCertainty;
use crate::api::handler::findings::schema::FindingSeverity;
use crate::api::handler::hosts::schema::HostCertainty;
use crate::api::handler::hosts::schema::OsType;
use crate::api::handler::http_services::schema::HttpServiceCertainty;
use crate::api::handler::ports::schema::PortCertainty;
use crate::api::handler::ports::schema::PortProtocol;
use crate::api::handler::services::schema::ServiceCertainty;
use crate::api::handler::services::schema::ServiceProtocols;
use crate::chan::ws_manager::schema::AggregationType;

/// The aggregated results of a workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AggregatedWorkspace {
    /// The hosts found by this workspace
    pub hosts: HashMap<Uuid, AggregatedHost>,

    /// The ports found by this workspace
    pub ports: HashMap<Uuid, AggregatedPort>,

    /// The services found by this workspace
    pub services: HashMap<Uuid, AggregatedService>,

    /// The domains found by this workspace
    pub domains: HashMap<Uuid, AggregatedDomain>,

    /// The http services found by this workspace
    pub http_services: HashMap<Uuid, AggregatedHttpService>,

    /// All m2m relations which are not inlined
    pub relations: HashMap<Uuid, AggregatedRelation>,

    /// The findings found by this workspace
    pub findings: HashMap<Uuid, AggregatedFinding>,
}

/// A representation of an host.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AggregatedHost {
    /// The host's uuid
    pub uuid: Uuid,

    /// The IP address of the host.
    ///
    /// If the host has multiple addresses, create a [Host] for each and link them.
    #[schema(value_type = String)]
    pub ip_addr: IpNetwork,

    /// The type of OS of this host
    pub os_type: OsType,

    /// The certainty of the host
    pub certainty: HostCertainty,

    /// Response time in ms
    pub response_time: Option<i32>,

    /// The ports of a host
    pub ports: Vec<Uuid>,

    /// The services of a host
    pub services: Vec<Uuid>,

    /// The http services of a host
    pub http_services: Vec<Uuid>,

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
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
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

    /// The http services of a port
    pub http_services: Vec<Uuid>,

    /// The certainty of the port
    pub certainty: PortCertainty,

    /// A comment to the port
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this port was encountered
    pub created_at: DateTime<Utc>,
}

/// A detected service on a host
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
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

    /// The protocols used above the port's protocol
    pub protocols: Option<ServiceProtocols>,

    /// A comment to the service
    pub comment: String,

    /// The certainty the service was detected
    pub certainty: ServiceCertainty,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this service was encountered
    pub created_at: DateTime<Utc>,
}

/// A domain
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AggregatedDomain {
    /// The domain's uuid
    pub uuid: Uuid,

    /// The domain that was found
    pub domain: String,

    /// The http services of a domain
    pub http_services: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainHost`]
    pub hosts: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainDomain`] where this domain is the `destination`
    pub sources: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainDomain`] where this domain is the `source`
    pub destinations: Vec<Uuid>,

    /// The certainty of the domain
    pub certainty: DomainCertainty,

    /// A comment to the domain
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this domain was encountered
    pub created_at: DateTime<Utc>,
}

/// A detected http service on a host
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AggregatedHttpService {
    /// The http service's uuid
    pub uuid: Uuid,

    /// Name of the http service
    pub name: String,

    /// Optional version of the http service
    pub version: Option<String>,

    /// The domain this http service is attached to
    pub domain: Option<Uuid>,

    /// The host this http service is attached to
    pub host: Uuid,

    /// The port this http service is attached to
    pub port: Uuid,

    /// The base path the http service is routed on
    pub base_path: String,

    /// Is this a https service?
    pub tls: bool,

    /// Does this http service require sni?
    pub sni_required: bool,

    /// A comment to the service
    pub comment: String,

    /// The certainty of this http service
    pub certainty: HttpServiceCertainty,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this service was encountered
    pub created_at: DateTime<Utc>,
}

/// A finding
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AggregatedFinding {
    /// The uuid of the finding
    pub uuid: Uuid,

    /// The finding's name (taken from its definition)
    pub name: String,

    /// The finding's cve (taken from its definition)
    pub cve: Option<String>,

    /// The finding's severity
    pub severity: FindingSeverity,

    /// Expected time duration required for the remediation
    pub remediation_duration: String,

    /// A weight without semantic used to sort findings
    pub sorting_weight: i32,

    /// The details of this finding
    pub details: String,

    /// List of all affected objects
    pub affected: HashMap<Uuid, AggregatedFindingAffected>,

    /// The point in time this finding was created
    pub created_at: DateTime<Utc>,

    /// The list of categories this finding falls into
    pub categories: Vec<String>,
}

/// A finding affected
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AggregatedFindingAffected {
    /// The uuid of the finding affected
    pub uuid: Uuid,

    /// The affected's type
    ///
    /// Determines how the uuid is to be used
    pub r#type: AggregationType,

    /// The details of this affected
    pub details: String,
}

/// Set of global and local tags
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Default)]
pub struct AggregatedTags {
    /// Global tags
    pub global_tags: Vec<String>,

    /// Tags which are local to the workspace
    pub local_tags: Vec<String>,
}

/// An m2m relation
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
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
