//! The schema for requests and responses regarding the workspaces are defined in this module

use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Create a new workspace and set the owner for it
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateWorkspaceRequest {
    /// The name of the workspace
    pub name: String,
    /// The description of the workspace
    pub description: Option<String>,
    /// The uuid of the owner of the workspace
    pub owner: Uuid,
}
