use crate::{
    error::{AuthError, Result},
    jwt::{Claims, JwtManager},
    rbac::{Permission, RbacManager, Role},
    session::SessionManager,
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

/// User context extracted from JWT
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub email: Option<String>,
    pub roles: Vec<Role>,
    pub scopes: Vec<String>,
    pub jti: String,
}

impl UserContext {
    pub fn from_claims(claims: &Claims) -> Self {
        let roles = claims
            .roles
            .iter()
            .filter_map(|r| Role::from_str(r))
            .collect();

        Self {
            user_id: claims.sub.clone(),
            email: claims.email.clone(),
            roles,
            scopes: claims.scopes.clone(),
            jti: claims.jti.clone(),
        }
    }

    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    jwt_manager: Rc<JwtManager>,
    session_manager: Rc<SessionManager>,
    rbac_manager: Rc<RbacManager>,
    required_permission: Option<Permission>,
}

impl AuthMiddleware {
    pub fn new(
        jwt_manager: Rc<JwtManager>,
        session_manager: Rc<SessionManager>,
        rbac_manager: Rc<RbacManager>,
    ) -> Self {
        Self {
            jwt_manager,
            session_manager,
            rbac_manager,
            required_permission: None,
        }
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.required_permission = Some(permission);
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            jwt_manager: self.jwt_manager.clone(),
            session_manager: self.session_manager.clone(),
            rbac_manager: self.rbac_manager.clone(),
            required_permission: self.required_permission.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_manager: Rc<JwtManager>,
    session_manager: Rc<SessionManager>,
    rbac_manager: Rc<RbacManager>,
    required_permission: Option<Permission>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        let rbac_manager = self.rbac_manager.clone();
        let required_permission = self.required_permission.clone();
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

            // Check required permission if specified
            if let Some(permission) = required_permission {
                rbac_manager
                    .require_permission(&user_context.roles, &permission)
                    .map_err(|e| Error::from(e))?;
            }

            // Insert user context into request extensions
            req.extensions_mut().insert(user_context);

            // Continue to next middleware/handler
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

/// Extract user context from request
pub fn extract_user_context(req: &actix_web::HttpRequest) -> Result<UserContext> {
    req.extensions()
        .get::<UserContext>()
        .cloned()
        .ok_or(AuthError::Internal("User context not found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_context_from_claims() {
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
        assert_eq!(context.user_id, "user123");
        assert!(context.has_role(&Role::FreeUser));
    }
}
