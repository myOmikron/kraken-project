use rorm::prelude::*;
use uuid::Uuid;

use super::Settings;

/// The patch to insert settings
#[derive(Patch)]
#[rorm(model = "Settings")]
pub(crate) struct SettingsInsert {
    /// The primary key of the settings
    pub(crate) uuid: Uuid,
    /// The email for the dehashed account
    pub(crate) dehashed_email: Option<String>,
    /// The api key for the dehashed account
    pub(crate) dehashed_api_key: Option<String>,
}
