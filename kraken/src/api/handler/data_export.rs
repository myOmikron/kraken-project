//! This module holds the data export of a workspace
//!
//! Data can be exported by an oauth application that was registered by an admin and has
//! access to a workspace granted by an user.

use std::collections::HashMap;

use actix_web::get;
use actix_web::web::{Data, Json, Path};
use chrono::Utc;
use futures::TryStreamExt;
use ipnetwork::IpNetwork;
use rorm::prelude::*;
use rorm::{and, query, Database};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::BearerToken;
use crate::api::handler::{ApiError, ApiResult, PathUuid};
use crate::models::{
    Certainty, Domain, DomainGlobalTag, DomainWorkspaceTag, Host, HostGlobalTag, HostWorkspaceTag,
    OsType, Port, PortGlobalTag, PortProtocol, PortWorkspaceTag, Service, ServiceGlobalTag,
    ServiceWorkspaceTag, WorkspaceAccessToken,
};

/// The aggregated results of a workspace
#[derive(Serialize, ToSchema)]
pub struct AggregatedWorkspace {
    /// The hosts found by this workspace
    pub hosts: HashMap<Uuid, AggregatedHost>,

    /// The ports found by this workspace
    pub ports: HashMap<Uuid, AggregatedPort>,

    /// The services found by this workspace
    pub services: HashMap<Uuid, AggregatedService>,

    /// The domains found by this workspace
    pub domains: HashMap<Uuid, AggregatedDomain>,
}

/// A representation of an host.
#[derive(Serialize, ToSchema)]
pub struct AggregatedHost {
    /// The host's uuid
    pub uuid: Uuid,

    /// The IP address of the host.
    ///
    /// If the host has multiple addresses, create a [Host] for each and link them.
    #[schema(value_type = String)]
    pub ip_addr: IpNetwork,

    /// The type of OS of this host
    pub os_type: OsType,

    /// Response time in ms
    pub response_time: Option<i32>,

    /// The ports of a host
    pub ports: Vec<Uuid>,

    /// The services of a host
    pub services: Vec<Uuid>,

    /// A comment to the host
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,
}

/// An open port on a host
#[derive(Serialize, ToSchema)]
pub struct AggregatedPort {
    /// The port's uuid
    pub uuid: Uuid,

    /// Port number
    pub port: u16,

    /// Port protocol
    pub protocol: PortProtocol,

    /// The host this service is attached to
    pub host: Uuid,

    /// The services that link to this port
    pub services: Vec<Uuid>,

    /// A comment to the port
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,
}

/// A detected service on a host
#[derive(Serialize, ToSchema)]
pub struct AggregatedService {
    /// The service's uuid
    pub uuid: Uuid,

    /// Name of the service
    pub name: String,

    /// Optional version of the service
    pub version: Option<String>,

    /// The host this service is attached to
    pub host: Uuid,

    /// The port this service is attached to
    pub port: Option<Uuid>,

    /// A comment to the service
    pub comment: String,

    /// The certainty the service was detected
    pub certainty: Certainty,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,
}

/// A domain
#[derive(Serialize, ToSchema)]
pub struct AggregatedDomain {
    /// The domain's uuid
    pub uuid: Uuid,

    /// The domain that was found
    pub domain: String,

    /// A comment to the domain
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,
}

/// Set of global and local tags
#[derive(Serialize, ToSchema, Default)]
pub struct AggregatedTags {
    /// Global tags
    global_tags: Vec<String>,

    /// Tags which are local to the workspace
    local_tags: Vec<String>,
}

