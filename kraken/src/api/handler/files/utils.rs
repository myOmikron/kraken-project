use std::fmt::Write;
use std::io;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;

use actix_web::http::header::ContentLength;
use actix_web::web::Payload;
use bytes::Bytes;
use futures::TryStreamExt;
use image::guess_format;
use image::ImageFormat;
use log::error;
use sha2::Digest;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::config::VAR_DIR;

/// The maximum size accepted by the file endpoints
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MiB

pub fn media_file_path(uuid: Uuid) -> PathBuf {
    Path::new(VAR_DIR).join("media").join(uuid.to_string())
}

pub fn media_thumbnail_path(uuid: Uuid) -> PathBuf {
    Path::new(VAR_DIR)
        .join("media")
        .join("thumbnails")
        .join(uuid.to_string())
}

pub fn valid_image(file_start: &[u8]) -> Option<ImageFormat> {
    match guess_format(file_start) {
        Ok(format) if matches!(format, ImageFormat::Png | ImageFormat::Jpeg) => Some(format),
        _ => None,
    }
}

pub async fn stream_into_file_with_magic<D: Digest>(
    path: &Path,
    content_length: ContentLength,
    body: Payload,
) -> Result<Option<((DeleteFile, String), ImageFormat)>, StreamIntoFileError> {
    let mut magic = [0u8; 8];
    let mut magic_cursor = Some(Cursor::new(&mut magic[..]));
    let option = stream_into_file::<D>(path, content_length, body, move |chunk| {
        if let Some(cursor) = magic_cursor.as_mut() {
            if io::Write::write_all(cursor, chunk).is_err() {
                valid_image(cursor.get_ref()).ok_or(())?;
            }
        }
        Ok(())
    })
    .await?;
    Ok(option.zip(valid_image(&magic)))
}

pub async fn stream_into_file<D: Digest>(
    path: &Path,
    content_length: ContentLength,
    mut body: Payload,
    mut hook: impl FnMut(&Bytes) -> Result<(), ()>,
) -> Result<Option<(DeleteFile, String)>, StreamIntoFileError> {
    use StreamIntoFileError::Actix;
    use StreamIntoFileError::FileClose;
    use StreamIntoFileError::FileCreate;
    use StreamIntoFileError::FileWrite;

    if content_length.0 > MAX_FILE_SIZE {
        return Err(StreamIntoFileError::BodyTooLarge);
    }

    let mut file = File::create(path).await.map_err(FileCreate)?;
    let mut hashed = D::new();
    let guard = DeleteFile::new(path);

    let mut file_size = 0;

    while let Some(chunk) = body.try_next().await.map_err(Actix)? {
        if hook(&chunk).is_err() {
            return Ok(None);
        }

        // The check above should be sufficient, but just to be sure
        file_size += chunk.len();
        if file_size > MAX_FILE_SIZE {
            return Err(StreamIntoFileError::BodyTooLarge);
        }

        file.write_all(&chunk).await.map_err(FileWrite)?;
        hashed.update(&chunk);
    }
    file.shutdown().await.map_err(FileClose)?;
    let raw_hash = hashed.finalize();

    let mut hash = String::with_capacity(raw_hash.len() * 2);
    for byte in raw_hash {
        write!(&mut hash, "{byte:x}").unwrap();
    }

    Ok(Some((guard, hash)))
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
    BodyTooLarge,
    Actix(actix_web::error::PayloadError),
    FileCreate(io::Error),
    FileWrite(io::Error),
    FileClose(io::Error),
}

impl From<StreamIntoFileError> for ApiError {
    fn from(value: StreamIntoFileError) -> Self {
        match value {
            StreamIntoFileError::BodyTooLarge => {
                ApiError::PayloadOverflow("The uploaded file is too large".to_string())
            }
            StreamIntoFileError::Actix(err) => ApiError::PayloadError(err),
            StreamIntoFileError::FileCreate(err)
            | StreamIntoFileError::FileWrite(err)
            | StreamIntoFileError::FileClose(err) => {
                error!("Failed to write uploaded file to tmp: {err}");
                ApiError::InternalServerError
            }
        }
    }
}
