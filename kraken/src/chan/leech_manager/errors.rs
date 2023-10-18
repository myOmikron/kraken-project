use std::fmt;

use thiserror::Error;

use crate::api::handler::ApiError;

/// The error returned by [`LeechClients::random_leech`] which can be converted into [`ApiError::NoLeechAvailable`]
#[derive(Debug, Error)]
pub struct NoLeechAvailable;
impl From<NoLeechAvailable> for ApiError {
    fn from(_: NoLeechAvailable) -> Self {
        ApiError::NoLeechAvailable
    }
}
impl fmt::Display for NoLeechAvailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ApiError::NoLeechAvailable.fmt(f)
    }
}

/// The error returned by [`LeechClients::get_leech`] which can be converted into [`ApiError::InvalidLeech`]
#[derive(Debug, Error)]
pub struct InvalidLeech;
impl From<InvalidLeech> for ApiError {
    fn from(_: InvalidLeech) -> Self {
        ApiError::InvalidLeech
    }
}
impl fmt::Display for InvalidLeech {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ApiError::InvalidLeech.fmt(f)
    }
}
