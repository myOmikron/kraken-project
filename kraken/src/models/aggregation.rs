use ipnetwork::IpNetwork;
use rorm::prelude::ForeignModel;
use rorm::{DbEnum, Model, Patch};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::Workspace;

/// A representation of an OS type
#[derive(DbEnum, Copy, Clone, Debug, ToSchema, Serialize)]
pub enum OsType {
    /// The OS type is currently unknown
    Unknown,
    /// Linux based OS
    Linux,
    /// Windows based OS
    Windows,
    /// Apple based OS
    Apple,
    /// Android based OS
    Android,
    /// FreeBSD based OS
    FreeBSD,
}

/// A representation of an host.
///
/// Will be collected from all results that yield IP addresses
#[derive(Model)]
pub struct Host {
    /// The primary key of a host
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The IP address of the host.
    ///
    /// If the host has multiple addresses, create a [Host] for each and link them.
    pub ip_addr: IpNetwork,

    /// The type of OS of this host
    pub os_type: OsType,

    /// A comment to the host
    #[rorm(max_length = 255)]
    pub comment: String,

    /// A reference to the workspace this host is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Host")]
pub(crate) struct HostInsert {
    pub(crate) uuid: Uuid,
    pub(crate) ip_addr: IpNetwork,
    pub(crate) os_type: OsType,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}
