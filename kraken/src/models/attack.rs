//! This module holds all the information regarding attacks

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use rorm::fields::{ForeignModel, Json};
use rorm::{DbEnum, Model, Patch};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{User, Workspace};

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
}

/// Representation of an attack
#[derive(Model)]
pub struct Attack {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The type of the attack.
    ///
    /// Currently only an integer as rorm currently hasn't support for this.
    ///
    /// Use [AttackType] for use in kraken.
    pub attack_type: AttackType,

    /// The user that started this attack
    pub started_by: ForeignModel<User>,

    /// The workspace this attack was started from
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this attack has finished
    pub finished_at: Option<chrono::NaiveDateTime>,

    /// The point in time, this attack was created
    #[rorm(auto_create_time)]
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Patch)]
#[rorm(model = "Attack")]
pub(crate) struct AttackInsert {
    pub(crate) uuid: Uuid,
    pub(crate) attack_type: AttackType,
    pub(crate) started_by: ForeignModel<User>,
    pub(crate) workspace: ForeignModel<Workspace>,
    pub(crate) finished_at: Option<chrono::NaiveDateTime>,
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
    pub created_at: chrono::NaiveDateTime,

    /// The ip address a port was found on
    pub address: Json<IpAddr>,

    /// The found port
    ///
    /// Stored in db as `i32` but ports are actually just an `u16`
    pub port: i32,
}

#[derive(Patch)]
#[rorm(model = "TcpPortScanResult")]
pub(crate) struct TcpPortScanResultInsert {
    pub(crate) uuid: Uuid,
    pub(crate) attack: ForeignModel<Attack>,
    pub(crate) address: Json<IpAddr>,
    pub(crate) port: i32,
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
    pub ip_address: Json<Option<IpAddr>>,
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

#[derive(Patch)]
#[rorm(model = "DehashedQueryResult")]
pub(crate) struct DehashedQueryResultInsert {
    pub(crate) uuid: Uuid,
    pub(crate) attack: ForeignModel<Attack>,
    pub(crate) dehashed_id: i64,
    pub(crate) email: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) hashed_password: Option<String>,
    pub(crate) ip_address: Json<Option<IpAddr>>,
    pub(crate) name: Option<String>,
    pub(crate) vin: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) phone: Option<String>,
    pub(crate) database_name: Option<String>,
}
