use crate::{
    error::AuthError, jwt::JwtManager, middleware::UserContext, rbac::Role, session::SessionManager,
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
    task::{Context, Poll},
};

/// Admin-only middleware that verifies user has admin role
pub struct AdminMiddleware {
    jwt_manager: Rc<JwtManager>,
    session_manager: Rc<SessionManager>,
}

impl AdminMiddleware {
    pub fn new(jwt_manager: Rc<JwtManager>, session_manager: Rc<SessionManager>) -> Self {
        Self {
            jwt_manager,
            session_manager,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AdminMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminMiddlewareService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminMiddlewareService {
            service: Rc::new(service),
            jwt_manager: self.jwt_manager.clone(),
            session_manager: self.session_manager.clone(),
        }))
    }
}

pub struct AdminMiddlewareService<S> {
    service: Rc<S>,
    jwt_manager: Rc<JwtManager>,
    session_manager: Rc<SessionManager>,
}

impl<S, B> Service<ServiceRequest> for AdminMiddlewareService<S>
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
        let jwt_manager = self.jwt_manager.clone();
        let session_manager = self.session_manager.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| {
                    Error::from(AuthError::InvalidToken(
                        "Missing Authorization header".to_string(),
                    ))
                })?;

            let token =
                JwtManager::extract_bearer_token(auth_header).map_err(|e| Error::from(e))?;

            // Verify JWT
            let claims = jwt_manager
                .verify_access_token(token)
                .map_err(|e| Error::from(e))?;

            // Check if token is revoked
            if session_manager
                .is_token_revoked(&claims.jti)
                .await
                .map_err(|e| Error::from(e))?
            {
                return Err(Error::from(AuthError::InvalidToken(
                    "Token revoked".to_string(),
                )));
            }

            // Create user context
            let user_context = UserContext::from_claims(&claims);

            // CRITICAL: Verify admin role
            if !user_context.has_role(&Role::Admin) {
                tracing::warn!(
                    user_id = %user_context.user_id,
                    roles = ?user_context.roles,
                    path = %req.path(),
                    "Unauthorized admin access attempt"
                );
                return Err(Error::from(AuthError::InsufficientPermissions));
            }

            // Insert user context into request extensions
            req.extensions_mut().insert(user_context.clone());

            tracing::info!(
                user_id = %user_context.user_id,
                path = %req.path(),
                method = %req.method(),
                "Admin action authorized"
            );

            // Continue to next middleware/handler
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::Claims;

    #[test]
    fn test_user_context_admin_role_check() {
        let claims = Claims {
            sub: "admin123".to_string(),
            email: Some("admin@example.com".to_string()),
            roles: vec!["admin".to_string()],
            scopes: vec!["admin:*".to_string()],
            iat: 0,
            exp: 0,
            jti: "jti123".to_string(),
            token_type: "access".to_string(),
            token_family_id: None,
        };

        let context = UserContext::from_claims(&claims);
        assert!(context.has_role(&Role::Admin));
    }

    #[test]
    fn test_user_context_non_admin_role_check() {
        let claims = Claims {
            sub: "user123".to_string(),
            email: Some("user@example.com".to_string()),
            roles: vec!["free_user".to_string()],
            scopes: vec!["read:content".to_string()],
            iat: 0,
            exp: 0,
            jti: "jti123".to_string(),
            token_type: "access".to_string(),
            token_family_id: None,
        };

        let context = UserContext::from_claims(&claims);
        assert!(!context.has_role(&Role::Admin));
    }
}
