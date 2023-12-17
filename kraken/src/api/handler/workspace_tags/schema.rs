use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::models::Color;

/// The request to create a workspace tag
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct CreateWorkspaceTagRequest {
    /// Name of the tag
    pub name: String,
    /// Color of a tag
    pub color: Color,
}

/// The full representation of a full workspace tag
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct FullWorkspaceTag {
    /// The uuid of the workspace tag
    pub uuid: Uuid,
    /// The name of the tag
    #[schema(example = "seems broken")]
    pub name: String,
    /// The color of the tag
    pub color: Color,
    /// The workspace this tag is linked to
    pub workspace: Uuid,
}

/// The response to a request to retrieve all workspace tags
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ListWorkspaceTags {
    /// Workspace tags
    pub workspace_tags: Vec<FullWorkspaceTag>,
}

/// The request to update a workspace tag
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UpdateWorkspaceTag {
    /// Name of the tag
    pub name: Option<String>,
    /// The color of the tag
    pub color: Option<Color>,
}

/// The path of a workspace tag
#[derive(Deserialize, IntoParams)]
pub struct PathWorkspaceTag {
    /// Workspace uuid
    pub w_uuid: Uuid,
    /// Tag uuid
    pub t_uuid: Uuid,
}
