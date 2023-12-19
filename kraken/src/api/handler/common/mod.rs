//! This module contains common types, such as [ApiError], [PathUuid] and the complete
//! error implementation

use serde::{Deserialize, Deserializer};

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
/// #[derive(Deserialize)]
///  pub(crate) struct UpdateRequest {
///     name: Option<String>,
///
///     #[serde(default)]
///     #[serde(deserialize_with = "crate::api::handler::de_optional")]
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
