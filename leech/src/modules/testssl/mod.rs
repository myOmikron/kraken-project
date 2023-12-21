//! Holds data and code to interact with `testssl.sh`

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
#[derive(Default, Debug)]
pub struct TestSSLSettings {
    /// The domain to scan
    pub uri: String,

    /// Timeout for TCP handshakes in seconds
    pub connect_timeout: Option<u64>,

    /// Timeout for `openssl` connections in seconds
    pub openssl_timeout: Option<u64>,

    /// Enable ip v6
    pub v6: bool,

    /// Set the `BASICAUTH` header when checking http headers
    pub basic_auth: Option<(String, String)>,

    /// Run against a STARTTLS enabled protocol
    pub starttls: Option<StartTLSProtocol>,

    /// Which scans `testssl.sh` should run
    pub scans: TestSSLScans,
}

/// Protocols to select from when using `--starttls`
#[derive(Debug)]
#[allow(missing_docs)] // The names are pretty unambiguous
pub enum StartTLSProtocol {
    FTP,
    SMTP,
    POP3,
    IMAP,
    XMPP,
    // Telnet, // WIP
    // LDAP,   // Requires `--ssl-native` which is less precise
    // IRC,    // WIP
    LMTP,
    NNTP,
    Postgres,
    MySQL,
}

/// Config option which scans `testssl.sh` should run
#[derive(Default, Debug)]
pub enum TestSSLScans {
    /// Sets no option and uses `testssl.sh`'s default run
    #[default]
    Default,

    /// Sets the `--full` option to run everything
    All,

    /// Select the scans to run manually
    ///
    /// Each field (except `cipher_tests_...`) correspond directly to a section in [`json_pretty::ScanResult`]
    Manual {
        /// Enables [`ScanResult`]'s `protocols` section
        protocols: bool,

        /// Enables [`ScanResult`]'s `grease` section
        grease: bool,

        /// Enables [`ScanResult`]'s `ciphers` section
        ciphers: bool,

        /// Enables [`ScanResult`]'s `pfs` section
        pfs: bool,

        /// Enables [`ScanResult`]'s `server_preferences` section
        server_preferences: bool,

        /// Enables [`ScanResult`]'s `server_defaults` section
        server_defaults: bool,

        /// Enables [`ScanResult`]'s `header_response` section
        header_response: bool,

        /// Enables [`ScanResult`]'s `vulnerabilities` section
        vulnerabilities: bool,

        /// Enables [`ScanResult`]'s `cipher_tests` section
        cipher_tests_all: bool,

        /// Enables [`ScanResult`]'s `cipher_tests` section
        cipher_tests_per_proto: bool,

        /// Enables [`ScanResult`]'s `browser_simulations` section
        browser_simulations: bool,
    },
}

/// Run `testssl.sh` and parse its output
pub async fn run_testssl(settings: TestSSLSettings) -> Result<json_pretty::File, TestSSLError> {
    let TestSSLSettings {
        uri,
        connect_timeout,
        openssl_timeout,
        v6,
        basic_auth,
        starttls,
        scans,
    } = settings;

    let (json_file, json_path) = NamedTempFile::new()?.into_parts();
    let mut json_file = TokioFile::from_std(json_file);

    let cmd = &mut Command::new("testssl-fix");

    // Declare json output
    cmd.arg("--jsonfile-pretty").arg(&json_path);

    // Don't wait for user confirmation when encountering problems
    cmd.arg("--warnings").arg("batch");

    // Add timeouts
    if let Some(seconds) = connect_timeout {
        cmd.arg("--connect-timeout").arg(&seconds.to_string());
    }
    if let Some(seconds) = openssl_timeout {
        cmd.arg("--openssl-timeout").arg(&seconds.to_string());
    }

    // Enable ip v6
    if v6 {
        cmd.arg("-6");
    }

    // Set BASICAUTH header
    if let Some((username, password)) = basic_auth {
        cmd.arg("--basicauth")
            .arg(&format!("{username}:{password}"));
    }

    // Enable STARTTLS
    if let Some(protocol) = starttls {
        cmd.arg("--starttls").arg(match protocol {
            StartTLSProtocol::FTP => "ftp",
            StartTLSProtocol::SMTP => "smtp",
            StartTLSProtocol::POP3 => "pop3",
            StartTLSProtocol::IMAP => "imap",
            StartTLSProtocol::XMPP => "xmpp",
            StartTLSProtocol::LMTP => "lmtp",
            StartTLSProtocol::NNTP => "nntp",
            StartTLSProtocol::Postgres => "postgres",
            StartTLSProtocol::MySQL => "mysql",
        });
    }

    // https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L20277
    match scans {
        TestSSLScans::Default => {}
        TestSSLScans::All => {
            cmd.arg("--full");
        }
        TestSSLScans::Manual {
            protocols,
            grease,
            ciphers,
            pfs,
            server_preferences,
            server_defaults,
            header_response,
            vulnerabilities,
            cipher_tests_all,
            cipher_tests_per_proto,
            browser_simulations,
        } => {
            if protocols {
                cmd.arg("--protocols");
            }
            if grease {
                cmd.arg("--grease");
            }
            if ciphers {
                cmd.arg("--std");
            }
            if pfs {
                cmd.arg("--pfs");
            }
            if server_preferences {
                cmd.arg("--server-preferences");
            }
            if server_defaults {
                cmd.arg("--server-defaults");
            }
            if header_response {
                cmd.arg("--headers");
            }
            if vulnerabilities {
                cmd.arg("--vulnerabilities");
            }
            if cipher_tests_all {
                cmd.arg("--each-cipher");
            }
            if cipher_tests_per_proto {
                cmd.arg("--cipher-per-proto");
            }
            if browser_simulations {
                cmd.arg("--client-simulation");
            }
        }
    }

    let output = cmd.arg(&uri).output().await?;

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
