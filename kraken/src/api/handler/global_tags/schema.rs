use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::common::schema::Color;

/// The request to create a global tag
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct CreateGlobalTagRequest {
    /// Name of the tag
    pub name: String,
    /// Color of a tag
    pub color: Color,
}

/// The request to update a global tag
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct UpdateGlobalTag {
    /// Name of the global tag
    pub name: Option<String>,
    /// Color of the global tag
    pub color: Option<Color>,
}

/// The full representation of a full
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullGlobalTag {
    /// The uuid of the tag
    pub uuid: Uuid,
    /// The name of the tag
    pub name: String,
    /// The color of the tag
    pub color: Color,
}

/// The response to a request to retrieve all global tags
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ListGlobalTags {
    /// List of global tags
    pub global_tags: Vec<FullGlobalTag>,
}
