//!
//! JWT middleware
//!
//! Checks is the authorization token is valid.

use crate::config::JwtConfig;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http, Error,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::{ready, Ready};

///
/// Represents JWT claims that are used when creating/validating the JWT tokens.
///
/// See RFC 7519 4. JWT Claims
/// https://datatracker.ietf.org/doc/html/rfc7519#section-4
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Claims {
    pub sub: String,
    pub exp: usize,
}

///
/// JWT middleware factory.
/// Contains JWT configuration part of server configuration.
#[derive(Copy, Clone, Debug)]
pub struct JwtMiddleware {
    /// JWT part of configuration.
    pub jwt_config: &'static JwtConfig,
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            jwt_cfg: self.jwt_config,
            service,
        }))
    }
}

/// JWT middleware service, responsible for authorization token validation.
#[derive(Debug)]
pub struct JwtMiddlewareService<S> {
    service: S,
    jwt_cfg: &'static JwtConfig,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let secret = self.jwt_cfg.secret.as_bytes();
        let fut = self.service.call(req);

        Box::pin(async move {
            let req = fut.await?;
            let headers = req.request().headers();

            if let Some(auth_header) = headers.get(http::header::AUTHORIZATION) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        if let Ok(_token_data) = decode::<Claims>(
                            token,
                            &DecodingKey::from_secret(secret),
                            &Validation::default(),
                        ) {
                            return Ok(req);
                        }
                    }
                }
            }

            Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
        })
    }
}
