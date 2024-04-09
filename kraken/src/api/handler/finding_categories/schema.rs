use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::common::schema::Color;

/// The request to create a finding category
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFindingCategoryRequest {
    /// The category's name
    pub name: String,

    /// The category's color
    pub color: Color,
}

/// The request to update a finding category
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFindingCategoryRequest {
    /// The category's name
    pub name: Option<String>,

    /// The category's color
    pub color: Option<Color>,
}

/// The response to a request to retrieve all finding categories
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListFindingCategories {
    /// List of finding categories
    pub categories: Vec<SimpleFindingCategory>,
}

/// A category findings and finding definitions can be categorized by
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimpleFindingCategory {
    /// The category's uuid
    pub uuid: Uuid,

    /// The category's name
    pub name: String,

    /// The category's color
    pub color: Color,
}
