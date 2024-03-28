use actix_web::body::BoxBody;
use actix_web::web::Redirect;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use log::error;
use serde::Serialize;
use swaggapi::as_responses::simple_responses;
use swaggapi::as_responses::AsResponses;
use swaggapi::as_responses::SimpleResponse;
use swaggapi::internals::SchemaGenerator;
use swaggapi::re_exports::openapiv3::MediaType;
use swaggapi::re_exports::openapiv3::Responses;
use swaggapi::re_exports::openapiv3::StatusCode;
use url::Url;

use crate::api::handler::common::error::ApiError;
use crate::modules::oauth::schemas::TokenError;
use crate::modules::oauth::schemas::TokenErrorType;

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

impl AsResponses for TokenError {
    fn responses(gen: &mut SchemaGenerator) -> Responses {
        let media_type = Some(MediaType {
            schema: Some(gen.generate::<Self>()),
            ..Default::default()
        });
        simple_responses([
            SimpleResponse {
                status_code: StatusCode::Code(400),
                mime_type: "application/json".parse().unwrap(),
                description: "Bad Request".to_string(),
                media_type: media_type.clone(),
            },
            SimpleResponse {
                status_code: StatusCode::Code(401),
                mime_type: "application/json".parse().unwrap(),
                description: "Invalid client_id".to_string(),
                media_type: media_type.clone(),
            },
            SimpleResponse {
                status_code: StatusCode::Code(500),
                mime_type: "application/json".parse().unwrap(),
                description: "Internal server error".to_string(),
                media_type,
            },
        ])
    }
}
