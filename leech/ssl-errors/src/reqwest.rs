use std::error::Error;
use std::io;

use log::error;
use log::trace;
use thiserror::Error;

use crate::NativeTlsError;

/// A more helpful representation of a [`reqwest::Error`]
///
/// Since [`Error::source`] only exposes a `&dyn Error`, this enum has to borrow the error.
#[derive(Debug, Error)]
pub enum ReqwestError<'a> {
    /// An error returned from a ssl routine
    #[error("TLS: {0}")]
    Tls(NativeTlsError<'a>),

    /// An io error during connecting
    ///
    /// Probably™ an error with the raw tcp connection i.e. "refused", "not available", etc.
    #[error("TCP: {0}")]
    Tcp(&'a io::Error),

    /// A timeout
    #[error("Connection timed out")]
    TimeOut,

    /// A redirect policy rejected with an error
    #[error("Redirect rejected: {0}")]
    Redirect(&'a (dyn Error + 'static)),

    /// `reqwest`'s API was used "badly"
    ///
    /// This entails errors returned from builders
    /// as well as errors from unrecommended parts of the `reqwest`'s API.
    ///
    /// "Unrecommended" being relative to this crate's intended use cause.
    ///
    /// For example, if you just want to request a specific json and abort in the case of an error,
    /// you don't need this enum.
    ///
    /// On the other hand, if you want the case of your request's error as detailed as possible,
    /// you shouldn't pollute it with additional error sources like
    /// [`Response::error_from_status`] or [`Response::json`].
    ///
    /// [`Response::error_from_status`]: reqwest::Response::error_for_status
    /// [`Response::json`]: reqwest::Response::json
    #[error("Blame your programmer: {0}")]
    ProgrammerFault(&'a reqwest::Error),

    /// Any other error
    #[error("{0}")]
    Other(&'a reqwest::Error),
}
impl<'a> ReqwestError<'a> {
    /// Inspects a [`reqwest::Error`] to get more details about its cause
    pub fn new(value: &'a reqwest::Error) -> Self {
        if value.is_builder() {
            error!("The builder should be used correctly");
            return Self::ProgrammerFault(value);
        } else if value.is_status() {
            error!("Response::error_for_status shouldn't be used");
            return Self::ProgrammerFault(value);
        } else if value.is_decode() {
            error!("The response's body shouldn't be decoded by reqwest");
            return Self::ProgrammerFault(value);
        } else if value.is_timeout() {
            return Self::TimeOut;
        } else if value.is_redirect() {
            return Self::Redirect(value.source().unwrap_or(&MissingRedirectError));
        } else if let Some(error) = find_cause::<native_tls::Error>(value) {
            return Self::Tls(NativeTlsError::new(error));
        } else if let Some(hyper) = find_cause::<hyper_util::client::legacy::Error>(value) {
            trace!("reqwest::Error originated from a hyper_util::Error");
            if hyper.is_connect() {
                trace!("hyper_util::Error::is_connect() == true");
                if let Some(io) = find_cause::<io::Error>(hyper) {
                    return Self::Tcp(io);
                }
            }
        }
        Self::Other(value)
    }
}

/// Fallback error
/// used in the [`ReqwestError::Redirect`] variant
/// if `reqwest::Error::source` returned `None`.
#[derive(Debug, Error)]
#[error("No reason was provided")]
pub struct MissingRedirectError;

/// Iterates through the chain of errors created through [`Error::source`]
/// and search for specific error type `E`.
fn find_cause<E: Error + 'static>(error: &impl Error) -> Option<&E> {
    let mut source = error.source();

    while let Some(err) = source {
        if let Some(e) = err.downcast_ref::<E>() {
            return Some(e);
        }
        source = err.source();
    }

    None
}
