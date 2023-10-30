use rorm::prelude::*;
use uuid::Uuid;

use super::Settings;
use crate::models::UserPermission;

/// The patch to insert settings
#[derive(Patch)]
#[rorm(model = "Settings")]
pub(crate) struct SettingsInsert {
    /// The primary key of the settings
    pub uuid: Uuid,

    /// Require mfa for local users
    pub mfa_required: bool,

    /// The default permission a user from oidc is set to
    pub oidc_initial_permission_level: UserPermission,

    /// The email for the dehashed account
    pub dehashed_email: Option<String>,

    /// The api key for the dehashed account
    pub dehashed_api_key: Option<String>,
}
