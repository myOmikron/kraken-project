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

#[derive(Model)]
pub struct ManualHttpService {
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The service's name
    #[rorm(max_length = 255)]
    pub name: String,

    /// Optional version of the http service
    #[rorm(index, max_length = 255)]
    pub version: Option<String>,

    /// The service's domain
    #[rorm(max_length = 255)]
    pub domain: Option<String>,

    /// The service's ip address
    pub ip_addr: IpNetwork,

    /// The service's port
    pub port: i32,

    /// The service's port's protocol
    pub port_protocol: PortProtocol,

    /// The certainty of this http service
    pub certainty: ManualHttpServiceCertainty,

    /// The base path the service is routed on
    ///
    /// (Should default to "/")
    #[rorm(max_length = 1024)]
    pub base_path: String,

    /// Is this a https service?
    pub tls: bool,

    /// Does this service require sni?
    pub sni_required: bool,

    /// The user which added the http service
    pub user: ForeignModel<User>,

    /// A reference to the workspace this http service is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The certainty of a manually added http service
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum ManualHttpServiceCertainty {
    /// Historical data
    Historical,
    /// Up to date data
    SupposedTo,
}
