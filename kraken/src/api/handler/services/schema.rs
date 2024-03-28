use std::net::IpAddr;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::api::handler::aggregation_source::schema::SimpleAggregationSource;
use crate::api::handler::common::schema::PageParams;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::SimplePort;
use crate::models::FindingSeverity;
use crate::models::ManualServiceCertainty;
use crate::models::ServiceCertainty;
use crate::models::ServiceProtocols;

/// The request to manually add a service
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct CreateServiceRequest {
    /// The service's name
    // TODO #[schema(example = "django")]
    pub name: String,

    /// Whether the port should exist right now or existed at some point
    pub certainty: ManualServiceCertainty,

    /// The ip address the service runs on
    #[schemars(with = "IpAddr")] // TODO
    pub host: IpNetwork,

    /// An optional port the service runs on
    ///
    /// If set, you must specify protocol
    // TODO #[schema(example = "8080")]
    pub port: Option<u16>,

    /// The port's protocol as well as its sub protocols
    pub protocols: Option<ServiceProtocols>,
}

/// The request to update a service
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct UpdateServiceRequest {
    /// The comment of the service
    pub comment: Option<String>,
    /// The global tags that are attached to the service
    pub global_tags: Option<Vec<Uuid>>,
    /// The workspace tags that are attached to the service
    pub workspace_tags: Option<Vec<Uuid>>,
}

/// Query parameters for filtering the services to get
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct GetAllServicesQuery {
    /// The parameters controlling the page to query
    #[serde(flatten)]
    pub page: PageParams,

    /// Only get services associated with a specific host
    pub host: Option<Uuid>,

    /// An optional general filter to apply
    pub global_filter: Option<String>,

    /// An optional service specific filter to apply
    pub service_filter: Option<String>,
}

/// A simple representation of a service
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct SimpleService {
    /// The uuid of the service
    pub uuid: Uuid,
    /// The name of the service
    // TODO #[schema(example = "postgresql")]
    pub name: String,
    /// The version of the service
    // TODO #[schema(example = "13.0.1")]
    pub version: Option<String>,
    /// The certainty the service is detected correct
    pub certainty: ServiceCertainty,
    /// The host this service is linked to
    pub host: Uuid,
    /// The port this service may linked to
    pub port: Option<Uuid>,
    /// The comment attached to the service
    // TODO #[schema(example = "Holds all relevant information")]
    pub comment: String,
    /// The workspace is service is linked to
    pub workspace: Uuid,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// A full representation of a service
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct FullService {
    /// Uuid of the service
    pub uuid: Uuid,
    /// The service's name
    // TODO #[schema(example = "postgresql")]
    pub name: String,
    /// An optional version of the running service
    // TODO #[schema(example = "13.0.1")]
    pub version: Option<String>,
    /// The certainty of the detection
    pub certainty: ServiceCertainty,
    /// The host this service is assigned to
    pub host: SimpleHost,
    /// An optional port this service listens on
    pub port: Option<SimplePort>,
    /// The protocols used above the port's protocol
    pub protocols: Option<ServiceProtocols>,
    /// A comment to the service
    // TODO #[schema(example = "Holds all relevant information")]
    pub comment: String,
    /// The workspace this service is linked to
    pub workspace: Uuid,
    /// The tags this service is linked to
    pub tags: Vec<SimpleTag>,
    /// The number of attacks which found this host
    pub sources: SimpleAggregationSource,
    /// The severest finding's severity associated with this host
    pub severity: Option<FindingSeverity>,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// The path parameter of a service
#[derive(Deserialize, Serialize, JsonSchema, Debug, Copy, Clone)]
pub struct PathService {
    /// The workspace's uuid
    pub w_uuid: Uuid,
    /// The service's uuid
    pub s_uuid: Uuid,
}

/// A service's direct relations
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct ServiceRelations {
    /// The port a service listens on
    pub port: Option<SimplePort>,

    /// The host a service runs on
    pub host: SimpleHost,
}
