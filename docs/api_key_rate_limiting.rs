// API Key Rate Limiting Integration Example
// This shows how to add rate limiting based on API key configuration

use crate::{
    api_keys::middleware::extract_api_key_context,
    error::{AuthError, Result},
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use redis::{AsyncCommands, Client};
use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
};

/// Rate limiter that uses API key-specific limits
pub struct ApiKeyRateLimiter {
    redis_client: Client,
}

impl ApiKeyRateLimiter {
    pub fn new(redis_client: Client) -> Self {
        Self { redis_client }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyRateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyRateLimiterService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyRateLimiterService {
            service: Rc::new(service),
            redis_client: self.redis_client.clone(),
        }))
    }
}

pub struct ApiKeyRateLimiterService<S> {
    service: Rc<S>,
    redis_client: Client,
}

impl<S, B> Service<ServiceRequest> for ApiKeyRateLimiterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let redis_client = self.redis_client.clone();

        Box::pin(async move {
            // Extract API key context from request (set by ApiKeyAuthMiddleware)
            let api_key_context = req
                .extensions()
                .get::<crate::api_keys::middleware::ApiKeyContext>()
                .cloned()
                .ok_or_else(|| Error::from(AuthError::Internal("API key context not found".to_string())))?;

            // Check rate limit using Redis
            let mut conn = redis_client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| Error::from(AuthError::Internal(format!("Redis error: {}", e))))?;

            let key = format!("ratelimit:apikey:{}", api_key_context.key_id);
            let current_minute = chrono::Utc::now().timestamp() / 60;
            let window_key = format!("{}:{}", key, current_minute);

            // Increment counter
            let count: i32 = conn
                .incr(&window_key, 1)
                .await
                .map_err(|e| Error::from(AuthError::Internal(format!("Redis error: {}", e))))?;

            // Set expiration on first request in window
            if count == 1 {
                let _: () = conn
                    .expire(&window_key, 120) // 2 minutes TTL
                    .await
                    .map_err(|e| Error::from(AuthError::Internal(format!("Redis error: {}", e))))?;
            }

            // Check if over limit
            if count > api_key_context.rate_limit_per_minute {
                return Err(Error::from(AuthError::RateLimitExceeded));
            }

            // Continue to next middleware/handler
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

// Usage in server.rs:
//
// HttpServer::new(move || {
//     App::new()
//         .wrap(RateLimitMiddleware::new(redis_client.clone(), rate_limit_config.clone()))
//         .app_data(app_state.clone())
//         .service(
//             web::scope("/api/v1")
//                 .wrap(ApiKeyAuthMiddleware::new(api_key_manager.clone()))
//                 .wrap(ApiKeyRateLimiter::new(redis_client.clone()))
//                 .service(protected_endpoint)
//         )
//         .service(create_api_key) // Uses JWT auth
//         .service(list_api_keys)   // Uses JWT auth
//         .service(revoke_api_key)  // Uses JWT auth
// })
