use rorm::Model;

/// The definition of a user
#[derive(Model, Debug)]
pub struct User {
    /// Primary key of the user, a uuid v4
    #[rorm(primary_key)]
    pub uuid: Vec<u8>,

    /// The username is used for login
    #[rorm(max_length = 255, unique)]
    pub username: String,

    /// Password hash of the user
    #[rorm(max_length = 1024)]
    pub password_hash: String,

    /// Last time the user has logged in
    pub last_login: Option<chrono::NaiveDateTime>,

    /// Creation time of the user
    #[rorm(auto_create_time)]
    pub created_at: chrono::NaiveDateTime,
}
