//! Code for dealing with file uploads outside the api handler

use std::fs::Permissions;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use log::error;
use rorm::conditions::Column;
use rorm::conditions::Unary;
use rorm::conditions::UnaryOperator;
use rorm::delete;
use rorm::prelude::*;
use rorm::query;
use tokio::fs;
use tokio::time::sleep;

use crate::chan::global::GLOBAL;
use crate::config::VAR_DIR;
use crate::models::MediaFile;

/// Spawn a task which deletes files without a workspace each 10 minutes.
pub async fn start_file_cleanup() -> io::Result<()> {
    let media_dir = Path::new(VAR_DIR).join("media");
    let thumbnails_dir = media_dir.join("thumbnails");

    for dir in [&media_dir, &thumbnails_dir] {
        if !dir.exists() {
            fs::create_dir(dir).await?;
            fs::set_permissions(&dir, Permissions::from_mode(0o750)).await?;
        }
    }

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60 * 10)).await;

            // I'm the only cleanup job and files without workspace can't get reassigned a new one
            // So no transaction is required

            // On the other side, if a database error would occur on the let's say third file,
            // a transaction would undo the deletion of the first two files.
            // But their filesystem files would already be gone.
            // So a transaction shouldn't be used

            let Ok(to_delete) = query!(&GLOBAL.db, MediaFile)
                .condition(Unary {
                    operator: UnaryOperator::IsNull,
                    fst_arg: Column(MediaFile::F.workspace),
                })
                .all()
                .await
                .inspect_err(|error| error!("Failed to query for orphaned files: {error}"))
            else {
                continue;
            };
            for file in to_delete {
                let uuid = file.uuid.to_string();
                if let Err(error) = fs::remove_file(media_dir.join(&uuid)).await {
                    error!("Failed to delete file {uuid}: {error}");
                    continue;
                }
                if file.is_image {
                    if let Err(error) = fs::remove_file(thumbnails_dir.join(&uuid)).await {
                        error!("Failed to delete thumbnail {uuid}, but the file has already been deleted: {error}");
                    }
                }
                if let Err(error) = delete!(&GLOBAL.db, MediaFile).single(&file).await {
                    error!(
                        "Failed to delete file model {uuid}, but the file has already been deleted: {error}"
                    );
                }
            }
        }
    });

    Ok(())
}
