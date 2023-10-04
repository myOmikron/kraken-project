use chrono::{DateTime, Utc};
use rorm::fields::types::Json;
use rorm::prelude::*;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

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

    /// Password hash of the user
    #[rorm(max_length = 1024)]
    pub password_hash: String,

    /// Flag whether the user is has administrative privileges
    pub admin: bool,

    /// Last time the user has logged in
    pub last_login: Option<DateTime<Utc>>,

    /// Creation time of the user
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,

    /// Backreference to the security keys of a user
    pub user_keys: BackRef<field!(UserKey::F.user)>,
}

#[derive(Patch)]
#[rorm(model = "User")]
pub(crate) struct UserInsert {
    pub(crate) uuid: Uuid,
    pub(crate) username: String,
    pub(crate) display_name: String,
    pub(crate) password_hash: String,
    pub(crate) admin: bool,
    pub(crate) last_login: Option<DateTime<Utc>>,
}

/// A security key (yubikey, e.g.) of a user
#[derive(Model)]
pub struct UserKey {
    /// Uuid of the key
    #[rorm(primary_key)]
    pub uuid: Uuid,
    /// Name of the key
    #[rorm(max_length = 255)]
    pub name: String,
    /// Owner of the key
    pub user: ForeignModel<User>,
    /// Key data
    pub key: Json<Passkey>,
}

#[derive(Patch)]
#[rorm(model = "UserKey")]
pub(crate) struct UserKeyInsert {
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) user: ForeignModel<User>,
    pub(crate) key: Json<Passkey>,
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
