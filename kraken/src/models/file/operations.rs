use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::ForeignModel;
use rorm::prelude::ForeignModelByField;
use rorm::Patch;
use uuid::Uuid;

use crate::models::MediaFile;
use crate::models::User;
use crate::models::Workspace;

impl MediaFile {
    /// Inserts a new [`MediaFile`]
    ///
    /// **Note:** this function doesn't check for duplicates,
    /// because when a duplicate is encountered,
    /// additional actions on the filesystem need to be taken.
    pub async fn insert(
        executor: impl Executor<'_>,
        uuid: Uuid,
        name: String,
        sha256: String,
        has_thumbnail: bool,
        user: Uuid,
        workspace: Uuid,
    ) -> Result<(), rorm::Error> {
        insert!(executor, MediaFile)
            .return_nothing()
            .single(&MediaFileInsert {
                uuid,
                name,
                sha256,
                has_thumbnail,
                user: Some(ForeignModelByField::Key(user)),
                workspace: Some(ForeignModelByField::Key(workspace)),
            })
            .await
    }
}

#[derive(Patch)]
#[rorm(model = "MediaFile")]
struct MediaFileInsert {
    uuid: Uuid,
    name: String,
    sha256: String,
    has_thumbnail: bool,
    user: Option<ForeignModel<User>>,
    workspace: Option<ForeignModel<Workspace>>,
}
