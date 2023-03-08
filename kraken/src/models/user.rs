use rorm::{BackRef, ForeignModel, Model, Patch};

/// The definition of a user
#[derive(Model, Debug)]
pub struct User {
    /// Primary key of the user, a uuid v4
    #[rorm(primary_key)]
    pub uuid: Vec<u8>,

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
    pub last_login: Option<chrono::NaiveDateTime>,

    /// Creation time of the user
    #[rorm(auto_create_time)]
    pub created_at: chrono::NaiveDateTime,

    /// Backreference to the security keys of a user
    #[rorm(field = "UserKey::F.user")]
    pub user_keys: BackRef<UserKey>,
}

#[derive(Patch)]
#[rorm(model = "User")]
pub(crate) struct UserInsert {
    pub(crate) uuid: Vec<u8>,
    pub(crate) username: String,
    pub(crate) display_name: String,
    pub(crate) password_hash: String,
    pub(crate) admin: bool,
    pub(crate) last_login: Option<chrono::NaiveDateTime>,
}

/// A security key (yubikey, e.g.) of a user
#[derive(Model, Debug)]
pub struct UserKey {
    /// ID of the key
    #[rorm(id)]
    pub id: i64,
    /// Name of the key
    #[rorm(max_length = 255)]
    pub name: String,
    /// Owner of the key
    pub user: ForeignModel<User>,
    /// Key data
    pub key: Vec<u8>,
}

#[derive(Patch)]
#[rorm(model = "UserKey")]
pub(crate) struct UserKeyInsert {
    pub(crate) name: String,
    pub(crate) user: ForeignModel<User>,
    pub(crate) key: Vec<u8>,
}
