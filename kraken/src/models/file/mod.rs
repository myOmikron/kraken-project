mod operations;

use chrono::DateTime;
use chrono::Utc;
pub use operations::DeferCommit;
use rorm::prelude::*;
use uuid::Uuid;

use crate::models::User;
use crate::models::Workspace;

/// A file uploaded by a user which is stored on disk for later downloads
///
/// The existence of this model should guarantee the existence of the actual file and vice versa.
/// That's why `user` and `workspace` are `Option`, because this model can only be deleted when the file is.
#[derive(Model)]
pub struct MediaFile {
    /// The primary key
    ///
    /// This is the name the actual file is stored as on disk.
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The file's name shown to the user
    ///
    /// On disk the actual file is stored under its `uuid`.
    #[rorm(max_length = 255)]
    pub name: String,

    /// Sha256 hash
    #[rorm(max_length = 64)]
    pub sha256: String,

    /// Is this file an image?
    ///
    /// This flag indicates whether there exists a thumbnail file
    /// and if it can be used where an image is required.
    #[rorm(default = false)]
    pub is_image: bool,

    /// The user who uploaded the file
    // SetNull because the model should only be deleted when the file is
    #[rorm(on_update = "Cascade", on_delete = "SetNull")]
    pub user: Option<ForeignModel<User>>,

    /// The workspace the file was uploaded to
    // SetNull because the model should only be deleted when the file is
    #[rorm(on_update = "Cascade", on_delete = "SetNull")]
    pub workspace: Option<ForeignModel<Workspace>>,

    /// Time the file was uploaded
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}
