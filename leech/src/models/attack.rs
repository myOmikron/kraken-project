//! This module holds all the information regarding attacks

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use rorm::prelude::*;
use uuid::Uuid;

/// The type of [DNS Record](crate::rpc::rpc_attacks::shared::dns_record::Record)
#[derive(DbEnum)]
pub enum DnsRecordType {
    /// [A](crate::rpc::rpc_attacks::shared::dns_record::Record::A) record type
    A,
    /// [Aaaa](crate::rpc::rpc_attacks::shared::dns_record::Record::Aaaa) record type
    Aaaa,
    /// [Caa](crate::rpc::rpc_attacks::shared::dns_record::Record::GenericRecord) record type
    Caa,
    /// [Cname](crate::rpc::rpc_attacks::shared::dns_record::Record::GenericRecord) record type
    Cname,
    /// [Mx](crate::rpc::rpc_attacks::shared::dns_record::Record::GenericRecord) record type
    Mx,
    /// [Tlsa](crate::rpc::rpc_attacks::shared::dns_record::Record::GenericRecord) record type
    Tlsa,
    /// [Txt](crate::rpc::rpc_attacks::shared::dns_record::Record::GenericRecord) record type
    Txt,
}

/// Representation of a [Bruteforce Subdomain](AttackType::BruteforceSubdomains) attack's result
pub type BruteforceSubdomainsResult = DnsResult;
pub(crate) type BruteforceSubdomainsResultInsert = DnsResultInsert;

/// Generic representation of a DNS result
#[derive(Model)]
pub struct DnsResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The [source](crate::rpc::rpc_attacks::shared::dns_record::Record)
    #[rorm(max_length = 255)]
    pub source: String,

    /// The [destination address](IpNetwork)
    #[rorm(max_length = 255)]
    pub destination: String,

    /// The type of [DNS record type](DnsRecordType)
    pub dns_record_type: DnsRecordType,

    /// The point in time, this result was produced
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

#[derive(Patch)]
#[rorm(model = "DnsResult")]
pub(crate) struct DnsResultInsert {
    pub(crate) uuid: Uuid,
    pub(crate) attack: Uuid,
    pub(crate) source: String,
    pub(crate) destination: String,
    pub(crate) dns_record_type: DnsRecordType,
}

/// Representation of a [tcp port scan](AttackType::TcpPortScan) attack's result
#[derive(Model)]
pub struct TcpPortScanResult {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack which produced this result
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub attack: Uuid,

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

#[derive(Patch)]
#[rorm(model = "TcpPortScanResult")]
pub(crate) struct TcpPortScanResultInsert {
    pub(crate) uuid: Uuid,
    pub(crate) attack: Uuid,
    pub(crate) address: IpNetwork,
    pub(crate) port: i32,
}
