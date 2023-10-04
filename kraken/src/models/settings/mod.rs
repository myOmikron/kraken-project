use chrono::{DateTime, Utc};
use rorm::prelude::*;
use uuid::Uuid;

/// The settings of kraken
#[derive(Model, Debug, Clone)]
pub struct Settings {
    /// The primary key of the settings
    #[rorm(primary_key)]
    pub uuid: Uuid,

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

/// The patch to insert settings
#[derive(Patch)]
#[rorm(model = "Settings")]
pub struct SettingsInsert {
    /// The primary key of the settings
    pub uuid: Uuid,
    /// The email for the dehashed account
    pub dehashed_email: Option<String>,
    /// The api key for the dehashed account
    pub dehashed_api_key: Option<String>,
}
