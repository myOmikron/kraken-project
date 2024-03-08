use rorm::and;
use rorm::db::Executor;
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

/// Transparent wrapper around an [`Executor`] to hint the user to defer the actual execution.
pub struct DeferCommit<E>(pub E);

impl MediaFile {
    /// Checks whether a file exists
    pub async fn exists(executor: impl Executor<'_>, uuid: Uuid) -> Result<bool, rorm::Error> {
        Ok(query!(executor, (MediaFile::F.uuid,))
            .condition(MediaFile::F.uuid.equals(uuid))
            .optional()
            .await?
            .is_some())
    }

    /// Checks whether a file exists and is an image
    pub async fn is_image(executor: impl Executor<'_>, uuid: Uuid) -> Result<bool, rorm::Error> {
        Ok(query!(executor, (MediaFile::F.is_image,))
            .condition(MediaFile::F.uuid.equals(uuid))
            .optional()
            .await?
            .map(|(is_image,)| is_image)
            .unwrap_or(false))
    }

    /// Inserts a new [`MediaFile`]
    ///
    /// ## Beware:
    /// - This function checks for duplicates using `name` and `sha256`.
    ///   If it finds one, it won't insert the new file but return the old file's uuid.
    ///   In that case, the new file has to be deleted on disk.
    /// - The insert should only be applied after every filesystem operation finished successfully.
    ///   But the duplicate should be checked as soon as the entire file is available.
    ///
    ///   => You can achieve this by using a transaction whose commit is delayed until everything else finished.
    pub async fn get_or_insert(
        DeferCommit(executor): DeferCommit<impl Executor<'_>>,
        uuid: Uuid,
        name: String,
        sha256: String,
        is_image: bool,
        user: Uuid,
        workspace: Uuid,
    ) -> Result<Uuid, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;

        let uuid = if let Some((uuid,)) = query!(guard.get_transaction(), (MediaFile::F.uuid,))
            .condition(and![
                MediaFile::F.workspace.equals(workspace),
                MediaFile::F.user.equals(user),
                MediaFile::F.name.equals(&name),
                MediaFile::F.sha256.equals(&sha256)
            ])
            .optional()
            .await?
        {
            uuid
        } else {
            insert!(guard.get_transaction(), MediaFile)
                .return_primary_key()
                .single(&MediaFileInsert {
                    uuid,
                    name,
                    sha256,
                    is_image,
                    user: Some(ForeignModelByField::Key(user)),
                    workspace: Some(ForeignModelByField::Key(workspace)),
                })
                .await?
        };

        guard.commit().await?;
        Ok(uuid)
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
