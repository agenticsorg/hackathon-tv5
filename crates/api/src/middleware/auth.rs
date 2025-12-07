use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

#[derive(Clone)]
pub struct AuthMiddleware {
    pub optional: bool,
}

impl AuthMiddleware {
    pub fn required() -> Self {
        Self { optional: false }
    }

    pub fn optional() -> Self {
        Self { optional: true }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            optional: self.optional,
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    optional: bool,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        // Extract bearer token from Authorization header
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        // Parse and validate JWT token
        let user_context = if let Some(token) = token {
            match validate_token(&token) {
                Ok(claims) => Some(UserContext {
                    user_id: claims.sub,
                    tier: claims.tier,
                    is_authenticated: true,
                }),
                Err(_) => {
                    if !self.optional {
                        return Box::pin(async move {
                            Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                        });
                    }
                    None
                }
            }
        } else {
            if !self.optional {
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("Missing authorization"))
                });
            }
            None
        };

        // Store user context or anonymous user
        let context = user_context.unwrap_or_else(|| UserContext {
            user_id: "anonymous".to_string(),
            tier: "anonymous".to_string(),
            is_authenticated: false,
        });

        req.extensions_mut().insert(context);

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub tier: String,
    pub is_authenticated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,
    tier: String,
    exp: i64,
}

fn validate_token(token: &str) -> Result<TokenClaims, jsonwebtoken::errors::Error> {
    // In a real implementation, this would:
    // 1. Validate JWT signature
    // 2. Check expiration
    // 3. Verify issuer
    // 4. Call auth service for additional validation

    // For now, we'll do basic JWT parsing without signature validation
    // This should be replaced with proper validation in production

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set - cannot start with default secret");

    let token_data = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

// Helper to extract user context from request
pub fn get_user_context(req: &actix_web::HttpRequest) -> Option<UserContext> {
    req.extensions().get::<UserContext>().cloned()
}

pub fn require_user_context(req: &actix_web::HttpRequest) -> Result<UserContext, actix_web::Error> {
    get_user_context(req)
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Authentication required"))
}
