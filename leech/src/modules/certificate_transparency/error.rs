//! The error that can be returned by certificate transparency

use std::fmt::{Display, Formatter};

/// The error that can be returned by certificate transparency
#[derive(Debug)]
pub enum CertificateTransparencyError {
    /// Couldn't fetch data from crt.sh
    CouldntFetchData,
    /// Reqwest error
    Reqwest(reqwest::Error),
    /// Couldn't deserialize data from json
    DeserializeError(serde_json::Error),
}

impl Display for CertificateTransparencyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CertificateTransparencyError::CouldntFetchData => write!(f, "Couldn't fetch the data"),
            CertificateTransparencyError::Reqwest(err) => write!(f, "Reqwest error: {err}"),
            CertificateTransparencyError::DeserializeError(err) => {
                write!(f, "Could not deserialize: {err}")
            }
        }
    }
}

impl From<CertificateTransparencyError> for String {
    fn from(value: CertificateTransparencyError) -> Self {
        value.to_string()
    }
}

impl From<reqwest::Error> for CertificateTransparencyError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
