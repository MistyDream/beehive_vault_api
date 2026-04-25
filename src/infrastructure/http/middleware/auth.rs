//! Bearer-token auth middleware for the `/v1` scope.
//!
//! Compares the `Authorization: Bearer <token>` header to the shared
//! `API_KEY` using a constant-time comparison (`subtle::ConstantTimeEq`)
//! so the response time does not leak how many bytes of the key matched.
//!
//! Intended usage: wrap the `/v1` scope only. Healthchecks live outside
//! the scope and are therefore implicitly exempt — the middleware never
//! runs for them.

use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

use actix_web::body::EitherBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::AUTHORIZATION;
use actix_web::{Error, ResponseError};
use subtle::ConstantTimeEq;

use crate::infrastructure::http::error::ApiError;

pub struct BearerAuth {
    expected: Rc<String>,
}

impl BearerAuth {
    pub fn new(expected_key: String) -> Self {
        Self {
            expected: Rc::new(expected_key),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for BearerAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = BearerAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BearerAuthMiddleware {
            service: Rc::new(service),
            expected: self.expected.clone(),
        }))
    }
}

pub struct BearerAuthMiddleware<S> {
    service: Rc<S>,
    expected: Rc<String>,
}

impl<S, B> Service<ServiceRequest> for BearerAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let presented = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        let authorized = match presented {
            Some(token) => bool::from(self.expected.as_bytes().ct_eq(token.as_bytes())),
            None => false,
        };

        if authorized {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await.map(ServiceResponse::map_into_left_body) })
        } else {
            tracing::warn!(
                method = %req.method(),
                path = %req.path(),
                "request rejected by bearer auth"
            );
            let (request, _payload) = req.into_parts();
            let response = ApiError::Unauthorized.error_response();
            let service_res = ServiceResponse::new(request, response).map_into_right_body();
            Box::pin(async move { Ok(service_res) })
        }
    }
}
