use std::future::ready;
use std::future::Ready;

use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use futures::future::LocalBoxFuture;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;

use crate::api::handler::common::error::ApiError;
use crate::chan::global::GLOBAL;
use crate::models::BearerToken;

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
        let auth_header = req.headers().get("Authorization").cloned();
        let next = self.service.call(req);
        Box::pin(async move {
            let auth_headers = auth_header
                .ok_or(ApiError::Unauthenticated)?
                .to_str()
                .map_err(|_| ApiError::Unauthenticated)?
                .to_owned();

            let split = auth_headers.split(' ');
            let mut count = 0;

            for half in split {
                count += 1;
                match count {
                    1 => {
                        if half != "Bearer" {
                            return Err(ApiError::Unauthenticated.into());
                        }
                    }
                    2 => {
                        query!(&GLOBAL.db, BearerToken)
                            .condition(BearerToken::F.token.equals(half))
                            .optional()
                            .await
                            .map_err(ApiError::DatabaseError)?
                            .ok_or(ApiError::Unauthenticated)?;
                    }
                    _ => break,
                }
            }

            if count != 2 {
                return Err(ApiError::Unauthenticated.into());
            }

            next.await
        })
    }
}
