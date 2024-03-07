use rorm::and;
use rorm::db::transaction::Transaction;
use rorm::insert;
use rorm::prelude::ForeignModel;
use rorm::prelude::ForeignModelByField;
use rorm::prelude::*;
use rorm::query;
use rorm::Patch;
use uuid::Uuid;

use crate::models::MediaFile;
use crate::models::User;
use crate::models::Workspace;

impl MediaFile {
    /// Inserts a new [`MediaFile`]
    ///
    /// ## Beware:
    /// - This function checks for duplicates using `name` and `sha256`.
    ///   If it finds one, it won't insert the new file but return the old file's uuid.
    ///   In that case, the new file has to be deleted on disk.
    /// - The insert should only be applied after every filesystem operation finished successfully.
    ///   But the duplicate should be checked as soon as the entire file is available.
    ///   So the insert has to be run in a transaction whose commit is delayed until everything else finished.
    pub async fn get_or_insert(
        tx: &mut Transaction,
        uuid: Uuid,
        name: String,
        sha256: String,
        is_image: bool,
        user: Uuid,
        workspace: Uuid,
    ) -> Result<Uuid, rorm::Error> {
        if let Some((uuid,)) = query!(&mut *tx, (MediaFile::F.uuid,))
            .condition(and![
                MediaFile::F.workspace.equals(workspace),
                MediaFile::F.user.equals(user),
                MediaFile::F.name.equals(&name),
                MediaFile::F.sha256.equals(&sha256)
            ])
            .optional()
            .await?
        {
            Ok(uuid)
        } else {
            insert!(&mut *tx, MediaFile)
                .return_primary_key()
                .single(&MediaFileInsert {
                    uuid,
                    name,
                    sha256,
                    is_image,
                    user: Some(ForeignModelByField::Key(user)),
                    workspace: Some(ForeignModelByField::Key(workspace)),
                })
                .await
        }
    }
}

#[derive(Patch)]
#[rorm(model = "MediaFile")]
struct MediaFileInsert {
    uuid: Uuid,
    name: String,
    sha256: String,
    is_image: bool,
    user: Option<ForeignModel<User>>,
    workspace: Option<ForeignModel<Workspace>>,
}
