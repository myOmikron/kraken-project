use std::path::Path;
use std::{fs, io};

use thiserror::Error;

use crate::schema::{ProbeFile, Protocol};

/// Parses a probe file and checks whether it is in the valid directory
pub fn parse_file(
    path: impl AsRef<Path>,
    directory: ProbeFileDirectory,
) -> Result<ProbeFile, ParseError> {
    inner_parse_file(path.as_ref(), directory).map_err(|kind| ParseError {
        file: path.as_ref().display().to_string(),
        kind,
    })
}

/// The kind of probes allows in a directory
#[derive(Debug, Copy, Clone)]
pub enum ProbeFileDirectory {
    /// Allows TCP and TLS
    Tcp,
    /// Allows UDP only
    Udp,
}

/// The error returned by [`parse_file`]
///
/// This is a wrapper which attaches the problematic file to the actual [`ParseErrorKind`].
#[derive(Debug, Error)]
#[error("Failed to parse '{file}': {kind}")]
pub struct ParseError {
    /// The path which has been passed to [`parse_file`]
    pub file: String,

    /// The actual error
    pub kind: ParseErrorKind,
}

/// The actual error stored in [`ParseError`] which is returned by [`parse_file`]
#[derive(Debug, Error)]
pub enum ParseErrorKind {
    /// Failed to read the file
    #[error("{0}")]
    ReadFile(#[source] io::Error),

    /// Failed to parse the file
    #[error("{0}")]
    ParseFile(#[source] serde_yaml::Error),

    /// Some check which is run post parsing on every probe failed
    #[error("Probe {index} is invalid because {error}")]
    CheckProbe {
        /// The probe which failed
        index: usize,
        /// The check's details
        error: CheckProbeError,
    },
}

/// Different checks which are run post parsing on every probe
#[derive(Debug, Error)]
pub enum CheckProbeError {
    /// Multiple payloads are specified
    #[error("more than one `payload_str`, `payload_b64` or `payload_hex` are specified")]
    ConflictingPayload,

    /// Value for `protocol` doesn't match the [`ProbeFileDirectory`] which was passed to [`parse_file`].
    #[error("the protocol {actual:?} can't be declared in a directory for {expected:?}")]
    ProtocolMismatch {
        expected: ProbeFileDirectory,
        actual: Protocol,
    },
}

/// Implementation of [`parse_file`]
///
/// This is a separate function to make the wrapping of a [`ParseErrorKind`] into a [`ParseError`] smoother.
/// As a nice bonus this allows removing the generic in `path`.
fn inner_parse_file(
    path: &Path,
    directory: ProbeFileDirectory,
) -> Result<ProbeFile, ParseErrorKind> {
    let string = fs::read_to_string(path).map_err(ParseErrorKind::ReadFile)?;

    let parsed: ProbeFile = serde_yaml::from_str(&string).map_err(ParseErrorKind::ParseFile)?;

    // Check probe
    for (index, probe) in parsed.probes.iter().enumerate() {
        match (&directory, probe.protocol) {
            (ProbeFileDirectory::Udp, Protocol::Udp) => {}
            (ProbeFileDirectory::Tcp, Protocol::Tcp | Protocol::Tls) => {}
            _ => {
                return Err(ParseErrorKind::CheckProbe {
                    index,
                    error: CheckProbeError::ProtocolMismatch {
                        expected: directory,
                        actual: probe.protocol,
                    },
                });
            }
        }

        if 1 < 0
            + probe.payload_str.is_some() as u8
            + probe.payload_b64.is_some() as u8
            + probe.payload_hex.is_some() as u8
        {
            return Err(ParseErrorKind::CheckProbe {
                index,
                error: CheckProbeError::ConflictingPayload,
            });
        }
    }

    Ok(parsed)
}
