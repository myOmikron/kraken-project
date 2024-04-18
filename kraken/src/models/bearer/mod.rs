mod operations;

use rorm::Model;
use uuid::Uuid;

/// A token that grants access to the Service API of kraken
#[derive(Model)]
pub struct BearerToken {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the application this bearer is intended for
    #[rorm(max_length = 255, unique)]
    pub name: String,

    /// The token that grants access
    ///
    /// This must be randomly generated
    #[rorm(max_length = 64, unique)]
    pub token: String,
}
