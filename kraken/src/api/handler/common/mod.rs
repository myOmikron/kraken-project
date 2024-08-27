//! This module contains common types, such as [ApiError], [PathUuid] and the complete
//! error implementation

use serde::Deserialize;
use serde::Deserializer;

#[cfg(feature = "bin")]
pub(crate) mod error;

/// This module contains common schemas such as [`UuidResponse`] or [`ApiErrorResponse`]
pub mod schema;

#[cfg(feature = "bin")]
pub(crate) mod utils;

/// Custom deserializer to enable the distinction of missing keys vs null values in JSON requests
///
/// # Example
/// ```rust
/// use kraken::api::handler::common::de_optional;
///
/// #[derive(serde::Deserialize)]
///  pub(crate) struct UpdateRequest {
///     name: Option<String>,
///
///     // Don't forget the `default`!
///     #[serde(default, deserialize_with = "de_optional")]
///     description: Option<Option<String>>,
/// }
/// ```
pub fn de_optional<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(d)?))
}
