use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;

/// Query parameters when uploading a file which provide the file's name
#[derive(Serialize, Deserialize, IntoParams, Debug, Clone)]
pub struct UploadQuery {
    /// The file's name
    pub filename: String,
}

/// The path parameter of a file
#[derive(Serialize, Deserialize, IntoParams, Debug, Copy, Clone)]
pub struct PathFile {
    /// The workspace's uuid
    pub w_uuid: Uuid,
    /// The file's uuid
    pub f_uuid: Uuid,
}

/// Query parameters to filter which files to retrieve
#[derive(Serialize, Deserialize, IntoParams, Debug, Copy, Clone)]
pub struct GetAllFilesQuery {
    /// Only get a single workspace's files
    pub workspace: Option<Uuid>,

    /// Only get a single user's files
    pub user: Option<Uuid>,
}

/// Metadata stored about a file uploaded to kraken
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullFile {
    /// The file's uuid
    pub uuid: Uuid,

    /// The file's name
    pub name: String,

    /// The file's sha256
    pub sha256: String,

    /// Is this file an image?
    ///
    /// This flag indicates whether there exists a thumbnail file
    /// and if it can be used where an image is required.
    pub is_image: bool,

    /// The user who uploaded the file
    pub user: SimpleUser,

    /// The workspace the file was uploaded to
    pub workspace: SimpleWorkspace,

    /// Time of first upload
    pub uploaded_at: DateTime<Utc>,
}
