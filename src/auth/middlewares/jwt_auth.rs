use std::rc::Rc;

use actix_web::{
    Error,
    dev::{ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};

use crate::{auth::jwt::verify_jwt, common::errors::api_error::ApiError};

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
        + 'static,
    B: 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Transform = JwtAuthMiddleware<S>;
    type Error = Error;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
        + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let auth_header = match req.headers().get("Authorization") {
                Some(header) => header.to_str().map_err(|_| {
                    return ApiError::Unauthorized("Invalid Authorization header".into());
                })?,
                None => {
                    return Err(
                        ApiError::Unauthorized("Authorization header missing".into()).into(),
                    );
                }
            };

            if !auth_header.starts_with("Bearer ") {
                return Err(
                    ApiError::Unauthorized("Invalid authorization header scheme".into()).into(),
                );
            }

            let token = &auth_header[7..];

            if verify_jwt(token).is_none() {
                return Err(ApiError::Unauthorized("Invalid or expired token".into()).into());
            };

            svc.call(req).await
        })
    }
}
