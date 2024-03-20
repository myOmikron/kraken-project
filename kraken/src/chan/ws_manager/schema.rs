//! The schema of the ws manager

use std::net::IpAddr;
use std::num::NonZeroU64;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::attack_results::schema::FullDnsTxtScanResult;
use crate::api::handler::attacks::schema::SimpleAttack;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::finding_affected::schema::UpdateFindingAffectedRequest;
use crate::api::handler::findings::schema::UpdateFindingRequest;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::SimpleService;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::models::OsType;

/// Message that is sent via websocket from the client to the server.
///
/// For messages the server is able to send, look at [WsMessage]
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    /// Content was changed in an editor
    EditorChangedContent {
        /// The changeset
        change: Change,
        /// The target of the editor
        target: EditorTarget,
    },
    /// The cursor position was changed
    EditorChangedCursor {
        /// The new cursor position
        cursor: CursorPosition,
        /// The target of the editor
        target: EditorTarget,
    },
}

/// Message that is sent via websocket
///
/// These messages are only invoked by the Server.
/// For messages the client is able to send, look at [WsClientMessage]
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// An invalid message was received.
    ///
    /// This message type is sent to the client.
    InvalidMessage {},
    /// An invitation to a workspace was issued
    InvitationToWorkspace {
        /// The uuid of the invitation
        invitation_uuid: Uuid,
        /// The workspace the user is invited to
        workspace: SimpleWorkspace,
        /// The user that has issued the invitation
        from: SimpleUser,
    },
    /// A notification about a started attack
    AttackStarted {
        /// The corresponding attack
        attack: SimpleAttack,
    },
    /// A notification about a finished attack
    AttackFinished {
        /// The corresponding attack
        attack: SimpleAttack,
    },
    // TODO: TaskFinished as generic result
    /// A notification about a finished search
    SearchFinished {
        /// The corresponding id of the search
        search_uuid: Uuid,
        /// Whether the search was finished successfully
        finished_successful: bool,
    },
    /// A notification about a search result
    SearchNotify {
        /// The corresponding id of the search results
        search_uuid: Uuid,
        /// A result entry
        result_uuid: Uuid,
    },
    /// A result for a subdomain enumeration using bruteforce DNS requests
    BruteforceSubdomainsResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The source address that was queried
        source: String,
        /// The destination address that was returned
        destination: String,
    },
    /// A result for hosts alive check
    HostsAliveCheck {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// A host which could be reached
        #[schema(value_type = String)]
        host: IpAddr,
    },
    /// A result for a tcp scan
    ScanTcpPortsResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The address of the result
        address: String,
        /// The port of the result
        port: u16,
    },
    /// A result to a certificate transparency request
    CertificateTransparencyResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The entries of the result
        entries: Vec<CertificateTransparencyEntry>,
    },
    /// A result to service detection request
    ServiceDetectionResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The address of the result
        address: String,
        /// The port of the result
        port: u16,
        /// Name of the service(s)
        services: Vec<String>,
    },
    /// A result to UDP service detection request
    UdpServiceDetectionResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The address of the result
        address: String,
        /// The port of the result
        port: u16,
        /// Name of the service(s)
        services: Vec<String>,
    },
    /// A result for a DNS resolution requests
    DnsResolutionResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The source address that was queried
        source: String,
        /// The destination address that was returned
        destination: String,
    },
    /// A result for a DNS TXT scan request
    DnsTxtScanResult {
        /// The corresponding id of the attack
        attack_uuid: Uuid,
        /// The destination address that was returned
        result: FullDnsTxtScanResult,
    },
    /// Result for an OS detection request
    OsDetectionResult {
        /// The major operating system type
        os: OsType,
        /// A host which could be reached
        #[schema(value_type = String)]
        host: IpAddr,
        /// Human-readable extra hints for the OS, new-line (\n) separated
        hints: String,
        /// Optional additional version information, separated by OR (`" OR "`)
        version: String,
    },
    /// A new domain was found
    NewDomain {
        /// The workspace this domain is related to
        workspace: Uuid,
        /// The domain that was inserted
        domain: SimpleDomain,
    },
    /// A new host was found
    NewHost {
        /// The workspace this host is related to
        workspace: Uuid,
        /// The host that was inserted
        host: SimpleHost,
    },
    /// A new port was found
    NewPort {
        /// The workspace this port is related to
        workspace: Uuid,
        /// The port that was inserted
        port: SimplePort,
    },
    /// A new service was found
    NewService {
        /// The workspace this service is related to
        workspace: Uuid,
        /// The service that was inserted
        service: SimpleService,
    },
    /// A domain was deleted
    DeletedDomain {
        /// The workspace this domain is related to
        workspace: Uuid,
        /// The uuid of the deleted domain
        domain: Uuid,
    },
    /// A host was deleted
    DeletedHost {
        /// The workspace this host is related to
        workspace: Uuid,
        /// The uuid of the deleted host
        host: Uuid,
    },
    /// A port was deleted
    DeletedPort {
        /// The workspace this port is related to
        workspace: Uuid,
        /// The uuid of the deleted port
        port: Uuid,
    },
    /// A service was deleted
    DeletedService {
        /// The workspace this service is related to
        workspace: Uuid,
        /// The uuid of the deleted service
        service: Uuid,
    },
    /// Global tags were updated on an aggregation
    UpdatedGlobalTags {
        /// The workspace the aggregation is related to
        workspace: Uuid,
        /// The type of the aggregation
        aggregation: AggregationType,
        /// The uuid of the model
        uuid: Uuid,
        /// The updated list of tags
        tags: Vec<Uuid>,
    },
    /// Workspace tags were updated on an aggregation
    UpdatedWorkspaceTags {
        /// The workspace the aggregation is related to
        workspace: Uuid,
        /// The type of the aggregation
        aggregation: AggregationType,
        /// The uuid of the model
        uuid: Uuid,
        /// The updated list of tags
        tags: Vec<Uuid>,
    },
    /// A finding definition was deleted
    DeletedFindingDefinition {
        /// The uuid of the finding definition
        uuid: Uuid,
    },
    /// A finding definition was updated
    EditorChangedContent {
        /// The changeset
        change: Change,
        /// The user that has done the change
        user: SimpleUser,
        /// The target of the editor
        target: EditorTarget,
    },
    /// A user has changed its cursor position in an editor
    EditorChangedCursor {
        /// The user that changed the position
        user: SimpleUser,
        /// The target of the editor
        target: EditorTarget,
        /// The new cursor position
        cursor: CursorPosition,
    },
    /// A finding has been updated
    UpdatedFinding {
        /// The workspace the updated finding is in
        workspace: Uuid,
        /// The finding which has been updated
        finding: Uuid,
        /// The update
        update: UpdateFindingRequest,
    },
    /// An affected has been added to a finding
    AddedFindingAffected {
        /// The workspace the updated finding is in
        workspace: Uuid,
        /// The finding which has been updated
        finding: Uuid,
        /// The affected's uuid
        affected_uuid: Uuid,
        /// The affected's type
        affected_type: AggregationType,
    },
    /// A finding's affected has been updated
    UpdatedFindingAffected {
        /// The workspace the updated finding is in
        workspace: Uuid,
        /// The finding which has been updated
        finding: Uuid,
        /// The affected's uuid
        affected_uuid: Uuid,
        /// The update
        update: UpdateFindingAffectedRequest,
    },
    /// An affected has been removed to a finding
    RemovedFindingAffected {
        /// The workspace the updated finding is in
        workspace: Uuid,
        /// The finding which has been updated
        finding: Uuid,
        /// The affected's uuid
        affected_uuid: Uuid,
    },
}

