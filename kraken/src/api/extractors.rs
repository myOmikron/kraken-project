//! Some common [extractors](FromRequest)

use std::ops::Deref;

use actix_toolbox::tb_middleware::Session;
use actix_web::dev::Payload;
use actix_web::http::header::HeaderValue;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult};

/// Extractor for the `Session`'s `"uuid"` field
pub struct SessionUser(pub Uuid);
impl FromRequest for SessionUser {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        std::future::ready(match Session::from_request(req, payload).into_inner() {
            Ok(session) => match session.get("uuid") {
                Ok(Some(uuid)) => Ok(Self(uuid)),
                Ok(None) => Err(Self::Error::from(ApiError::SessionCorrupt)),
                Err(error) => Err(Self::Error::from(ApiError::from(error))),
            },
            Err(error) => Err(error),
        })
    }
}

/// Extractor for the `Authorization: Bearer <token>` header
pub struct BearerToken(
    /// **MUSTN'T contain non ascii characters**
    ///
    /// Use [`HeaderValue::to_str`] to check this requirement
    HeaderValue,
);
impl BearerToken {
    const PREFIX: &'static str = "Bearer ";

    /// Access the contained bearer token
    pub fn as_str(&self) -> &str {
        // SAFETY: as checked in FromRequest the bytes only contain visible ascii characters
        unsafe { std::str::from_utf8_unchecked(&self.0.as_bytes()[Self::PREFIX.len()..]) }
    }
}
impl Deref for BearerToken {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
impl FromRequest for BearerToken {
    type Error = ApiError;
    type Future = Ready<ApiResult<Self>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(header) = req.headers().get("Authorization") {
            if let Ok(string) = header.to_str() {
                if let Some(token) = string.strip_prefix(BearerToken::PREFIX) {
                    if !token.is_empty() {
                        return ready(Ok(Self(header.clone())));
                    }
                }
            }
        }
        ready(Err(ApiError::Unauthenticated))
    }
}
