#![allow(missing_docs)]
//! This module defines the models to store information about manually inserted aggregations

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use rorm::prelude::*;
use rorm::Model;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::OsType;
use crate::models::PortProtocol;
use crate::models::User;
use crate::models::Workspace;

mod convert;
#[cfg(feature = "bin")]
mod operations;

#[derive(Model)]
pub struct ManualHost {
    #[rorm(primary_key)]
    pub uuid: Uuid,

    pub ip_addr: IpNetwork,

    pub os_type: OsType,

    pub certainty: ManualHostCertainty,

    /// The user which added the host
    pub user: ForeignModel<User>,

    /// A reference to the workspace this host was added in
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The certainty of a manually added host
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum ManualHostCertainty {
    /// Historical data
    Historical,
    /// Up to date data
    SupposedTo,
}

#[derive(Model)]
pub struct ManualService {
    #[rorm(primary_key)]
    pub uuid: Uuid,

    #[rorm(max_length = 255)]
    pub name: String,

    #[rorm(max_length = 255)]
    pub version: Option<String>,

    pub certainty: ManualServiceCertainty,

    pub host: IpNetwork,

    pub port: Option<i32>,
    pub protocol: PortProtocol,

    #[rorm(default = 0)] // 0 = Unknown
    pub protocols: i16,

    /// The user which added the host
    pub user: ForeignModel<User>,

    /// A reference to the workspace this service is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The certainty of a manually added service
#[derive(Debug, Copy, Clone, ToSchema, Deserialize, Serialize, DbEnum)]
pub enum ManualServiceCertainty {
    /// Historical data
    Historical,
    /// Up to date data
    SupposedTo,
}

#[derive(Model)]
pub struct ManualPort {
    #[rorm(primary_key)]
    pub uuid: Uuid,

    #[rorm(index)]
    pub port: i32,

    pub protocol: PortProtocol,

    pub certainty: ManualPortCertainty,

    pub host: IpNetwork,

    /// The user which added the port
    pub user: ForeignModel<User>,

    /// A reference to the workspace this port is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The certainty of a manually added port
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum ManualPortCertainty {
    /// Historical data
    Historical,
    /// Up to date data
    SupposedTo,
}

#[derive(Model)]
pub struct ManualDomain {
    #[rorm(primary_key)]
    pub uuid: Uuid,

    #[rorm(max_length = 255)]
    pub domain: String,

    /// The user which added the domain
    pub user: ForeignModel<User>,

    /// A reference to the workspace this domain is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}
