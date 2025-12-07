use crate::{
    api_keys::manager::ApiKeyManager,
    error::{AuthError, Result},
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApiKeyContext {
    pub key_id: Uuid,
    pub user_id: Uuid,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: i32,
}

pub struct ApiKeyAuthMiddleware {
    manager: Arc<ApiKeyManager>,
}

impl ApiKeyAuthMiddleware {
    pub fn new(manager: Arc<ApiKeyManager>) -> Self {
        Self { manager }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthMiddlewareService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthMiddlewareService {
            service: Rc::new(service),
            manager: self.manager.clone(),
        }))
    }
}

pub struct ApiKeyAuthMiddlewareService<S> {
    service: Rc<S>,
    manager: Arc<ApiKeyManager>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddlewareService<S>
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
        let manager = self.manager.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| {
                    Error::from(AuthError::InvalidToken(
                        "Missing Authorization header".to_string(),
                    ))
                })?;

            let api_key = extract_api_key(auth_header).map_err(|e| Error::from(e))?;

            let verified_key = manager
                .verify_key(&api_key)
                .await
                .map_err(|e| Error::from(e))?;

            let context = ApiKeyContext {
                key_id: verified_key.id,
                user_id: verified_key.user_id,
                scopes: verified_key.scopes,
                rate_limit_per_minute: verified_key.rate_limit_per_minute,
            };

            tokio::spawn({
                let manager = manager.clone();
                let key_id = verified_key.id;
                async move {
                    let _ = manager.update_last_used(key_id).await;
                }
            });

            req.extensions_mut().insert(context);

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

fn extract_api_key(auth_header: &str) -> Result<String> {
    if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
        Ok(stripped.to_string())
    } else {
        Err(AuthError::InvalidToken(
            "Invalid Authorization header format".to_string(),
        ))
    }
}

pub fn extract_api_key_context(req: &actix_web::HttpRequest) -> Result<ApiKeyContext> {
    req.extensions()
        .get::<ApiKeyContext>()
        .cloned()
        .ok_or(AuthError::Internal("API key context not found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_api_key() {
        let header = "Bearer mg_live_x7k9m2p4q8r1s5t3u6v0w2y4z8a1b3c5";
        let key = extract_api_key(header).unwrap();
        assert_eq!(key, "mg_live_x7k9m2p4q8r1s5t3u6v0w2y4z8a1b3c5");
    }

    #[test]
    fn test_extract_api_key_invalid() {
        let header = "InvalidFormat mg_live_x7k9m2p4q8r1s5t3u6v0w2y4z8a1b3c5";
        let result = extract_api_key(header);
        assert!(result.is_err());
    }
}
