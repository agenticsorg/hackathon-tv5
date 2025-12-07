use async_trait::async_trait;
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Failed to send email: {0}")]
    SendFailed(String),

    #[error("Invalid verification token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Template error: {0}")]
    Template(String),
}

pub type Result<T> = std::result::Result<T, EmailError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationToken {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub created_at: i64,
}

impl VerificationToken {
    pub fn new(user_id: String, email: String) -> Self {
        let token = Self::generate_token();
        let created_at = chrono::Utc::now().timestamp();

        Self {
            token,
            user_id,
            email,
            created_at,
        }
    }

    fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        hex::encode(bytes)
    }
}

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_verification(&self, email: &str, token: &str) -> Result<()>;
    async fn send_password_reset(&self, email: &str, token: &str) -> Result<()>;
    async fn send_password_changed(&self, email: &str) -> Result<()>;
}

pub struct EmailManager {
    provider: Arc<dyn EmailService>,
    redis: redis::Client,
    config: super::EmailConfig,
}

impl EmailManager {
    pub fn new(
        provider: Arc<dyn EmailService>,
        redis: redis::Client,
        config: super::EmailConfig,
    ) -> Self {
        Self {
            provider,
            redis,
            config,
        }
    }

    pub async fn create_verification_token(
        &self,
        user_id: String,
        email: String,
    ) -> Result<VerificationToken> {
        let token = VerificationToken::new(user_id, email);

        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let key = format!("email_verification:{}", token.token);
        let ttl = self.config.verification_ttl_hours * 3600;

        let token_data =
            serde_json::to_string(&token).map_err(|e| EmailError::Template(e.to_string()))?;

        conn.set_ex::<_, _, ()>(&key, token_data, ttl as u64)
            .await?;

        Ok(token)
    }

    pub async fn verify_token(&self, token: &str) -> Result<VerificationToken> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let key = format!("email_verification:{}", token);

        let token_data: Option<String> = conn.get(&key).await?;

        match token_data {
            Some(data) => {
                let verification_token: VerificationToken =
                    serde_json::from_str(&data).map_err(|e| EmailError::Template(e.to_string()))?;

                // Delete token after verification
                conn.del::<_, ()>(&key).await?;

                Ok(verification_token)
            }
            None => Err(EmailError::InvalidToken),
        }
    }

    pub async fn check_resend_rate_limit(&self, email: &str) -> Result<bool> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let key = format!("email_resend_limit:{}", email);

        let exists: bool = conn.exists(&key).await?;

        if exists {
            Err(EmailError::RateLimitExceeded)
        } else {
            // Set rate limit: 1 per minute
            conn.set_ex::<_, _, ()>(&key, "1", 60).await?;
            Ok(true)
        }
    }

    pub async fn send_verification_email(
        &self,
        user_id: String,
        email: String,
    ) -> Result<VerificationToken> {
        // Check rate limit
        self.check_resend_rate_limit(&email).await?;

        // Create token
        let token = self
            .create_verification_token(user_id, email.clone())
            .await?;

        // Send email
        self.provider
            .send_verification(&email, &token.token)
            .await?;

        Ok(token)
    }

    pub async fn resend_verification_email(
        &self,
        user_id: String,
        email: String,
    ) -> Result<VerificationToken> {
        self.send_verification_email(user_id, email).await
    }

    pub async fn send_password_reset_email(&self, email: String, token: String) -> Result<()> {
        // Send email via provider
        self.provider.send_password_reset(&email, &token).await?;
        Ok(())
    }

    pub async fn send_password_changed_notification(&self, email: String) -> Result<()> {
        // Send email via provider
        self.provider.send_password_changed(&email).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockEmailProvider {
        sent_emails: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl MockEmailProvider {
        fn new() -> Self {
            Self {
                sent_emails: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl EmailService for MockEmailProvider {
        async fn send_verification(&self, email: &str, token: &str) -> Result<()> {
            self.sent_emails
                .lock()
                .unwrap()
                .push((email.to_string(), token.to_string()));
            Ok(())
        }

        async fn send_password_reset(&self, email: &str, token: &str) -> Result<()> {
            self.sent_emails
                .lock()
                .unwrap()
                .push((email.to_string(), token.to_string()));
            Ok(())
        }

        async fn send_password_changed(&self, email: &str) -> Result<()> {
            self.sent_emails
                .lock()
                .unwrap()
                .push((email.to_string(), "changed".to_string()));
            Ok(())
        }
    }

    #[test]
    fn test_verification_token_generation() {
        let token = VerificationToken::new("user123".to_string(), "test@example.com".to_string());

        assert_eq!(token.user_id, "user123");
        assert_eq!(token.email, "test@example.com");
        assert_eq!(token.token.len(), 64); // 32 bytes = 64 hex chars
        assert!(token.created_at > 0);
    }

    #[test]
    fn test_tokens_are_unique() {
        let token1 = VerificationToken::new("user1".to_string(), "test1@example.com".to_string());
        let token2 = VerificationToken::new("user2".to_string(), "test2@example.com".to_string());

        assert_ne!(token1.token, token2.token);
    }

    #[tokio::test]
    async fn test_email_manager_create_and_verify_token() {
        let provider = Arc::new(MockEmailProvider::new());
        let redis = redis::Client::open("redis://localhost:6379").unwrap();
        let config = super::super::EmailConfig::default();

        let manager = EmailManager::new(provider, redis, config);

        let token = manager
            .create_verification_token("user123".to_string(), "test@example.com".to_string())
            .await
            .unwrap();

        let verified = manager.verify_token(&token.token).await.unwrap();

        assert_eq!(verified.user_id, "user123");
        assert_eq!(verified.email, "test@example.com");
        assert_eq!(verified.token, token.token);
    }

    #[tokio::test]
    async fn test_verify_invalid_token() {
        let provider = Arc::new(MockEmailProvider::new());
        let redis = redis::Client::open("redis://localhost:6379").unwrap();
        let config = super::super::EmailConfig::default();

        let manager = EmailManager::new(provider, redis, config);

        let result = manager.verify_token("invalid_token").await;

        assert!(matches!(result, Err(EmailError::InvalidToken)));
    }

    #[tokio::test]
    async fn test_token_deleted_after_verification() {
        let provider = Arc::new(MockEmailProvider::new());
        let redis = redis::Client::open("redis://localhost:6379").unwrap();
        let config = super::super::EmailConfig::default();

        let manager = EmailManager::new(provider, redis, config);

        let token = manager
            .create_verification_token("user123".to_string(), "test@example.com".to_string())
            .await
            .unwrap();

        // First verification succeeds
        manager.verify_token(&token.token).await.unwrap();

        // Second verification fails (token deleted)
        let result = manager.verify_token(&token.token).await;
        assert!(matches!(result, Err(EmailError::InvalidToken)));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let provider = Arc::new(MockEmailProvider::new());
        let redis = redis::Client::open("redis://localhost:6379").unwrap();
        let config = super::super::EmailConfig::default();

        let manager = EmailManager::new(provider, redis, config);

        // First send succeeds
        manager
            .send_verification_email("user123".to_string(), "test@example.com".to_string())
            .await
            .unwrap();

        // Second send within rate limit fails
        let result = manager
            .send_verification_email("user123".to_string(), "test@example.com".to_string())
            .await;

        assert!(matches!(result, Err(EmailError::RateLimitExceeded)));
    }
}
