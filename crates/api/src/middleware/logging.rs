use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::time::Instant;
use tracing::{info, warn};

pub struct LoggingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddlewareService { service }))
    }
}

pub struct LoggingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddlewareService<S>
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
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();

        // Extract request ID from extensions
        let request_id = req
            .extensions()
            .get::<super::request_id::RequestIdData>()
            .map(|data| data.request_id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Extract user context
        let user_id = req
            .extensions()
            .get::<super::auth::UserContext>()
            .map(|ctx| ctx.user_id.clone())
            .unwrap_or_else(|| "anonymous".to_string());

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start.elapsed();

            match &result {
                Ok(res) => {
                    let status = res.status();
                    let duration_ms = duration.as_millis();

                    if status.is_success() {
                        info!(
                            request_id = %request_id,
                            user_id = %user_id,
                            method = %method,
                            path = %path,
                            query = %query,
                            status = status.as_u16(),
                            duration_ms = duration_ms,
                            "Request completed"
                        );
                    } else if status.is_client_error() {
                        warn!(
                            request_id = %request_id,
                            user_id = %user_id,
                            method = %method,
                            path = %path,
                            query = %query,
                            status = status.as_u16(),
                            duration_ms = duration_ms,
                            "Request failed (client error)"
                        );
                    } else {
                        warn!(
                            request_id = %request_id,
                            user_id = %user_id,
                            method = %method,
                            path = %path,
                            query = %query,
                            status = status.as_u16(),
                            duration_ms = duration_ms,
                            "Request failed (server error)"
                        );
                    }
                }
                Err(err) => {
                    warn!(
                        request_id = %request_id,
                        user_id = %user_id,
                        method = %method,
                        path = %path,
                        query = %query,
                        error = %err,
                        duration_ms = duration.as_millis(),
                        "Request error"
                    );
                }
            }

            result
        })
    }
}
