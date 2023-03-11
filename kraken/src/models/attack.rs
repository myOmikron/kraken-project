//! This module holds all the information regarding attacks

use rorm::fields::ForeignModel;
use rorm::{DbEnum, Model, Patch};

use crate::models::User;

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
    pub(crate) finished_at: Option<chrono::NaiveDateTime>,
}
