//! Some utility functions for Uris

use url::Url;

/// Checks the address of a leech
///
/// To be valid, the address must contain a scheme, which must be one of "http" or "https"
pub fn check_leech_address(address: &Url) -> bool {
    !(address.scheme() != "https" && address.scheme() != "http")
}
