use std::future::{ready, Ready};

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::HeaderValue;
use futures::future::LocalBoxFuture;
use log::debug;

use crate::api::handler::ApiError;

pub(crate) struct TokenRequired(pub(crate) String);

impl<S, B> Transform<S, ServiceRequest> for TokenRequired
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = TokenRequiredMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TokenRequiredMiddleware {
            service,
            token: self.0.clone(),
        }))
    }
}

pub(crate) struct TokenRequiredMiddleware<S> {
    service: S,
    token: String,
}

impl<S, B> Service<ServiceRequest> for TokenRequiredMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut authenticated = false;

        if let Some(auth) = req.headers().get("Authorization") {
            if self.token.is_empty() {
                authenticated = false
            } else {
                authenticated = match HeaderValue::try_from(&format!("Bearer {}", &self.token)) {
                    Ok(v) => auth == v,
                    Err(err) => {
                        debug!("Invalid header value: {err}");
                        false
                    }
                }
            }
        }

        let next = self.service.call(req);
        Box::pin(async move {
            if !authenticated {
                return Err(ApiError::Unauthenticated.into());
            }

            next.await
        })
    }
}
