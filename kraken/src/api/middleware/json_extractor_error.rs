use actix_web::error::JsonPayloadError;
use actix_web::HttpRequest;
use log::info;

use crate::api::handler::ApiError;

pub(crate) fn json_extractor_error(err: JsonPayloadError, _req: &HttpRequest) -> actix_web::Error {
    match err {
        JsonPayloadError::ContentType => ApiError::InvalidContentType.into(),
        JsonPayloadError::Deserialize(err) => ApiError::InvalidJson(err).into(),
        JsonPayloadError::Serialize(err) => ApiError::InvalidJson(err).into(),
        JsonPayloadError::Overflow { .. } => ApiError::PayloadOverflow(err.to_string()).into(),
        JsonPayloadError::OverflowKnownLength { .. } => {
            ApiError::PayloadOverflow(err.to_string()).into()
        }
        JsonPayloadError::Payload(err) => {
            info!("Payload error {err}");
            ApiError::InternalServerError.into()
        }
        _ => ApiError::InternalServerError.into(),
    }
}
