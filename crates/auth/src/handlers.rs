use crate::{
    email::{EmailError, EmailManager},
    error::{AuthError, Result},
    jwt::JwtManager,
    session::SessionManager,
    storage::AuthStorage,
    user::{PostgresUserRepository, UserRepository},
};
use actix_web::{post, web, HttpResponse, Responder};
use media_gateway_core::{
    ActivityEventType, KafkaActivityProducer, UserActivityEvent, UserActivityProducer,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// Registration Handler
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub email: String,
    pub message: String,
}

#[post("/api/v1/auth/register")]
pub async fn register(
    req: web::Json<RegisterRequest>,
    user_repo: web::Data<Arc<PostgresUserRepository>>,
    email_manager: web::Data<Arc<EmailManager>>,
) -> Result<impl Responder> {
    // Hash password
    use crate::user::PasswordHasher;
    let password_hasher = PasswordHasher::default();
    let password_hash = password_hasher.hash_password(&req.password)?;

    // Create user (email_verified defaults to false from migration)
    let display_name = req
        .display_name
        .clone()
        .unwrap_or_else(|| req.email.split('@').next().unwrap_or("User").to_string());

    let user = user_repo
        .create_user(&req.email, &password_hash, &display_name)
        .await?;

    // Send verification email
    let token = email_manager
        .send_verification_email(user.id.to_string(), user.email.clone())
        .await
        .map_err(|e| match e {
            EmailError::RateLimitExceeded => AuthError::RateLimitExceeded,
            _ => AuthError::Internal(format!("Failed to send verification email: {}", e)),
        })?;

    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        token = %token.token,
        "User registered, verification email sent"
    );

    Ok(HttpResponse::Created().json(RegisterResponse {
        user_id: user.id.to_string(),
        email: user.email,
        message: "Registration successful. Please check your email to verify your account."
            .to_string(),
    }))
}

// ============================================================================
// Email Verification Handler
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    pub message: String,
    pub email: String,
}

#[post("/api/v1/auth/verify-email")]
pub async fn verify_email(
    req: web::Json<VerifyEmailRequest>,
    user_repo: web::Data<Arc<PostgresUserRepository>>,
    email_manager: web::Data<Arc<EmailManager>>,
) -> Result<impl Responder> {
    // Verify token and get user info
    let verification = email_manager
        .verify_token(&req.token)
        .await
        .map_err(|e| match e {
            EmailError::InvalidToken => {
                AuthError::InvalidToken("Invalid or expired verification token".to_string())
            }
            EmailError::TokenExpired => {
                AuthError::InvalidToken("Verification token has expired".to_string())
            }
            _ => AuthError::Internal(format!("Failed to verify token: {}", e)),
        })?;

    // Update user's email_verified status
    let user_id = uuid::Uuid::parse_str(&verification.user_id)
        .map_err(|_| AuthError::Internal("Invalid user ID format".to_string()))?;

    user_repo.update_email_verified(user_id, true).await?;

    tracing::info!(
        user_id = %verification.user_id,
        email = %verification.email,
        "Email verified successfully"
    );

    Ok(HttpResponse::Ok().json(VerifyEmailResponse {
        message: "Email verified successfully. You can now log in.".to_string(),
        email: verification.email,
    }))
}

// ============================================================================
// Resend Verification Email Handler
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ResendVerificationResponse {
    pub message: String,
}

#[post("/api/v1/auth/resend-verification")]
pub async fn resend_verification(
    req: web::Json<ResendVerificationRequest>,
    user_repo: web::Data<Arc<PostgresUserRepository>>,
    email_manager: web::Data<Arc<EmailManager>>,
) -> Result<impl Responder> {
    // Get user by email
    let user = user_repo
        .find_by_email(&req.email)
        .await?
        .ok_or_else(|| AuthError::InvalidCredentials)?;

    // Check if already verified
    if user.email_verified {
        return Err(AuthError::Internal("Email is already verified".to_string()));
    }

    // Resend verification email (includes rate limiting)
    email_manager
        .resend_verification_email(user.id.to_string(), user.email.clone())
        .await
        .map_err(|e| match e {
            EmailError::RateLimitExceeded => AuthError::RateLimitExceeded,
            _ => AuthError::Internal(format!("Failed to resend verification email: {}", e)),
        })?;

    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        "Verification email resent"
    );

    Ok(HttpResponse::Ok().json(ResendVerificationResponse {
        message: "Verification email sent. Please check your inbox.".to_string(),
    }))
}

// ============================================================================
// Login Handler (Updated to check email verification)
// ============================================================================

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