/// The target of the editor
///
/// Used to specify the target for the editor, for example the
/// specific section in a [FindingDefinition]
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone, Copy)]
pub enum EditorTarget {
    /// The editor for a [FindingDefinition]
    FindingDefinition {
        /// The finding definition that is active
        finding_definition: Uuid,
        /// The finding section which is active
        finding_section: FindingSection,
    },
    /// The editor for the `user_details` in [Finding]
    Finding {
        /// Uuid of the [Finding]
        finding: Uuid,
    },
    /// The editor for the `user_details` in [FindingAffected]
    FindingAffected {
        /// Uuid of the [Finding]
        finding: Uuid,
        /// Uuid of the [FindingAffected]
        affected: Uuid,
    },
    /// The editor for notes in a [Workspace]
    WorkspaceNotes {
        /// The uuid of the workspace
        workspace: Uuid,
    },
}

/// The different types of aggregations
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone, Copy)]
pub enum AggregationType {
    /// The domain model
    Domain,
    /// The host model
    Host,
    /// The service model
    Service,
    /// The port model
    Port,
}

/// The section that was edited
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[allow(missing_docs)]
pub enum FindingSection {
    Summary,
    Description,
    Impact,
    Remediation,
    References,
}

/// Defines a change
///
/// Columns and lines are treated as 1-indexed
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Change {
    /// The text that should be set to the range given by the other values
    pub text: String,
    /// Start of the column
    #[schema(value_type = u64, minimum = 1)]
    pub start_column: NonZeroU64,
    /// End of the column
    #[schema(value_type = u64, minimum = 1)]
    pub end_column: NonZeroU64,
    /// Starting line number
    #[schema(value_type = u64, minimum = 1)]
    pub start_line: NonZeroU64,
    /// Ending line number
    #[schema(value_type = u64, minimum = 1)]
    pub end_line: NonZeroU64,
}

/// Defines this position of a cursor
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct CursorPosition {
    /// The line the cursor was placed in
    #[schema(value_type = u64, minimum = 1)]
    pub line: NonZeroU64,
    /// The column the cursor was placed in
    #[schema(value_type = u64, minimum = 1)]
    pub column: NonZeroU64,
}

/// Entry of certificate transparency results
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct CertificateTransparencyEntry {
    /// The serial number of the certificate
    pub serial_number: String,
    /// The name of the issuer for the certificate
    pub issuer_name: String,
    /// The common name of the certificate
    pub common_name: String,
    /// The value names of the certificate
    pub value_names: Vec<String>,
    /// The point in time after the certificate is valid
    pub not_before: Option<DateTime<Utc>>,
    /// The point in time before the certificate is valid
    pub not_after: Option<DateTime<Utc>>,
}
