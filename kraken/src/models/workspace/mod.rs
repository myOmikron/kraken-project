//!
//! This module holds all database related definitions of workspace related structs
//!

use chrono::{DateTime, Utc};
use rorm::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{Attack, OauthClient, User};
mod operations;

/// The permission of a member in a workspace
#[derive(Debug, Copy, Clone, Deserialize, Serialize, ToSchema, DbEnum)]
pub enum WorkspaceMemberPermission {
    /// The member may only read the existing data, but not start any attacks
    ReadOnly,
    /// The member may only start attacks, but not access any data in read or writing form
    ///
    /// This may be expanded for "Workflows"
    AttackOnly,
    /// The member has access reading, writing of data as well as starting attacks
    ReadWrite,
}

/// A workspace member has the privileges to access and modify a workspace of another user
///
/// The owner of the workspace can add and remove members at any time
#[derive(Model)]
pub struct WorkspaceMember {
    /// Unique identifier of the workspace member
    #[rorm(id)]
    pub id: i64,

    /// The user to grant access
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub member: ForeignModel<User>,

    /// The permission the member has in the workspace
    pub permission: WorkspaceMemberPermission,

    /// The workspace to grant access to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time the member was granted access to the workspace
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// Representation of a set of connected data.
///
/// Workspaces are owned by a user and can be shared with others.
#[derive(Model)]
pub struct Workspace {
    /// Unique identifier of the workspace
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the workspace
    #[rorm(max_length = 255)]
    pub name: String,

    /// Optional description of the workspace
    #[rorm(max_length = 65535)]
    pub description: Option<String>,

    /// The user that owns this workspace
    #[rorm(index)]
    pub owner: ForeignModel<User>,

    /// Marks the workspace as archived
    #[rorm(default = false)]
    pub archived: bool,

    /// Timestamp when the workspace was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// The workspace's members
    pub members: BackRef<field!(WorkspaceMember::F.workspace)>,

    /// All attacks started in this workspace
    pub attacks: BackRef<field!(Attack::F.workspace)>,
}

/// An oauth `access_token` for a workspace
#[derive(Model)]
pub struct WorkspaceAccessToken {
    /// Primary key
    #[rorm(id)]
    pub id: i64,

    /// The `access_token`
    #[rorm(max_length = 255)]
    pub token: String,

    /// The user which granted the access
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub user: ForeignModel<User>,

    /// The workspace to grant access on
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The oauth client which received this token
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub application: ForeignModel<OauthClient>,

    /// Date after which the token is invalid
    pub expires_at: DateTime<Utc>,
}

/// This represents an invitation to a workspace
#[derive(Model)]
pub struct WorkspaceInvitation {
    /// Primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace, this invitation is for
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The user this invitation is from
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub from: ForeignModel<User>,

    /// The receiver of the invitation
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub target: ForeignModel<User>,

    /// The point in time this invite was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The target of a [WorkspaceQueryFilter]
#[derive(DbEnum, Debug, ToSchema, Deserialize, Serialize, Copy, Clone)]
#[non_exhaustive]
pub enum FilterTarget {
    /// A global filter
    Global,
    /// A domain filter
    Domain,
    /// A host filter
    Host,
    /// A port filter
    Port,
    /// A service filter
    Service,
}

/// The representation of a single query filter
#[derive(Model)]
pub struct WorkspaceQueryFilter {
    /// The primary key of the query filter
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The name of the filter
    #[rorm(max_length = 255)]
    pub name: String,

    /// The query itself
    #[rorm(max_length = 1024)]
    pub query: String,

    /// The target of the filter
    pub target: FilterTarget,

    /// The workspace this filter is valid for
    pub workspace: ForeignModel<Workspace>,

    /// The point in time this filter was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The combination of different [WorkspaceQueryFilter] that results in a view
#[derive(Model)]
pub struct WorkspaceQueryFilterView {
    /// The primary key of the query filter
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The name of the filter
    #[rorm(max_length = 255)]
    pub name: String,

    /// The global query
    #[rorm(default = "", max_length = 1024)]
    pub global_query: String,

    /// The domain query
    #[rorm(default = "", max_length = 1024)]
    pub domain_query: String,

    /// The host query
    #[rorm(default = "", max_length = 1024)]
    pub host_query: String,

    /// The port query
    #[rorm(default = "", max_length = 1024)]
    pub port_query: String,

    /// The service query
    #[rorm(default = "", max_length = 1024)]
    pub service_query: String,

    /// The workspace this filter is valid for
    pub workspace: ForeignModel<Workspace>,

    /// The point in time this filter was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}
