use crate::{
    error::{AuthError, Result},
    jwt::JwtManager,
    user::{
        password::PasswordHasher,
        repository::{CreateUserRequest, UserRepository, UserResponse},
    },
};
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

pub struct UserHandlerState {
    pub user_repository: Arc<dyn UserRepository>,
    pub password_hasher: Arc<PasswordHasher>,
    pub jwt_manager: Arc<JwtManager>,
    pub require_email_verification: bool,
}

#[post("/api/v1/auth/register")]
pub async fn register(
    req: web::Json<CreateUserRequest>,
    state: web::Data<UserHandlerState>,
) -> Result<impl Responder> {
    // Validate password strength
    let strength = PasswordHasher::validate_password_strength(&req.password);
    if !strength.is_valid {
        return Err(AuthError::Internal(format!(
            "Password validation failed: {}",
            strength.errors.join(", ")
        )));
    }

    // Hash password
    let password_hash = state.password_hasher.hash_password(&req.password)?;

    // Create user
    let user = state
        .user_repository
        .create_user(&req.email, &password_hash, &req.display_name)
        .await?;

    // Convert to response (no password hash)
    let user_response: UserResponse = user.into();

    Ok(HttpResponse::Created().json(RegisterResponse {
        user: user_response,
    }))
}

#[post("/api/v1/auth/login")]
pub async fn login(
    req: web::Json<LoginRequest>,
    state: web::Data<UserHandlerState>,
) -> Result<impl Responder> {
    // Find user by email
    let user = state
        .user_repository
        .find_by_email(&req.email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    let is_valid = state
        .password_hasher
        .verify_password(&req.password, &user.password_hash)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Check email verification if required
    if state.require_email_verification && !user.email_verified {
        return Err(AuthError::Internal(
            "Email verification required".to_string(),
        ));
    }

    // Generate tokens
    let access_token = state.jwt_manager.create_access_token(
        user.id.to_string(),
        Some(user.email.clone()),
        vec!["user".to_string()],
        vec!["read:content".to_string(), "write:content".to_string()],
    )?;

    let refresh_token = state.jwt_manager.create_refresh_token(
        user.id.to_string(),
        Some(user.email.clone()),
        vec!["user".to_string()],
        vec!["read:content".to_string(), "write:content".to_string()],
    )?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::repository::User;
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    struct MockUserRepository {
        users: std::sync::Mutex<Vec<User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create_user(
            &self,
            email: &str,
            password_hash: &str,
            display_name: &str,
        ) -> Result<User> {
            let user = User {
                id: Uuid::new_v4(),
                email: email.to_string(),
                password_hash: password_hash.to_string(),
                display_name: display_name.to_string(),
                email_verified: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };

            self.users.lock().unwrap().push(user.clone());
            Ok(user)
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.email == email).cloned())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.id == id).cloned())
        }

        async fn update_email_verified(&self, id: Uuid, verified: bool) -> Result<()> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.iter_mut().find(|u| u.id == id) {
                user.email_verified = verified;
            }
            Ok(())
        }
    }

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"email":"test@example.com","password":"Test1234"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.password, "Test1234");
    }

    #[test]
    fn test_register_response_serialization() {
        let user_response = UserResponse {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            email_verified: false,
            created_at: Utc::now(),
        };

        let response = RegisterResponse {
            user: user_response,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test@example.com"));
    }
}
