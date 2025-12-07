use crate::error::{AuthError, Result};
use media_gateway_core::audit::{AuditAction, AuditEvent, AuditLogger, PostgresAuditLogger};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum UserTier {
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "premium")]
    Premium,
    #[serde(rename = "enterprise")]
    Enterprise,
}

impl std::fmt::Display for UserTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserTier::Anonymous => write!(f, "anonymous"),
            UserTier::Free => write!(f, "free"),
            UserTier::Premium => write!(f, "premium"),
            UserTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

impl std::str::FromStr for UserTier {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "anonymous" => Ok(UserTier::Anonymous),
            "free" => Ok(UserTier::Free),
            "premium" => Ok(UserTier::Premium),
            "enterprise" => Ok(UserTier::Enterprise),
            _ => Err(AuthError::Internal(format!("Invalid user tier: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub endpoint: String,
    pub tier: UserTier,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
}

impl RateLimitConfig {
    pub fn new(
        endpoint: String,
        tier: UserTier,
        requests_per_minute: u32,
        requests_per_hour: u32,
        burst_size: u32,
    ) -> Self {
        Self {
            endpoint,
            tier,
            requests_per_minute,
            requests_per_hour,
            burst_size,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.endpoint.is_empty() {
            return Err(AuthError::Internal("Endpoint cannot be empty".to_string()));
        }

        if self.requests_per_minute == 0 && self.requests_per_hour == 0 {
            return Err(AuthError::Internal(
                "At least one rate limit must be non-zero".to_string(),
            ));
        }

        if self.burst_size == 0 {
            return Err(AuthError::Internal(
                "Burst size must be non-zero".to_string(),
            ));
        }

        if self.burst_size < self.requests_per_minute {
            return Err(AuthError::Internal(
                "Burst size must be >= requests per minute".to_string(),
            ));
        }

        Ok(())
    }

    pub fn matches_endpoint(&self, endpoint: &str) -> bool {
        if self.endpoint.ends_with('*') {
            let prefix = &self.endpoint[..self.endpoint.len() - 1];
            endpoint.starts_with(prefix)
        } else {
            endpoint == self.endpoint
        }
    }
}

#[derive(Clone)]
pub struct RateLimitConfigStore {
    redis_client: redis::Client,
    db_pool: PgPool,
}

impl RateLimitConfigStore {
    pub fn new(redis_client: redis::Client, db_pool: PgPool) -> Self {
        Self {
            redis_client,
            db_pool,
        }
    }

    async fn get_redis_conn(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::Internal(format!("Redis connection error: {}", e)))
    }

    fn redis_key(endpoint: &str, tier: UserTier) -> String {
        format!("rate_limit_config:{}:{}", endpoint, tier)
    }

    pub async fn get_config(
        &self,
        endpoint: &str,
        tier: UserTier,
    ) -> Result<Option<RateLimitConfig>> {
        let mut conn = self.get_redis_conn().await?;
        let key = Self::redis_key(endpoint, tier);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let config: RateLimitConfig = serde_json::from_str(&v)
                    .map_err(|e| AuthError::Internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_configs(&self) -> Result<Vec<RateLimitConfig>> {
        let mut conn = self.get_redis_conn().await?;
        let pattern = "rate_limit_config:*";

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis KEYS error: {}", e)))?;

        let mut configs = Vec::new();
        for key in keys {
            let value: Option<String> = conn
                .get(&key)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

            if let Some(v) = value {
                if let Ok(config) = serde_json::from_str::<RateLimitConfig>(&v) {
                    configs.push(config);
                }
            }
        }

        Ok(configs)
    }

    pub async fn set_config(&self, config: &RateLimitConfig) -> Result<()> {
        config.validate()?;

        let mut conn = self.get_redis_conn().await?;
        let key = Self::redis_key(&config.endpoint, config.tier);
        let value = serde_json::to_string(config)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        conn.set::<_, _, ()>(&key, value)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    pub async fn delete_config(&self, endpoint: &str, tier: UserTier) -> Result<bool> {
        let mut conn = self.get_redis_conn().await?;
        let key = Self::redis_key(endpoint, tier);

        let deleted: usize = conn
            .del::<_, usize>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;

        Ok(deleted > 0)
    }

    pub async fn get_matching_config(
        &self,
        endpoint: &str,
        tier: UserTier,
    ) -> Result<Option<RateLimitConfig>> {
        if let Some(config) = self.get_config(endpoint, tier).await? {
            return Ok(Some(config));
        }

        let all_configs = self.get_all_configs().await?;
        for config in all_configs {
            if config.tier == tier && config.matches_endpoint(endpoint) {
                return Ok(Some(config));
            }
        }

        Ok(None)
    }

    pub async fn get_default_config(&self, tier: UserTier) -> RateLimitConfig {
        match tier {
            UserTier::Anonymous => RateLimitConfig {
                endpoint: "*".to_string(),
                tier,
                requests_per_minute: 10,
                requests_per_hour: 100,
                burst_size: 15,
            },
            UserTier::Free => RateLimitConfig {
                endpoint: "*".to_string(),
                tier,
                requests_per_minute: 30,
                requests_per_hour: 1000,
                burst_size: 50,
            },
            UserTier::Premium => RateLimitConfig {
                endpoint: "*".to_string(),
                tier,
                requests_per_minute: 100,
                requests_per_hour: 5000,
                burst_size: 150,
            },
            UserTier::Enterprise => RateLimitConfig {
                endpoint: "*".to_string(),
                tier,
                requests_per_minute: 500,
                requests_per_hour: 50000,
                burst_size: 1000,
            },
        }
    }

    pub async fn get_effective_config(
        &self,
        endpoint: &str,
        tier: UserTier,
    ) -> Result<RateLimitConfig> {
        match self.get_matching_config(endpoint, tier).await? {
            Some(config) => Ok(config),
            None => Ok(self.get_default_config(tier).await),
        }
    }

    pub async fn log_config_change(
        &self,
        admin_user_id: Uuid,
        action: &str,
        config: &RateLimitConfig,
    ) -> Result<()> {
        let audit_logger = PostgresAuditLogger::new(self.db_pool.clone());

        let audit_action = match action {
            "create" => AuditAction::AdminAction,
            "update" => AuditAction::AdminAction,
            "delete" => AuditAction::AdminAction,
            _ => AuditAction::AdminAction,
        };

        let metadata = serde_json::json!({
            "endpoint": config.endpoint,
            "tier": config.tier.to_string(),
            "requests_per_minute": config.requests_per_minute,
            "requests_per_hour": config.requests_per_hour,
            "burst_size": config.burst_size,
        });

        let event = AuditEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_id: Some(admin_user_id),
            action: audit_action,
            resource_type: "rate_limit_config".to_string(),
            resource_id: Some(format!("{}:{}", config.endpoint, config.tier)),
            details: metadata,
            ip_address: None,
            user_agent: None,
        };

        audit_logger
            .log(event)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to log audit event: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_tier_from_str() {
        assert_eq!(
            "anonymous".parse::<UserTier>().unwrap(),
            UserTier::Anonymous
        );
        assert_eq!("free".parse::<UserTier>().unwrap(), UserTier::Free);
        assert_eq!("premium".parse::<UserTier>().unwrap(), UserTier::Premium);
        assert_eq!(
            "enterprise".parse::<UserTier>().unwrap(),
            UserTier::Enterprise
        );
        assert!("invalid".parse::<UserTier>().is_err());
    }

    #[test]
    fn test_user_tier_display() {
        assert_eq!(UserTier::Anonymous.to_string(), "anonymous");
        assert_eq!(UserTier::Free.to_string(), "free");
        assert_eq!(UserTier::Premium.to_string(), "premium");
        assert_eq!(UserTier::Enterprise.to_string(), "enterprise");
    }

    #[test]
    fn test_rate_limit_config_validation() {
        let valid_config =
            RateLimitConfig::new("/api/v1/test".to_string(), UserTier::Free, 10, 100, 15);
        assert!(valid_config.validate().is_ok());

        let empty_endpoint = RateLimitConfig::new("".to_string(), UserTier::Free, 10, 100, 15);
        assert!(empty_endpoint.validate().is_err());

        let zero_limits =
            RateLimitConfig::new("/api/v1/test".to_string(), UserTier::Free, 0, 0, 15);
        assert!(zero_limits.validate().is_err());

        let zero_burst =
            RateLimitConfig::new("/api/v1/test".to_string(), UserTier::Free, 10, 100, 0);
        assert!(zero_burst.validate().is_err());

        let invalid_burst =
            RateLimitConfig::new("/api/v1/test".to_string(), UserTier::Free, 10, 100, 5);
        assert!(invalid_burst.validate().is_err());
    }

    #[test]
    fn test_endpoint_matching() {
        let exact_config =
            RateLimitConfig::new("/api/v1/users".to_string(), UserTier::Free, 10, 100, 15);
        assert!(exact_config.matches_endpoint("/api/v1/users"));
        assert!(!exact_config.matches_endpoint("/api/v1/users/123"));

        let wildcard_config =
            RateLimitConfig::new("/api/v1/*".to_string(), UserTier::Free, 10, 100, 15);
        assert!(wildcard_config.matches_endpoint("/api/v1/users"));
        assert!(wildcard_config.matches_endpoint("/api/v1/users/123"));
        assert!(wildcard_config.matches_endpoint("/api/v1/posts"));
        assert!(!wildcard_config.matches_endpoint("/api/v2/users"));
    }

    #[test]
    fn test_redis_key_format() {
        let key = RateLimitConfigStore::redis_key("/api/v1/test", UserTier::Free);
        assert_eq!(key, "rate_limit_config:/api/v1/test:free");
    }

    #[test]
    fn test_default_configs() {
        let store = RateLimitConfigStore {
            redis_client: redis::Client::open("redis://127.0.0.1:6379").unwrap(),
            db_pool: PgPool::connect_lazy("postgresql://localhost/test").unwrap(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let anon = store.get_default_config(UserTier::Anonymous).await;
            assert_eq!(anon.requests_per_minute, 10);
            assert_eq!(anon.requests_per_hour, 100);

            let free = store.get_default_config(UserTier::Free).await;
            assert_eq!(free.requests_per_minute, 30);
            assert_eq!(free.requests_per_hour, 1000);

            let premium = store.get_default_config(UserTier::Premium).await;
            assert_eq!(premium.requests_per_minute, 100);
            assert_eq!(premium.requests_per_hour, 5000);

            let enterprise = store.get_default_config(UserTier::Enterprise).await;
            assert_eq!(enterprise.requests_per_minute, 500);
            assert_eq!(enterprise.requests_per_hour, 50000);
        });
    }
}
