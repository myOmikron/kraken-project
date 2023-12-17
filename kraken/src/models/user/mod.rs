use chrono::{DateTime, Utc};
use rorm::fields::types::Json;
use rorm::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

#[cfg(feature = "bin")]
mod operations;

/// The permission of a user
#[derive(DbEnum, Copy, Clone, ToSchema, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub enum UserPermission {
    /// The user can not create workspaces or start any attacks.
    /// The user can only be invited to existing workspaces to retrieve the data of the workspace
    ReadOnly,
    /// Default permission for users
    Default,
    /// Administrative access
    Admin,
}

/// A user imported from oidc
#[derive(Model)]
pub struct OidcUser {
    /// Primary key of the user, a uuid v4
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The relation to the generic user
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub user: ForeignModel<User>,
}

/// A user that exists in the local database
#[derive(Model)]
pub struct LocalUser {
    /// Primary key of the user, a uuid v4
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The relation to the generic user
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub user: ForeignModel<User>,

    /// Password hash of the user
    #[rorm(max_length = 1024)]
    pub password_hash: String,

    /// Backreference to the security keys of a user
    pub user_keys: BackRef<field!(LocalUserKey::F.user)>,
}

/// A security key (yubikey, e.g.) of a local user
#[derive(Model)]
pub struct LocalUserKey {
    /// Uuid of the key
    #[rorm(primary_key)]
    pub uuid: Uuid,
    /// Name of the key
    #[rorm(max_length = 255)]
    pub name: String,
    /// Owner of the key
    pub user: ForeignModel<LocalUser>,
    /// Key data
    pub key: Json<Passkey>,
}

/// The definition of a user
#[derive(Model)]
pub struct User {
    /// Primary key of the user, a uuid v4
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The username is used for login
    #[rorm(max_length = 255, unique, index)]
    pub username: String,

    /// This name is displayed to other users
    #[rorm(max_length = 255)]
    pub display_name: String,

    /// The permission of the user account
    pub permission: UserPermission,

    /// Last time the user has logged in
    pub last_login: Option<DateTime<Utc>>,

    /// Creation time of the user
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// Api key which grants a leech access when used as cli instead of a service
#[derive(Model)]
pub struct LeechApiKey {
    /// Uuid of the key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Owner of the api key
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub user: ForeignModel<User>,

    /// The api key
    #[rorm(max_length = 255, unique)]
    pub key: String,

    /// A descriptive name helping the user to identify the key
    #[rorm(max_length = 255)]
    pub name: String,
}
