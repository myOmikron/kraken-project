//! The schema of the ws manager

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::attack_results::schema::SimpleDnsTxtScanResult;
use crate::api::handler::attacks::schema::SimpleAttack;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::SimpleService;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;

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

/// Message that is sent via websocket
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
        entries: Vec<SimpleDnsTxtScanResult>,
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
