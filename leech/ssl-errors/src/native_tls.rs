use std::error::Error;
use std::io;

use thiserror::Error;

/// A more helpful representation of a [`native_tls::Error`]
///
/// Since [`Error::source`] only exposes a `&dyn Error`, this enum has to borrow the error.
#[derive(Debug, Error)]
pub enum NativeTlsError<'a> {
    /// The server didn't respond with valid ssl
    #[error("Not SSL response")]
    NotSsl,

    /// [TLS error alert 112](https://www.rfc-editor.org/rfc/rfc3546#section-4)
    #[error("SNI: Unrecognized name")]
    UnrecognizedName,

    /// The server presented an invalid certificate
    #[error("Invalid certificate")]
    BadCert,

    /// Any error returned from one of openssl's ssl routines
    /// which is not captured by the variant above
    #[error("{0}")]
    OpenSsl(&'a openssl::error::ErrorStack),

    /// Any other error
    /// which is not captured by the variant above
    #[error("{0}")]
    Other(&'a native_tls::Error),
}
impl<'a> NativeTlsError<'a> {
    pub fn new(error: &'a native_tls::Error) -> Self {
        let Some(source) = error.source() else {
            return Self::Other(error);
        };

        // The whole `if` is pointless but serves to help future extensions
        #[allow(clippy::redundant_pattern_matching)]
        if let Some(_) = source.downcast_ref::<io::Error>() {
            // This source is retrievable but not yet found to be of any interest
        }

        let Some(stack) = source.downcast_ref::<openssl::error::ErrorStack>() else {
            return Self::Other(error);
        };

        for error in stack.errors() {
            // All reason constants are taken from https://github.com/openssl/openssl/blob/a7e992847de83aa36be0c399c89db3fb827b0be2/include/openssl/sslerr.h
            const SSL_R_TLSV1_UNRECOGNIZED_NAME: std::ffi::c_int = 1112;
            const SSL_R_PACKET_LENGTH_TOO_LONG: std::ffi::c_int = 198;
            const SSL_R_CERTIFICATE_VERIFY_FAILED: std::ffi::c_int = 134;

            match error.reason_code() {
                SSL_R_TLSV1_UNRECOGNIZED_NAME => return Self::UnrecognizedName,
                SSL_R_PACKET_LENGTH_TOO_LONG => return Self::NotSsl,
                SSL_R_CERTIFICATE_VERIFY_FAILED => return Self::BadCert,
                _ => {}
            }
        }

        return Self::OpenSsl(stack);
    }
}
