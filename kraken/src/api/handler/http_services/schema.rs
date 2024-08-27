use std::net::IpAddr;
use std::num::NonZeroU16;

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
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::findings::schema::FindingSeverity;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::PortProtocol;
use crate::api::handler::ports::schema::SimplePort;

/// The request to manually add a http service
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct CreateHttpServiceRequest {
    /// The service's name
    pub name: String,

    /// Optional version of the http service
    pub version: Option<String>,

    /// The service's domain
    pub domain: Option<String>,

    /// The service's ip address
    #[schema(value_type = String)]
    pub ip_addr: IpAddr,

    /// The service's port
    #[schema(value_type = u16)]
    pub port: NonZeroU16,

    /// The service's port's protocol
    pub port_protocol: PortProtocol,

    /// The certainty of this http service
    pub certainty: ManualHttpServiceCertainty,

    /// The base path the service is routed on
    ///
    /// (Should default to "/")
    pub base_path: String,

    /// Is this a https service?
    pub tls: bool,

    /// Does this service require sni?
    pub sni_require: bool,
}

/// The request to update a http service
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct UpdateHttpServiceRequest {
    /// The comment of a host
    pub comment: Option<String>,

    /// The global tags of a host
    pub global_tags: Option<Vec<Uuid>>,

    /// The workspace tags of a host
    pub workspace_tags: Option<Vec<Uuid>>,
}

/// Query parameters for filtering the http services to get
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct GetAllHttpServicesQuery {
    /// The parameters controlling the page to query
    #[serde(flatten)]
    pub page: PageParams,

    /// Only get http services associated with a specific host
    pub host: Option<Uuid>,

    /// An optional general filter to apply
    pub global_filter: Option<String>,

    /// An optional http service specific filter to apply
    pub http_service_filter: Option<String>,
}

/// The simple representation of a http service
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleHttpService {
    /// The primary key of the http service
    pub uuid: Uuid,

    /// The http service's name
    pub name: String,

    /// Optional version of the http service
    pub version: Option<String>,

    /// The http service's domain
    pub domain: Option<Uuid>,

    /// The http service's ip address
    pub host: Uuid,

    /// The http service's port
    pub port: Uuid,

    /// The base path the http service is routed on
    pub base_path: String,

    /// Is this a https service?
    pub tls: bool,

    /// Does this http service require sni?
    pub sni_required: bool,

    /// A comment
    pub comment: String,

    /// The certainty of this http service
    pub certainty: HttpServiceCertainty,

    /// The workspace this http service is in
    pub workspace: Uuid,

    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
}

/// The full representation of a http service
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullHttpService {
    /// The primary key of the http service
    pub uuid: Uuid,

    /// The http service's name
    pub name: String,

    /// Optional version of the http service
    pub version: Option<String>,

    /// The http service's domain
    pub domain: Option<SimpleDomain>,

    /// The http service's ip address
    pub host: SimpleHost,

    /// The http service's port
    pub port: SimplePort,

    /// The base path the http service is routed on
    pub base_path: String,

    /// Is this a https service?
    pub tls: bool,

    /// Does this http service require sni?
    pub sni_required: bool,

    /// A comment
    pub comment: String,

    /// The certainty of this http service
    pub certainty: HttpServiceCertainty,

    /// The workspace this http service is linked to
    pub workspace: Uuid,

    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,

    /// The list of tags this http service has attached to
    pub tags: Vec<SimpleTag>,

    /// The number of attacks which found this http service
    pub sources: SimpleAggregationSource,

    /// The severest finding's severity associated with this http service
    pub severity: Option<FindingSeverity>,
}

/// The path parameter of a http service
#[derive(Serialize, Deserialize, IntoParams, Debug, Copy, Clone)]
pub struct PathHttpService {
    /// Workspace uuid
    pub w_uuid: Uuid,

    /// Http service uuid
    pub hs_uuid: Uuid,
}

/// A http service's direct relations
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct HttpServiceRelations {
    /// The host a service runs on
    pub host: SimpleHost,

    /// The port a service runs on
    pub port: SimplePort,

    /// The domain a service runs on
    pub domain: Option<SimpleDomain>,
}

/// The certainty of a http service
#[derive(Copy, Clone, Deserialize, Serialize, ToSchema, Debug, PartialOrd, PartialEq)]
pub enum HttpServiceCertainty {
    /// 3rd party historical data
    Historical = 0,
    /// 3rd party data
    SupposedTo = 1,
    /// One of our attacks verified this service
    Verified = 2,
}

/// The certainty of a manually added http service
#[derive(Copy, Clone, Deserialize, Serialize, ToSchema, Debug, PartialOrd, PartialEq)]
pub enum ManualHttpServiceCertainty {
    /// Historical data
    Historical,
    /// Up to date data
    SupposedTo,
}
