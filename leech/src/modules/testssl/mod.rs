//! Holds data and code to interact with `testssl.sh`

use std::borrow::Cow;
use std::io;

use log::error;
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

mod json;
mod json_pretty;

/// The settings of a `testssl.sh` invocation
pub struct TestSSLSettings {
    /// Optional alternative path to the `testssl.sh`
    pub binary_path: Option<Cow<'static, str>>,

    /// The domain to scan
    pub uri: String,
}
/// Run `testssl.sh` and parse its output
pub async fn run_testssl(settings: TestSSLSettings) -> Result<json_pretty::File, TestSSLError> {
    let (json_file, json_path) = NamedTempFile::new()?.into_parts();
    let mut json_file = TokioFile::from_std(json_file);

    let output = Command::new(settings.binary_path.as_deref().unwrap_or("testssl"))
        .arg("--jsonfile-pretty")
        .arg(&json_path)
        .arg(&settings.uri)
        .output()
        .await?;

    if output.status.success() {
        let mut json_output = Vec::new();
        json_file.read_to_end(&mut json_output).await?;
        let json = serde_json::from_slice(&json_output)?;
        Ok(json)
    } else {
        error!(
            "testssl.sh exited with [{}]",
            output
                .status
                .code()
                .expect("None should have send this process a signal")
        );
        Err(TestSSLError::NonZeroExitStatus)
    }
}

/// Error type produced by [`run_testssl`]
#[derive(Error, Debug)]
pub enum TestSSLError {
    /// An io error occurred while running the subprocess or interacting with its output file
    #[error("Io error: {}", .0)]
    Io(#[from] io::Error),

    /// Failed to parse the json output
    #[error("Json error: {}", .0)]
    Json(#[from] serde_json::Error),

    /// The `testssl` process exited with a non zero status
    #[error("testssl exited with a non zero status")]
    NonZeroExitStatus,
}
