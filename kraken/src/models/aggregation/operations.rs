use ipnetwork::IpNetwork;
use rorm::prelude::*;
use uuid::Uuid;

use crate::models::{Domain, Host, OsType, Port, Service, Workspace};

#[derive(Patch)]
#[rorm(model = "Host")]
pub(crate) struct HostInsert {
    pub(crate) uuid: Uuid,
    pub(crate) ip_addr: IpNetwork,
    pub(crate) os_type: OsType,
    pub(crate) response_time: Option<i32>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Service")]
pub(crate) struct ServiceInsert {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) host: ForeignModel<Host>,
    pub(crate) port: Option<ForeignModel<Port>>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Port")]
pub(crate) struct PortInsert {
    pub(crate) uuid: Uuid,
    pub(crate) port: i16,
    pub(crate) host: ForeignModel<Host>,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}
#[derive(Patch)]
#[rorm(model = "Domain")]
pub(crate) struct DomainInsert {
    pub(crate) uuid: Uuid,
    pub(crate) domain: String,
    pub(crate) comment: String,
    pub(crate) workspace: ForeignModel<Workspace>,
}
