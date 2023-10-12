//! This module holds all the information regarding attacks

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use rorm::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub use crate::models::attack::operations::*;
pub(crate) use crate::models::attack::patches::*;
use crate::models::{Certainty, User, Workspace};

mod operations;
mod patches;

/// The type of an attack
#[derive(Copy, Clone, DbEnum, ToSchema, Serialize, Deserialize)]
pub enum AttackType {
    /// First variant to be mapped for 0
    Undefined,
    /// Bruteforce subdomains via DNS requests
    BruteforceSubdomains,
    /// Scan tcp ports
    TcpPortScan,
    /// Query certificate transparency
    QueryCertificateTransparency,
    /// Query the unhashed API
    QueryUnhashed,
    /// Check if a host is reachable via icmp
    HostAlive,
    /// Detect the service that is running on a port
    ServiceDetection,
    /// Resolve domain names
    DnsResolution,
}

/// Representation of an attack
#[derive(Model)]
pub struct Attack {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [type](AttackType) of the attack.
    pub attack_type: AttackType,

    /// The user that started this attack
    pub started_by: ForeignModel<User>,

    /// The workspace this attack was started from
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this attack has finished
    pub finished_at: Option<DateTime<Utc>>,

    /// Contains an error message if the attack didn't finish successfully
    #[rorm(max_length = 255)]
    pub error: Option<String>,

    /// The point in time, this attack was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The type of DNS Record
#[derive(Copy, Clone, DbEnum)]
pub enum DnsRecordType {
    /// [A](crate::rpc::rpc_definitions::shared::dns_record::Record::A) record type
    A,
    /// [Aaaa](crate::rpc::rpc_definitions::shared::dns_record::Record::Aaaa) record type
    Aaaa,
    /// [Caa](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Caa,
    /// [Cname](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Cname,
    /// [Mx](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Mx,
    /// [Tlsa](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Tlsa,
    /// [Txt](crate::rpc::rpc_definitions::shared::dns_record::Record::GenericRecord) record type
    Txt,
}

/// Representation of a [Bruteforce Subdomain](AttackType::BruteforceSubdomains) attack's result
pub type BruteforceSubdomainsResult = DnsRecordResult;

/// Representation of a [DNS resolution](AttackType::DnsResolution) attack's result
pub type DnsResolutionResult = DnsRecordResult;

/// Generic representation of a DNS result
#[derive(Model)]
pub struct DnsRecordResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The source address
    #[rorm(max_length = 255)]
    pub source: String,

    /// The destination address
    #[rorm(max_length = 255)]
    pub destination: String,

    /// The type of [DNS record type](DnsRecordType)
    pub dns_record_type: DnsRecordType,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// Representation of a [tcp port scan](AttackType::TcpPortScan) attack's result
#[derive(Model)]
pub struct TcpPortScanResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The ip address a port was found on
    pub address: IpNetwork,

    /// The found port
    ///
    /// Stored in db as `i32` but ports are actually just an `u16`
    pub port: i32,
}

/// Representation of a [dehashed query](AttackType::Dehashed) result
#[derive(Model)]
pub struct DehashedQueryResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// ID of the entry
    pub dehashed_id: i64,
    /// An email address, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub email: Option<String>,
    /// An username, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub username: Option<String>,
    /// A password, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub password: Option<String>,
    /// An hashed password, may be [None] if the result didn't include this field
    #[rorm(max_length = 8192)]
    pub hashed_password: Option<String>,
    /// An ip address, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub ip_address: Option<IpNetwork>,
    /// A name, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub name: Option<String>,
    /// A vin, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub vin: Option<String>,
    /// An address, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub address: Option<String>,
    /// A phone, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub phone: Option<String>,
    /// A database name, may be [None] if the result didn't include this field
    #[rorm(max_length = 255)]
    pub database_name: Option<String>,
}

/// A value name in a [AttackType::QueryCertificateTransparency] result
#[derive(Model)]
pub struct CertificateTransparencyValueName {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// a single value
    #[rorm(max_length = 255)]
    pub value_name: String,

    /// The result this value is originating from
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub ct_result: ForeignModel<CertificateTransparencyResult>,
}

/// Representation of a [AttackType::QueryCertificateTransparency] result
#[derive(Model)]
pub struct CertificateTransparencyResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The name of the issuer
    #[rorm(max_length = 255)]
    pub issuer_name: String,
    /// The common name of the certificate
    #[rorm(max_length = 255)]
    pub common_name: String,
    /// The values of the certificate
    pub value_names: BackRef<field!(CertificateTransparencyValueName::F.ct_result)>,
    /// The start date of the certificate
    pub not_before: Option<DateTime<Utc>>,
    /// The end date of the certificate
    pub not_after: Option<DateTime<Utc>>,
    /// The serial number of the certificate
    #[rorm(max_length = 255)]
    pub serial_number: String,
}

/// Representation of a [Host Alive](AttackType::HostAlive) attack's result
#[derive(Model)]
pub struct HostAliveResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// A host that responded
    pub host: IpNetwork,
}

/// The name of a result of a service that was found during a service detection
#[derive(Model)]
pub struct ServiceDetectionName {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The name of found service
    #[rorm(max_length = 255)]
    pub name: String,

    /// The result this service name is linked to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub result: ForeignModel<ServiceDetectionResult>,
}

/// Representation of a [Service Detection](AttackType::ServiceDetection) attack's result
#[derive(Model)]
pub struct ServiceDetectionResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The [attack](Attack) which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: ForeignModel<Attack>,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The certainty of the result
    pub certainty: Certainty,

    /// The ip address a port was found on
    pub host: IpNetwork,

    /// Port number
    ///
    /// Reinterpret as u16 with to_ne_bytes and from_ne_bytes
    pub port: i16,

    /// The found names of the service
    pub service_names: BackRef<field!(ServiceDetectionName::F.result)>,
}
