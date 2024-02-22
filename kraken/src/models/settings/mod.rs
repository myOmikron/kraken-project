use chrono::DateTime;
use chrono::Utc;
use rorm::prelude::*;
use uuid::Uuid;

#[cfg(feature = "bin")]
pub(crate) use self::operations::*;
use crate::models::UserPermission;
mod operations;

/// The settings of kraken
#[derive(Model, Debug, Clone)]
pub struct Settings {
    /// The primary key of the settings
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Require mfa for local users
    pub mfa_required: bool,

    /// The default permission a user from oidc is set to
    pub oidc_initial_permission_level: UserPermission,

    /// The email for the dehashed account
    #[rorm(max_length = 1024)]
    pub dehashed_email: Option<String>,

    /// The api key for the dehashed account
    #[rorm(max_length = 1024)]
    pub dehashed_api_key: Option<String>,

    /// The point in time the settings were created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}
