//! This module holds all the information regarding attacks

use std::net::IpAddr;

use rorm::fields::{ForeignModel, Json};
use rorm::{DbEnum, Model, Patch};

use crate::models::{User, Workspace};

/// The type of an attack
#[derive(Copy, Clone, DbEnum)]
pub enum AttackType {
    /// First variant to be mapped for 0
    Undefined,
    /// Bruteforce subdomains via DNS requests
    BruteforceSubdomains,
    /// Scan tcp ports
    TcpPortScan,
    /// Query certificate transparency
    QueryCertificateTransparency,
}

/// Representation of an attack
#[derive(Model)]
pub struct Attack {
    /// The primary key
    #[rorm(id)]
    pub id: i64,

    /// The type of the attack.
    ///
    /// Currently only an integer as rorm currently hasn't support for this.
    ///
    /// Use [AttackType] for use in kraken.
    pub attack_type: AttackType,

    /// The user that started this attack
    pub started_from: ForeignModel<User>,

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
    pub(crate) attack_type: AttackType,
    pub(crate) started_from: ForeignModel<User>,
    pub(crate) workspace: ForeignModel<Workspace>,
    pub(crate) finished_at: Option<chrono::NaiveDateTime>,
}

/// Representation of a [tcp port scan](AttackType::TcpPortScan) attack's result
#[derive(Model)]
pub struct TcpPortScanResult {
    /// The primary key
    #[rorm(id)]
    pub id: i64,

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
    pub(crate) attack: ForeignModel<Attack>,
    pub(crate) address: Json<IpAddr>,
    pub(crate) port: i32,
}
