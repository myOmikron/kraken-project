use std::future::{ready, Ready};

use actix_toolbox::tb_middleware::actix_session::SessionExt;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::web::Data;
use futures::future::LocalBoxFuture;
use rorm::{query, Database, Model};

use crate::api::handler::ApiError;
use crate::models::UserKey;

pub(crate) struct AuthenticationRequired;

impl<S, B> Transform<S, ServiceRequest> for AuthenticationRequired
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = AuthenticationRequiredMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationRequiredMiddleware { service }))
    }
}

pub(crate) struct AuthenticationRequiredMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationRequiredMiddleware<S>
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

        let logged_in = session
            .get("logged_in")
            .map(|logged_in_maybe| logged_in_maybe.map_or(false, |v| v));

        let second_factor = session
            .get("2fa")
            .map(|sec_fac| sec_fac.map_or(false, |v| v));

        let uuid = session.get("uuid");

        let db = if let Some(db) = &req.app_data::<Data<Database>>() {
            db.get_ref().clone()
        } else {
            panic!("Did you forgot to provide the Database to your App?")
        };

        let next = self.service.call(req);
        Box::pin(async move {
            if !logged_in.map_err(ApiError::SessionGet)? {
                return Err(ApiError::Unauthenticated.into());
            }

            let uuid: Vec<u8> = uuid
                .map_err(ApiError::SessionGet)?
                .ok_or(ApiError::SessionCorrupt)?;

            let second_factor_required = query!(&db, (UserKey::F.id,))
                .condition(UserKey::F.user.equals(&uuid))
                .optional()
                .await
                .map_err(ApiError::DatabaseError)?;

            if second_factor_required.is_some() && !second_factor.map_err(ApiError::SessionGet)? {
                return Err(ApiError::Missing2FA.into());
            }

            next.await
        })
    }
}
