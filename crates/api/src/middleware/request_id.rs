use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use uuid::Uuid;

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService { service }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
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
        // Extract or generate request ID
        let request_id = req
            .headers()
            .get("X-Request-ID")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Extract correlation ID if present
        let correlation_id = req
            .headers()
            .get("X-Correlation-ID")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| request_id.clone());

        // Check for AI agent headers
        let is_ai_agent =
            req.headers().get("X-AI-Agent").is_some() || req.headers().get("AI-Agent-ID").is_some();

        // Store in request extensions
        req.extensions_mut().insert(RequestIdData {
            request_id: request_id.clone(),
            correlation_id: correlation_id.clone(),
            is_ai_agent,
        });

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // Add request ID to response headers
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                actix_web::http::header::HeaderValue::from_str(&request_id).unwrap(),
            );

            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-correlation-id"),
                actix_web::http::header::HeaderValue::from_str(&correlation_id).unwrap(),
            );

            Ok(res)
        })
    }
}

#[derive(Clone, Debug)]
pub struct RequestIdData {
    pub request_id: String,
    pub correlation_id: String,
    pub is_ai_agent: bool,
}

// Helper to extract request ID from request
pub fn get_request_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions()
        .get::<RequestIdData>()
        .map(|data| data.request_id.clone())
}

pub fn get_correlation_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions()
        .get::<RequestIdData>()
        .map(|data| data.correlation_id.clone())
}

pub fn is_ai_agent(req: &actix_web::HttpRequest) -> bool {
    req.extensions()
        .get::<RequestIdData>()
        .map(|data| data.is_ai_agent)
        .unwrap_or(false)
}
