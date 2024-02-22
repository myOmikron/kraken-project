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
use crate::api::handler::hosts::schema::SimpleHost;
use crate::models::DomainCertainty;

/// The request to manually add a domain
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct CreateDomainRequest {
    /// The domain to add
    #[schema(example = "kraken.test")]
    pub domain: String,
}

/// The request to update a domain
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct UpdateDomainRequest {
    /// The comment of the domain
    pub comment: Option<String>,
    /// Global tags that are linked to the domain
    pub global_tags: Option<Vec<Uuid>>,
    /// Workspace tags that are linked to the domain
    pub workspace_tags: Option<Vec<Uuid>>,
}

/// Query parameters for filtering the domains to get
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct GetAllDomainsQuery {
    /// The parameters controlling the page to query
    #[serde(flatten)]
    pub page: PageParams,
    /// Only get domains pointing to a specific host
    ///
    /// This includes domains which point to another domain which points to this host.
    pub host: Option<Uuid>,
    /// An optional general filter to apply
    pub global_filter: Option<String>,
    /// An optional domain specific filter to apply
    pub domain_filter: Option<String>,
}

/// A simple representation of a domain in a workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleDomain {
    /// The uuid of the domain
    pub uuid: Uuid,
    /// The domain name
    #[schema(example = "example.com")]
    pub domain: String,
    /// The comment to the domain
    #[schema(example = "This is a important domain!")]
    pub comment: String,
    /// The workspace this domain is linked to
    pub workspace: Uuid,
    /// The point in time this domain was created
    pub created_at: DateTime<Utc>,
    /// The certainty of this domain entry
    pub certainty: DomainCertainty,
}

/// A full representation of a domain in a workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullDomain {
    /// The primary key of the domain
    pub uuid: Uuid,
    /// The domain's name
    #[schema(example = "example.com")]
    pub domain: String,
    /// A comment
    #[schema(example = "This is a important domain!")]
    pub comment: String,
    /// The workspace this domain is in
    pub workspace: Uuid,
    /// The list of tags this domain has attached to
    pub tags: Vec<SimpleTag>,
    /// The number of attacks which found this domain
    pub sources: SimpleAggregationSource,
    /// The point in time, the record was created
    pub created_at: DateTime<Utc>,
    /// The certainty of this domain entry
    pub certainty: DomainCertainty,
}

/// The path parameter of a domain
#[derive(Serialize, Deserialize, IntoParams, Debug, Clone, Copy)]
pub struct PathDomain {
    /// The workspace's uuid
    pub w_uuid: Uuid,
    /// The domain's uuid
    pub d_uuid: Uuid,
}

/// A domain's direct relations
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct DomainRelations {
    /// All domains which contain a `CNAME` record with this domain
    pub source_domains: Vec<SimpleDomain>,

    /// All domains this domain has `CNAME` records to
    pub target_domains: Vec<SimpleDomain>,

    /// All hosts this domain has an `A` or `AAAA` record for
    pub direct_hosts: Vec<SimpleHost>,

    /// All hosts any `target_domains` resolves to
    pub indirect_hosts: Vec<SimpleHost>,
}
