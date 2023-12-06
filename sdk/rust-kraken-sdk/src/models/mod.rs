use serde::{Deserialize, Deserializer};

mod api_error;
pub use self::api_error::*;
mod data_export;
pub use self::data_export::*;
mod oauth;
pub use self::oauth::*;

/// Deserialize potentially non-existing optional value
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}
