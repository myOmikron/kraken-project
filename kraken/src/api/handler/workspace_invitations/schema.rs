use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;

/// The full representation of an invitation to a workspace
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct FullWorkspaceInvitation {
    /// The uuid of the invitation
    pub uuid: Uuid,
    /// The workspace the invitation is
    pub workspace: SimpleWorkspace,
    /// The user that has issued the invitation
    pub from: SimpleUser,
    /// The user that was invited
    pub target: SimpleUser,
}

/// A list of invitations to workspaces
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct WorkspaceInvitationList {
    /// All invitations of the current user
    pub invitations: Vec<FullWorkspaceInvitation>,
}
