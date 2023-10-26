mod patches;

use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use rorm::prelude::*;
use uuid::Uuid;

pub(crate) use crate::models::search::patches::*;
use crate::models::{User, Workspace};

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
    /// [Attack](crate::models::Attack) type
    Attack,
    /// [Host](crate::models::Host) type
    Host,
    /// [Service](crate::models::Service) type
    Service,
    /// [Port](crate::models::Port) type
    Port,
    /// [Domain](crate::models::Domain) type
    Domain,
    /// [DnsRecordResult](crate::models::DnsRecordResult) type
    DnsRecordResult,
    /// [TcpPortScanResult](crate::models::TcpPortScanResult) type
    TcpPortScanResult,
    /// [DehashedQueryResult](crate::models::DehashedQueryResult) type
    DehashedQueryResult,
    /// [CertificateTransparencyResult](crate::models::CertificateTransparencyResult) type
    CertificateTransparencyResult,
    /// [HostAliveResult](crate::models::HostAliveResult) type
    HostAliveResult,
    /// [ServiceDetectionResult](crate::models::ServiceDetectionResult) type
    ServiceDetectionResult,
}

impl Display for ModelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Attack => {
                write!(f, "Attack")
            }
            ModelType::Host => {
                write!(f, "Host")
            }
            ModelType::Service => {
                write!(f, "Service")
            }
            ModelType::Port => {
                write!(f, "Port")
            }
            ModelType::Domain => {
                write!(f, "Domain")
            }
            ModelType::DnsRecordResult => {
                write!(f, "DnsRecordResult")
            }
            ModelType::TcpPortScanResult => {
                write!(f, "TcpPortScanResult")
            }
            ModelType::DehashedQueryResult => {
                write!(f, "DehashedQueryResult")
            }
            ModelType::CertificateTransparencyResult => {
                write!(f, "CertificateTransparencyResult")
            }
            ModelType::HostAliveResult => {
                write!(f, "HostAliveResult")
            }
            ModelType::ServiceDetectionResult => {
                write!(f, "ServiceDetectionResult")
            }
        }
    }
}
