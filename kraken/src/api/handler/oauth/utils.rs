use actix_web::body::BoxBody;
use actix_web::web::Redirect;
use actix_web::{HttpResponse, ResponseError};
use log::error;
use serde::Serialize;
use url::Url;

use crate::api::handler::common::error::ApiError;
use crate::modules::oauth::schemas::{TokenError, TokenErrorType};

pub(crate) fn build_redirect(
    url: &str,
    query: impl Serialize + std::fmt::Debug,
) -> Result<Redirect, ApiError> {
    let Ok(mut url) = Url::parse(url) else {
        error!("Failed to parse url: {url}");
        return Err(ApiError::InternalServerError);
    };

    {
        let mut pairs = url.query_pairs_mut();
        let serializer = serde_urlencoded::Serializer::new(&mut pairs);
        if query.serialize(serializer).is_err() {
            error!("Failed to serialize url query: {query:?}");
            return Err(ApiError::InternalServerError);
        }
    }

    Ok(Redirect::to(url.to_string()))
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_description {
            Some(desc) => write!(f, "{:?}: {desc}", self.error),
            None => write!(f, "{:?}", self.error),
        }
    }
}

impl From<rorm::Error> for TokenError {
    fn from(err: rorm::Error) -> Self {
        error!("Database error in `/token` endpoint: {err}");
        Self {
            error: TokenErrorType::ServerError,
            error_description: Some("An internal server error occured"),
        }
    }
}

impl ResponseError for TokenError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        (match self.error {
            TokenErrorType::ServerError => HttpResponse::InternalServerError(),
            TokenErrorType::InvalidClient => HttpResponse::Unauthorized(),
            _ => HttpResponse::BadRequest(),
        })
        .json(self)
    }
}
