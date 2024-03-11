//! The schemas of workspace related are defined in this module

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::attack_results::schema::FullQueryCertificateTransparencyResult;
use crate::api::handler::attack_results::schema::FullServiceDetectionResult;
use crate::api::handler::attack_results::schema::FullUdpServiceDetectionResult;
use crate::api::handler::attack_results::schema::SimpleDnsResolutionResult;
use crate::api::handler::attack_results::schema::SimpleDnsTxtScanResult;
use crate::api::handler::attack_results::schema::SimpleHostAliveResult;
use crate::api::handler::attack_results::schema::SimpleQueryUnhashedResult;
use crate::api::handler::attacks::schema::SimpleAttack;
use crate::api::handler::common::de_optional;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::SimpleService;
use crate::api::handler::users::schema::SimpleUser;

/// The request to create a new workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct CreateWorkspaceRequest {
    /// The name of the workspace
    #[schema(example = "secure-workspace")]
    pub name: String,
    /// The description of the workspace
    #[schema(example = "This workspace is super secure and should not be looked at!!")]
    pub description: Option<String>,
}

/// Request to search the workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SearchWorkspaceRequest {
    /// the term to search for
    pub search_term: String,
}

/// The request type to update a workspace
///
/// All parameter are optional, but at least one of them must be specified
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct UpdateWorkspaceRequest {
    /// Name of the workspace
    #[schema(example = "Workspace for work")]
    pub name: Option<String>,
    /// Description of the workspace
    #[schema(example = "This workspace is for work and for work only!")]
    #[serde(deserialize_with = "de_optional")]
    pub description: Option<Option<String>>,
}

/// The request to transfer a workspace to another account
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy)]
pub struct TransferWorkspaceRequest {
    /// The uuid of the user that should receive the workspace
    pub user: Uuid,
}

/// The request to invite a user to the workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy)]
pub struct InviteToWorkspaceRequest {
    /// The user to invite
    pub user: Uuid,
}

/// The url components of an invitation
#[derive(Serialize, Deserialize, IntoParams, Debug, Clone, Copy)]
pub struct InviteUuid {
    /// The UUID of the workspace
    pub w_uuid: Uuid,
    /// The UUID of the invitation
    pub i_uuid: Uuid,
}

/// The url components of an search
#[derive(Serialize, Deserialize, IntoParams, Debug, Clone, Copy)]
pub struct SearchUuid {
    /// The UUID of the workspace
    pub w_uuid: Uuid,
    /// The UUID of the search
    pub s_uuid: Uuid,
}

/// A simple version of a workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SimpleWorkspace {
    /// The uuid of the workspace
    pub uuid: Uuid,
    /// The name of the workspace
    #[schema(example = "ultra-secure-workspace")]
    pub name: String,
    /// The description of the workspace
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub description: Option<String>,
    /// The owner of the workspace
    pub owner: SimpleUser,
    /// The point in time the workspace was created
    pub created_at: DateTime<Utc>,
}

/// A full version of a workspace
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullWorkspace {
    /// The uuid of the workspace
    pub uuid: Uuid,
    #[schema(example = "ultra-secure-workspace")]
    /// The name of the workspace
    pub name: String,
    /// The description of the workspace
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub description: Option<String>,
    /// Notes of the workspace
    pub notes: String,
    /// The owner of the workspace
    pub owner: SimpleUser,
    /// The attacks linked to this workspace
    pub attacks: Vec<SimpleAttack>,
    /// The member of the workspace
    pub members: Vec<SimpleUser>,
    /// The point in time the workspace was created
    pub created_at: DateTime<Utc>,
}

/// The response to retrieve a list of workspaces
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ListWorkspaces {
    /// The list of workspaces
    pub workspaces: Vec<SimpleWorkspace>,
}

/// Dynamic result of a search
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub enum SearchResultEntry {
    /// Host Result
    HostEntry(SimpleHost),
    /// Service Result
    ServiceEntry(SimpleService),
    /// Port Result
    PortEntry(SimplePort),
    /// Domain Result
    DomainEntry(SimpleDomain),
    /// DNS Record Result
    DnsRecordResultEntry(SimpleDnsResolutionResult),
    /// DNS TXT Scan Result
    DnsTxtScanResultEntry(SimpleDnsTxtScanResult),
    /// Dehashed Query Result
    DehashedQueryResultEntry(SimpleQueryUnhashedResult),
    /// Certificate Transparency Result
    CertificateTransparencyResultEntry(FullQueryCertificateTransparencyResult),
    /// Host Alive Result
    HostAliveResult(SimpleHostAliveResult),
    /// Service Detection Result
    ServiceDetectionResult(FullServiceDetectionResult),
    /// UDP Service Detection Result
    UdpServiceDetectionResult(FullUdpServiceDetectionResult),
}

/// Searched entry
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct SearchEntry {
    /// The uuid of the search
    pub uuid: Uuid,
    /// The point in time this search was created
    pub created_at: DateTime<Utc>,
    /// The point in time this search was finished
    pub finished_at: Option<DateTime<Utc>>,
    /// The search term that was used
    pub search_term: String,
}
