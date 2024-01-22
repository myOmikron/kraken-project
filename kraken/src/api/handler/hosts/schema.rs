use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::aggregation_source::schema::SimpleAggregationSource;
use crate::api::handler::common::schema::{PageParams, SimpleTag};
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::SimpleService;
use crate::models::{HostCertainty, ManualHostCertainty, OsType};

/// The request to manually add a host
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct CreateHostRequest {
    /// The host's ip address
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip_addr: IpNetwork,

    /// Whether the host should exist right now or existed at some point
    pub certainty: ManualHostCertainty,
}

/// The request to update a host
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct UpdateHostRequest {
    /// The comment of a host
    pub comment: Option<String>,
    /// The global tags of a host
    pub global_tags: Option<Vec<Uuid>>,
    /// The workspace tags of a host
    pub workspace_tags: Option<Vec<Uuid>>,
}

/// Query parameters for filtering the hosts to get
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct GetAllHostsQuery {
    /// The parameters controlling the page to query
    #[serde(flatten)]
    pub page: PageParams,

    /// An optional general filter to apply
    pub global_filter: Option<String>,

    /// An optional host specific filter to apply
    pub host_filter: Option<String>,
}

/// The simple representation of a host
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleHost {
    /// The primary key of the host
    pub uuid: Uuid,
    /// The ip address of the host
    #[schema(example = "172.0.0.1")]
    pub ip_addr: String,
    /// The type of OS
    pub os_type: OsType,
    /// Response time in ms
    pub response_time: Option<i32>,
    /// A comment
    pub comment: String,
    /// The workspace this host is in
    pub workspace: Uuid,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
    /// The certainty of this host
    pub certainty: HostCertainty,
}

/// The full representation of a host
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullHost {
    /// The primary key of the host
    pub uuid: Uuid,
    /// The ip address of the host
    #[schema(example = "172.0.0.1")]
    pub ip_addr: String,
    /// The type of OS
    pub os_type: OsType,
    /// Response time in ms
    pub response_time: Option<i32>,
    /// A comment
    pub comment: String,
    /// The workspace this host is in
    pub workspace: Uuid,
    /// The list of tags this host has attached to
    pub tags: Vec<SimpleTag>,
    /// The number of attacks which found this host
    pub sources: SimpleAggregationSource,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
    /// The certainty of this host
    pub certainty: HostCertainty,
}

/// The path parameter of a host
#[derive(Serialize, Deserialize, IntoParams, Debug, Copy, Clone)]
pub struct PathHost {
    /// Workspace uuid
    pub w_uuid: Uuid,
    /// Host uuid
    pub h_uuid: Uuid,
}

/// A host's direct relations
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct HostRelations {
    /// This host's ports
    pub ports: Vec<SimplePort>,

    /// This host's services
    pub services: Vec<SimpleService>,

    /// Domains pointing to this host via a direct `A` or `AAAA` record
    pub direct_domains: Vec<SimpleDomain>,

    /// Domains pointing to this host via a `CNAME` record which eventually resolves to the host
    pub indirect_domains: Vec<SimpleDomain>,
}