#[post("/api/v1/auth/login")]
pub async fn login(
    req: web::Json<LoginRequest>,
    user_repo: web::Data<Arc<PostgresUserRepository>>,
    jwt_manager: web::Data<Arc<JwtManager>>,
    session_manager: web::Data<Arc<SessionManager>>,
    storage: web::Data<Arc<AuthStorage>>,
    activity_producer: web::Data<Option<Arc<KafkaActivityProducer>>>,
) -> Result<impl Responder> {
    // Get user by email
    let user = user_repo
        .find_by_email(&req.email)
        .await?
        .ok_or_else(|| AuthError::InvalidCredentials)?;

    // Verify password
    use crate::user::PasswordHasher;
    let password_hasher = PasswordHasher::default();
    if !password_hasher.verify_password(&req.password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    // Check if email is verified (configurable via env var)
    let require_verification = std::env::var("REQUIRE_EMAIL_VERIFICATION")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if require_verification && !user.email_verified {
        return Err(AuthError::EmailNotVerified);
    }

    // Generate tokens
    let access_token = jwt_manager.create_access_token(
        user.id.to_string(),
        Some(user.email.clone()),
        vec!["user".to_string()],
        vec!["read:profile".to_string(), "write:profile".to_string()],
    )?;

    let refresh_token = jwt_manager.create_refresh_token(
        user.id.to_string(),
        Some(user.email.clone()),
        vec!["user".to_string()],
        vec!["read:profile".to_string(), "write:profile".to_string()],
    )?;

    // Create session
    let refresh_claims = jwt_manager.verify_refresh_token(&refresh_token)?;
    session_manager
        .create_session(user.id.to_string(), refresh_claims.jti, None)
        .await?;

    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        "User logged in successfully"
    );

    // Publish user login activity event (non-blocking)
    if let Some(producer) = activity_producer.as_ref() {
        let metadata = serde_json::json!({
            "email": user.email.clone(),
            "login_time": chrono::Utc::now().to_rfc3339(),
        });

        let event = UserActivityEvent::new(user.id, ActivityEventType::UserLogin, metadata);

        let producer_clone = producer.clone();
        tokio::spawn(async move {
            if let Err(e) = producer_clone.publish_activity(event).await {
                tracing::warn!(error = %e, "Failed to publish login activity event");
            }
        });
    }

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
    use crate::{
        email::{ConsoleProvider, EmailConfig, EmailService},
        user::{PasswordHasher, User},
    };
    use actix_web::{test, web::Data, App};
    use std::sync::Arc;
    use uuid::Uuid;

    async fn setup_test_redis() -> redis::Client {
        redis::Client::open("redis://localhost:6379").unwrap()
    }

    #[tokio::test]
    async fn test_verify_email_success() {
        let redis = setup_test_redis().await;
        let email_config = EmailConfig::default();
        let provider = Arc::new(ConsoleProvider::new(
            email_config.base_url.clone(),
            email_config.from_name.clone(),
        ));
        let email_manager = EmailManager::new(provider, redis, email_config);

        // Create verification token
        let token = email_manager
            .create_verification_token("user123".to_string(), "test@example.com".to_string())
            .await
            .unwrap();

        // Verify token
        let verified = email_manager.verify_token(&token.token).await.unwrap();

        assert_eq!(verified.user_id, "user123");
        assert_eq!(verified.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_verify_email_invalid_token() {
        let redis = setup_test_redis().await;
        let email_config = EmailConfig::default();
        let provider = Arc::new(ConsoleProvider::new(
            email_config.base_url.clone(),
            email_config.from_name.clone(),
        ));
        let email_manager = EmailManager::new(provider, redis, email_config);

        let result = email_manager.verify_token("invalid_token").await;

        assert!(matches!(result, Err(EmailError::InvalidToken)));
    }

    #[tokio::test]
    async fn test_resend_verification_rate_limit() {
        let redis = setup_test_redis().await;
        let email_config = EmailConfig::default();
        let provider = Arc::new(ConsoleProvider::new(
            email_config.base_url.clone(),
            email_config.from_name.clone(),
        ));
        let email_manager = EmailManager::new(provider, redis, email_config);

        // First send succeeds
        let result1 = email_manager
            .send_verification_email("user123".to_string(), "test@example.com".to_string())
            .await;
        assert!(result1.is_ok());

        // Second send within 60 seconds fails
        let result2 = email_manager
            .send_verification_email("user456".to_string(), "test@example.com".to_string())
            .await;
        assert!(matches!(result2, Err(EmailError::RateLimitExceeded)));
    }
}
