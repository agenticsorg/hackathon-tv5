use crate::error::{AuthError, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Password reset token - 32-byte random hex string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub created_at: u64,
    pub expires_at: u64,
}

impl PasswordResetToken {
    /// Generate new password reset token with 1-hour TTL
    pub fn new(user_id: String, email: String) -> Self {
        let token = generate_reset_token();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = created_at + 3600; // 1 hour

        Self {
            token,
            user_id,
            email,
            created_at,
            expires_at,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
}

/// Generate 32-byte random hex token
fn generate_reset_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Request to initiate password reset
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

/// Response for password reset request
#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

/// Request to reset password with token
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
    #[serde(default)]
    pub keep_current_session: bool,
}

/// Response for password reset
#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
    pub sessions_invalidated: u32,
    pub tokens_revoked: u32,
}

/// Password strength validator
pub struct PasswordValidator;

impl PasswordValidator {
    /// Validate password meets minimum requirements
    /// - At least 8 characters
    /// - Contains at least one uppercase letter
    /// - Contains at least one lowercase letter
    /// - Contains at least one digit
    pub fn validate(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(AuthError::Internal(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AuthError::Internal(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(AuthError::Internal(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_numeric()) {
            return Err(AuthError::Internal(
                "Password must contain at least one digit".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_reset_token_length() {
        let token = generate_reset_token();
        assert_eq!(token.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_generate_reset_token_unique() {
        let token1 = generate_reset_token();
        let token2 = generate_reset_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_password_reset_token_creation() {
        let token = PasswordResetToken::new("user123".to_string(), "test@example.com".to_string());

        assert_eq!(token.user_id, "user123");
        assert_eq!(token.email, "test@example.com");
        assert_eq!(token.token.len(), 64);
        assert!(token.expires_at > token.created_at);
        assert_eq!(token.expires_at - token.created_at, 3600);
    }

    #[test]
    fn test_password_reset_token_not_expired() {
        let token = PasswordResetToken::new("user123".to_string(), "test@example.com".to_string());
        assert!(!token.is_expired());
    }

    #[test]
    fn test_password_reset_token_expired() {
        let mut token =
            PasswordResetToken::new("user123".to_string(), "test@example.com".to_string());
        // Set expiration to past
        token.expires_at = token.created_at - 1;
        assert!(token.is_expired());
    }

    #[test]
    fn test_password_validator_valid() {
        assert!(PasswordValidator::validate("Password123").is_ok());
        assert!(PasswordValidator::validate("StrongPass1").is_ok());
        assert!(PasswordValidator::validate("MySecure123Password").is_ok());
    }

    #[test]
    fn test_password_validator_too_short() {
        let result = PasswordValidator::validate("Pass1");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 8 characters"));
    }

    #[test]
    fn test_password_validator_no_uppercase() {
        let result = PasswordValidator::validate("password123");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("uppercase letter"));
    }

    #[test]
    fn test_password_validator_no_lowercase() {
        let result = PasswordValidator::validate("PASSWORD123");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lowercase letter"));
    }

    #[test]
    fn test_password_validator_no_digit() {
        let result = PasswordValidator::validate("PasswordOnly");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("digit"));
    }
}
