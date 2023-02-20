//! Some utility functions for Uris

use std::str::FromStr;

use actix_web::http::Uri;

/// Checks the address of a leech
///
/// To be valid, the address must contain a scheme, which must be one of "http" or "https"
pub fn check_leech_address(address: &str) -> bool {
    match Uri::from_str(address) {
        Ok(uri) => {
            if let Some(scheme) = uri.scheme_str() {
                !(scheme != "https" && scheme != "http")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
