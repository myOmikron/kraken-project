use chrono::DateTime;
use chrono::Utc;
use rorm::Patch;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::User;
use crate::models::UserPermission;

/// The request to create a user
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct CreateUserRequest {
    /// The username
    #[schema(example = "user123")]
    pub username: String,
    /// The displayname
    #[schema(example = "Anon")]
    pub display_name: String,
    /// The password that should be set
    #[schema(example = "super-secure-password")]
    pub password: String,
    /// The permissions that the user should have
    pub permission: UserPermission,
}

/// The request to set a new password for a user
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct SetPasswordRequest {
    /// The current password
    #[schema(example = "super-secure-password")]
    pub current_password: String,
    /// The new password
    #[schema(example = "ultra-secure-password!1!1!")]
    pub new_password: String,
}
/// The request to update the own user
///
/// At least one of the options must be set
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct UpdateMeRequest {
    /// The username
    #[schema(example = "cyber-user-123")]
    pub username: Option<String>,
    /// The displayname
    #[schema(example = "Cyberhacker")]
    pub display_name: Option<String>,
}

/// A single user representation
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct FullUser {
    /// The uuid of the user
    pub uuid: Uuid,
    /// The username of the user
    #[schema(example = "user123")]
    pub username: String,
    /// The displayname of the user
    #[schema(example = "Anon")]
    pub display_name: String,
    /// The permissions that the user has
    pub permission: UserPermission,
    /// The point in time this user was created
    pub created_at: DateTime<Utc>,
    /// The last point in time when the user has logged in
    pub last_login: Option<DateTime<Utc>>,
}

/// The response of all users
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct ListFullUsers {
    /// The list of full users
    pub users: Vec<FullUser>,
}

/// This struct holds the user information.
///
/// Note that `username` is unique, but as it is changeable,
/// identify the user by its `uuid`
#[derive(Deserialize, Serialize, ToSchema, Patch, Debug, Clone)]
#[rorm(model = "User")]
pub struct SimpleUser {
    /// The uuid of the user
    pub uuid: Uuid,
    /// The username of the user
    pub username: String,
    /// The displayname of the user
    pub display_name: String,
}

/// The response with all users
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct ListUsers {
    /// List of users
    pub users: Vec<SimpleUser>,
}
