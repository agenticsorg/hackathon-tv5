use actix_web::{HttpResponse, ResponseError};

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid PKCE verifier")]
    InvalidPkceVerifier,

    #[error("Invalid authorization code")]
    InvalidAuthCode,

    #[error("Authorization code already used")]
    AuthCodeReused,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid redirect URI")]
    InvalidRedirectUri,

    #[error("Invalid client")]
    InvalidClient,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Device code not found")]
    DeviceCodeNotFound,

    #[error("Authorization pending")]
    AuthorizationPending,

    #[error("Access denied")]
    AccessDenied,

    #[error("Device code expired")]
    DeviceCodeExpired,

    #[error("Invalid user code")]
    InvalidUserCode,

    #[error("Device already approved")]
    DeviceAlreadyApproved,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("MFA not enrolled")]
    MfaNotEnrolled,

    #[error("MFA already enrolled")]
    MfaAlreadyEnrolled,

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("MFA enrollment not found")]
    MfaEnrollmentNotFound,

    #[error("Invalid backup code")]
    InvalidBackupCode,

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("User not found")]
    UserNotFound,
}

impl From<anyhow::Error> for AuthError {
    fn from(err: anyhow::Error) -> Self {
        AuthError::Internal(err.to_string())
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::InvalidCredentials => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_credentials",
                "error_description": "Invalid username or password"
            })),
            AuthError::InvalidToken(_) | AuthError::TokenExpired => HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "invalid_token",
                    "error_description": self.to_string()
                })),
            AuthError::InvalidPkceVerifier => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "Invalid code verifier"
            })),
            AuthError::InvalidAuthCode | AuthError::AuthCodeReused => HttpResponse::BadRequest()
                .json(serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": self.to_string()
                })),
            AuthError::InsufficientPermissions => {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "insufficient_scope",
                    "error_description": "Insufficient permissions"
                }))
            }
            AuthError::InvalidScope(scope) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_scope",
                "error_description": format!("Invalid scope: {}", scope)
            })),
            AuthError::InvalidRedirectUri => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "Invalid redirect URI"
            })),
            AuthError::InvalidClient => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_client",
                "error_description": "Invalid client ID"
            })),
            AuthError::RateLimitExceeded => {
                HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "rate_limit_exceeded",
                    "error_description": "Too many requests"
                }))
            }
            AuthError::AuthorizationPending => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "authorization_pending",
                "error_description": "User has not yet completed authorization"
            })),
            AuthError::AccessDenied => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "access_denied",
                "error_description": "User denied authorization"
            })),
            AuthError::DeviceCodeExpired => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "expired_token",
                "error_description": "Device code expired"
            })),
            AuthError::InvalidUserCode => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "Invalid user code"
            })),
            AuthError::DeviceAlreadyApproved => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": "Device already approved"
                }))
            }
            AuthError::Unauthorized => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "error_description": "Authentication required"
            })),
            AuthError::MfaNotEnrolled => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "mfa_not_enrolled",
                "error_description": "MFA is not enrolled for this user"
            })),
            AuthError::MfaAlreadyEnrolled => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "mfa_already_enrolled",
                "error_description": "MFA is already enrolled for this user"
            })),
            AuthError::InvalidMfaCode => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_mfa_code",
                "error_description": "Invalid MFA code"
            })),
            AuthError::InvalidBackupCode => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_backup_code",
                "error_description": "Invalid backup code"
            })),
            AuthError::EmailNotVerified => HttpResponse::Forbidden().json(serde_json::json!({
                "error": "email_not_verified",
                "error_description": "Please verify your email address before logging in"
            })),
            AuthError::ValidationError(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "error_description": msg
            })),
            AuthError::DatabaseError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "database_error",
                    "error_description": "Database operation failed"
                }))
            }
            AuthError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "internal_error",
                    "error_description": "Internal server error"
                }))
            }
            AuthError::UserNotFound => HttpResponse::NotFound().json(serde_json::json!({
                "error": "user_not_found",
                "error_description": "User not found"
            })),
            _ => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "server_error",
                "error_description": "Internal server error"
            })),
        }
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for AuthError {
    fn from(err: redis::RedisError) -> Self {
        AuthError::Redis(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AuthError::InvalidToken(err.to_string())
    }
}

// ResponseError trait already provides From<AuthError> for actix_web::Error
