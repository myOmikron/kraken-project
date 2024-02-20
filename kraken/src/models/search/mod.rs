use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use rorm::prelude::*;
use uuid::Uuid;

use crate::models;
#[cfg(feature = "bin")]
pub(crate) use crate::models::search::patches::*;
use crate::models::{User, Workspace};
#[cfg(feature = "bin")]
mod patches;

/// Saves the search parameters
#[derive(Model)]
pub struct Search {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The user that started this search
    pub started_by: ForeignModel<User>,

    /// The workspace this search was started from/for
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this search has finished
    pub finished_at: Option<DateTime<Utc>>,

    /// Contains an error message if the search didn't finish successfully
    #[rorm(max_length = 255)]
    pub error: Option<String>,

    /// The point in time, this search was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The term that was searched for
    #[rorm(max_length = 20)]
    pub search_term: String,

    /// The results this search yielded
    pub results: BackRef<field!(SearchResult::F.search)>,
}

/// Saves the uuid and the Model Type the uuid belongs to
/// so that the correct row in said model can be retrieved
#[derive(Model)]
pub struct SearchResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The primary key of the 'ForeignModel'
    pub ref_key: Uuid,

    /// The table the Uuid refers to
    #[rorm(index)]
    pub ref_type: ModelType,

    /// Link this result to a search
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub search: ForeignModel<Search>,
}

/// Model Types
#[derive(DbEnum, Copy, Clone)]
pub enum ModelType {
    /// [Host](models::Host) type
    Host,
    /// [Service](models::Service) type
    Service,
    /// [Port](models::Port) type
    Port,
    /// [Domain](models::Domain) type
    Domain,
    /// [DnsRecordResult](models::DnsRecordResult) type
    DnsRecordResult,
    /// [DnsTxtScanResult](models::DnsTxtScanResult) type
    DnsTxtScanResult,

    /// Effectively deleted, but postgres can't delete enum variants
    TcpPortScanResult,

    /// [DehashedQueryResult](models::DehashedQueryResult) type
    DehashedQueryResult,
    /// [CertificateTransparencyResult](models::CertificateTransparencyResult) type
    CertificateTransparencyResult,
    /// [HostAliveResult](models::HostAliveResult) type
    HostAliveResult,
    /// [ServiceDetectionResult](models::ServiceDetectionResult) type
    ServiceDetectionResult,
    /// [UdpServiceDetectionResult](models::UdpServiceDetectionResult) type
    UdpServiceDetectionResult,
}

impl Display for ModelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Host => {
                write!(f, "{}", models::Host::TABLE)
            }
            ModelType::Service => {
                write!(f, "{}", models::Service::TABLE)
            }
            ModelType::Port => {
                write!(f, "{}", models::Port::TABLE)
            }
            ModelType::Domain => {
                write!(f, "{}", models::Domain::TABLE)
            }
            ModelType::DnsRecordResult => {
                write!(f, "{}", models::DnsRecordResult::TABLE)
            }
            ModelType::DnsTxtScanResult => {
                write!(f, "{}", models::DnsTxtScanAttackResult::TABLE)
            }
            ModelType::TcpPortScanResult => Err(std::fmt::Error),
            ModelType::DehashedQueryResult => {
                write!(f, "{}", models::DehashedQueryResult::TABLE)
            }
            ModelType::CertificateTransparencyResult => {
                write!(f, "{}", models::CertificateTransparencyResult::TABLE)
            }
            ModelType::HostAliveResult => {
                write!(f, "{}", models::HostAliveResult::TABLE)
            }
            ModelType::ServiceDetectionResult => {
                write!(f, "{}", models::ServiceDetectionResult::TABLE)
            }
            ModelType::UdpServiceDetectionResult => {
                write!(f, "{}", models::UdpServiceDetectionResult::TABLE)
            }
        }
    }
}
