use std::net::IpAddr;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::aggregation_source::schema::SimpleAggregationSource;
use crate::api::handler::common::schema::PageParams;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::findings::schema::FindingSeverity;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::http_services::schema::SimpleHttpService;
use crate::api::handler::services::schema::SimpleService;

/// The request to manually add a port
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct CreatePortRequest {
    /// The ip address the port is open on
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip_addr: IpAddr,

    /// The port to add
    #[schema(example = "8080")]
    pub port: u16,

    /// Whether the port should exist right now or existed at some point
    pub certainty: ManualPortCertainty,

    /// The port's protocol
    #[schema(example = "Tcp")]
    pub protocol: PortProtocol,
}

/// The request to update a port
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct UpdatePortRequest {
    /// The comment of the port
    pub comment: Option<String>,
    /// Global tags that are linked to the port
    pub global_tags: Option<Vec<Uuid>>,
    /// Workspace tags that are linked to the port
    pub workspace_tags: Option<Vec<Uuid>>,
}

/// Query parameters for filtering the ports to get
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct GetAllPortsQuery {
    /// The parameters controlling the page to query
    #[serde(flatten)]
    pub page: PageParams,

    /// Only get ports associated with a specific host
    pub host: Option<Uuid>,

    /// An optional general filter to apply
    pub global_filter: Option<String>,

    /// An optional port specific filter to apply
    pub port_filter: Option<String>,
}

/// The simple representation of a port
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimplePort {
    /// Uuid of the port
    pub uuid: Uuid,
    /// Port number
    #[schema(example = 1337)]
    pub port: u16,
    /// Port protocol
    pub protocol: PortProtocol,
    /// The certainty of this port
    pub certainty: PortCertainty,
    /// The host this port is assigned to
    pub host: Uuid,
    /// A comment to the port
    pub comment: String,
    /// The workspace this port is linked to
    pub workspace: Uuid,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// The full representation of a port
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullPort {
    /// Uuid of the port
    pub uuid: Uuid,
    /// Port number
    #[schema(example = 1337)]
    pub port: u16,
    /// Port protocol
    pub protocol: PortProtocol,
    /// The certainty of this port
    pub certainty: PortCertainty,
    /// The host this port is assigned to
    pub host: SimpleHost,
    /// A comment to the port
    pub comment: String,
    /// The tags this port is linked to
    pub tags: Vec<SimpleTag>,
    /// The workspace this port is linked to
    pub workspace: Uuid,
    /// The number of attacks which found this host
    pub sources: SimpleAggregationSource,
    /// The severest finding's severity associated with this host
    pub severity: Option<FindingSeverity>,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// The path parameter of a port
#[derive(Serialize, Deserialize, IntoParams, Debug, Copy, Clone)]
pub struct PathPort {
    /// The workspace's uuid
    pub w_uuid: Uuid,
    /// The port's uuid
    pub p_uuid: Uuid,
}

/// A port's direct relations
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct PortRelations {
    /// The host this port is assigned to
    pub host: SimpleHost,

    /// Services listening on this port
    pub services: Vec<SimpleService>,

    /// Http services listening on this port
    pub http_services: Vec<SimpleHttpService>,
}

/// The certainty states of a port
#[derive(Copy, Clone, Deserialize, Serialize, ToSchema, Debug, PartialOrd, PartialEq)]
pub enum PortCertainty {
    /// 3rd party historical data
    Historical = 0,
    /// 3rd party data
    SupposedTo = 1,
    /// The host has responded either by HostAlive, Port or Service Detection or something similar
    Verified = 2,
}

/// The certainty of a manually added port
#[derive(Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum ManualPortCertainty {
    /// Historical data
    Historical,
    /// Up to date data
    SupposedTo,
}

/// A protocol of a port
#[derive(ToSchema, Debug, Copy, Clone, Serialize, Deserialize)]
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
