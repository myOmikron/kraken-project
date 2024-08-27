use actix_web::dev;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::ResponseError;

use crate::api::handler::common::error::ApiError;

pub(crate) fn handle_not_found<B>(
    res: dev::ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    Ok(ErrorHandlerResponse::Response(
        res.into_response(ApiError::NotFound.error_response())
            .map_into_right_body(),
    ))
}
