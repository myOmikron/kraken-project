use std::fmt::Write;
use std::io;
use std::path::Path;

use actix_web::web::Payload;
use futures::TryStreamExt;
use log::error;
use sha2::Digest;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn stream_into_file<D: Digest>(
    path: &Path,
    mut body: Payload,
) -> Result<(DeleteFile, String), StreamIntoFileError> {
    use StreamIntoFileError::Actix;
    use StreamIntoFileError::FileClose;
    use StreamIntoFileError::FileCreate;
    use StreamIntoFileError::FileWrite;

    let mut file = File::create(path).await.map_err(FileCreate)?;
    let mut hashed = D::new();
    let guard = DeleteFile::new(path);

    while let Some(chunk) = body.try_next().await.map_err(Actix)? {
        file.write_all(&chunk).await.map_err(FileWrite)?;
        hashed.update(&chunk);
    }
    file.shutdown().await.map_err(FileClose)?;
    let raw_hash = hashed.finalize();

    let mut hash = String::with_capacity(raw_hash.len() * 2);
    for byte in raw_hash {
        write!(&mut hash, "{byte:x}").unwrap();
    }

    Ok((guard, hash))
}

pub struct DeleteFile<'p>(Option<&'p Path>);
impl<'p> DeleteFile<'p> {
    pub fn new(path: &'p Path) -> Self {
        Self(Some(path))
    }

    pub fn dont(mut self) {
        self.0 = None;
    }
}
impl<'p> Drop for DeleteFile<'p> {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let path = path.to_path_buf();
            tokio::task::spawn(async move {
                if let Err(err) = fs::remove_file(&path).await {
                    error!("Failed to delete {} in drop: {err}", path.display());
                }
            });
        }
    }
}

#[derive(Debug)]
pub enum StreamIntoFileError {
    Actix(actix_web::error::PayloadError),
    FileCreate(io::Error),
    FileWrite(io::Error),
    FileClose(io::Error),
}