#[utoipa::path(
    tag = "Data Export",
    context_path = "/api/v1/export",
    responses(
        (status = 200, description = "All hosts in the workspace", body = AggregatedWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("bearer_token" = []))
)]
#[get("/workspace/{uuid}")]
pub(crate) async fn export_workspace(
    db: Data<Database>,
    path: Path<PathUuid>,
    token: BearerToken,
) -> ApiResult<Json<AggregatedWorkspace>> {
    let mut tx = db.start_transaction().await?;

    // Check access
    query!(&mut tx, (WorkspaceAccessToken::F.id,))
        .condition(and![
            WorkspaceAccessToken::F.token.equals(token.as_str()),
            WorkspaceAccessToken::F.workspace.equals(path.uuid),
            WorkspaceAccessToken::F.expires_at.greater_than(Utc::now()),
        ])
        .optional()
        .await?
        .ok_or(ApiError::MissingPrivileges)?;

    // Query all models without joins
    let mut hosts: HashMap<Uuid, AggregatedHost> = query!(&mut tx, Host)
        .condition(Host::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|host| (host.uuid, host.into()))
        .try_collect()
        .await?;
    let mut ports: HashMap<Uuid, AggregatedPort> = query!(&mut tx, Port)
        .condition(Port::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|port| (port.uuid, port.into()))
        .try_collect()
        .await?;
    let mut services: HashMap<Uuid, AggregatedService> = query!(&mut tx, Service)
        .condition(Service::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|service| (service.uuid, service.into()))
        .try_collect()
        .await?;
    let mut domains: HashMap<Uuid, AggregatedDomain> = query!(&mut tx, Domain)
        .condition(Domain::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|domain| (domain.uuid, domain.into()))
        .try_collect()
        .await?;

    // Resolve BackRefs manually
    for service in services.values() {
        if let Some(host) = hosts.get_mut(&service.host) {
            host.services.push(service.uuid);
        }
        if let Some(port) = service.port.as_ref() {
            if let Some(port) = ports.get_mut(port) {
                port.services.push(service.uuid);
            }
        }
    }
    for port in ports.values() {
        if let Some(host) = hosts.get_mut(&port.uuid) {
            host.ports.push(port.uuid);
        }
    }

    // Query all tags
    macro_rules! query_tags {
        ($owner:ident, $owner_set:ident, $GlobalTag:ident, $WorkspaceTag:ident) => {
            let mut stream = query!(
                &mut tx,
                ($GlobalTag::F.$owner.uuid, $GlobalTag::F.global_tag.name)
            )
            .condition($GlobalTag::F.$owner.workspace.equals(path.uuid))
            .stream();
            while let Some((owner_uuid, name)) = stream.try_next().await? {
                if let Some(owner) = $owner_set.get_mut(&owner_uuid) {
                    owner.tags.global_tags.push(name);
                }
            }
            drop(stream);
            let mut stream = query!(
                &mut tx,
                ($WorkspaceTag::F.$owner, $WorkspaceTag::F.workspace_tag.name)
            )
            .condition($WorkspaceTag::F.workspace_tag.workspace.equals(path.uuid))
            .stream();
            while let Some((owner_uuid, name)) = stream.try_next().await? {
                if let Some(owner) = $owner_set.get_mut(owner_uuid.key()) {
                    owner.tags.local_tags.push(name);
                }
            }
            drop(stream);
        };
    }
    query_tags!(host, hosts, HostGlobalTag, HostWorkspaceTag);
    query_tags!(port, ports, PortGlobalTag, PortWorkspaceTag);
    query_tags!(service, services, ServiceGlobalTag, ServiceWorkspaceTag);
    query_tags!(domain, domains, DomainGlobalTag, DomainWorkspaceTag);

    tx.commit().await?;
    Ok(Json(AggregatedWorkspace {
        hosts,
        ports,
        services,
        domains,
    }))
}

impl From<Host> for AggregatedHost {
    fn from(value: Host) -> Self {
        let Host {
            uuid,
            ip_addr,
            os_type,
            response_time,
            ports: _,
            services: _,
            comment,
            workspace: _,
        } = value;
        Self {
            uuid,
            ip_addr,
            os_type,
            response_time,
            ports: Vec::new(),
            services: Vec::new(),
            comment,
            tags: Default::default(),
        }
    }
}
impl From<Port> for AggregatedPort {
    fn from(value: Port) -> Self {
        let Port {
            uuid,
            port,
            protocol,
            host,
            services: _,
            comment,
            workspace: _,
        } = value;
        Self {
            uuid,
            port: u16::from_ne_bytes(port.to_ne_bytes()),
            protocol,
            host: *host.key(),
            services: Vec::new(),
            comment,
            tags: Default::default(),
        }
    }
}
impl From<Service> for AggregatedService {
    fn from(value: Service) -> Self {
        let Service {
            uuid,
            name,
            version,
            host,
            port,
            comment,
            certainty,
            workspace: _,
        } = value;
        Self {
            uuid,
            name,
            version,
            host: *host.key(),
            port: port.map(|port| *port.key()),
            comment,
            certainty,
            tags: Default::default(),
        }
    }
}
impl From<Domain> for AggregatedDomain {
    fn from(value: Domain) -> Self {
        let Domain {
            uuid,
            domain,
            comment,
            workspace: _,
        } = value;
        Self {
            uuid,
            domain,
            comment,
            tags: Default::default(),
        }
    }
}
