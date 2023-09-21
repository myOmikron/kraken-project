use std::future::{ready, Ready};

use actix_toolbox::tb_middleware::actix_session::SessionExt;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::LocalBoxFuture;

use crate::api::handler::ApiError;

pub(crate) struct TokenRequired;

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
        ready(Ok(TokenRequiredMiddleware { service }))
    }
}

pub(crate) struct TokenRequiredMiddleware<S> {
    service: S,
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
        let session = req.get_session();

        let mut token = None;

        if let Some(auth) = req.headers().get("Authorization") {
            if let Ok(header) = auth.to_str() {
                let parts: Vec<&str> = header.split(' ').collect();
                if parts.len() == 2 && parts[0] == "Bearer" && !parts[1].is_empty() {
                    token = Some(parts[1].to_owned());
                }
            }
        }

        let next = self.service.call(req);
        Box::pin(async move {
            let Some(token) = token else {
                return Err(ApiError::Unauthenticated.into());
            };

            session.insert("token", token)?;

            next.await
        })
    }
}
