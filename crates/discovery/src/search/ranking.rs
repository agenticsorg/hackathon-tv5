use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use media_gateway_core::audit::{AuditAction, AuditEvent, AuditLogger, PostgresAuditLogger};

/// Ranking configuration with adjustable weights
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankingConfig {
    pub version: u32,
    pub vector_weight: f64,
    pub keyword_weight: f64,
    pub quality_weight: f64,
    pub freshness_weight: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RankingConfig {
    /// Create new ranking configuration with weight validation
    pub fn new(
        vector_weight: f64,
        keyword_weight: f64,
        quality_weight: f64,
        freshness_weight: f64,
        created_by: Option<Uuid>,
        description: Option<String>,
    ) -> Result<Self> {
        let config = Self {
            version: 1,
            vector_weight,
            keyword_weight,
            quality_weight,
            freshness_weight,
            created_at: Utc::now(),
            created_by,
            description,
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate that weights sum to 1.0 (with small tolerance for floating point)
    pub fn validate(&self) -> Result<()> {
        let sum =
            self.vector_weight + self.keyword_weight + self.quality_weight + self.freshness_weight;

        const EPSILON: f64 = 0.0001;
        if (sum - 1.0).abs() > EPSILON {
            return Err(anyhow::anyhow!("Weights must sum to 1.0, got {:.4}", sum));
        }

        if self.vector_weight < 0.0
            || self.keyword_weight < 0.0
            || self.quality_weight < 0.0
            || self.freshness_weight < 0.0
        {
            return Err(anyhow::anyhow!("All weights must be non-negative"));
        }

        Ok(())
    }

    /// Get total weight (should always be 1.0 if validated)
    pub fn total_weight(&self) -> f64 {
        self.vector_weight + self.keyword_weight + self.quality_weight + self.freshness_weight
    }
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            version: 1,
            vector_weight: 0.35,
            keyword_weight: 0.30,
            quality_weight: 0.20,
            freshness_weight: 0.15,
            created_at: Utc::now(),
            created_by: None,
            description: Some("Default ranking configuration".to_string()),
        }
    }
}

/// Named ranking configuration for A/B testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedRankingConfig {
    pub name: String,
    pub config: RankingConfig,
    pub is_active: bool,
    pub traffic_percentage: Option<u8>,
}

/// Request to update ranking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRankingConfigRequest {
    pub vector_weight: f64,
    pub keyword_weight: f64,
    pub quality_weight: f64,
    pub freshness_weight: f64,
    pub description: Option<String>,
}

impl UpdateRankingConfigRequest {
    pub fn validate(&self) -> Result<()> {
        let sum =
            self.vector_weight + self.keyword_weight + self.quality_weight + self.freshness_weight;

        const EPSILON: f64 = 0.0001;
        if (sum - 1.0).abs() > EPSILON {
            return Err(anyhow::anyhow!("Weights must sum to 1.0, got {:.4}", sum));
        }

        if self.vector_weight < 0.0
            || self.keyword_weight < 0.0
            || self.quality_weight < 0.0
            || self.freshness_weight < 0.0
        {
            return Err(anyhow::anyhow!("All weights must be non-negative"));
        }

        Ok(())
    }
}

/// Ranking configuration store with Redis backend and versioning
pub struct RankingConfigStore {
    redis: ConnectionManager,
    audit_logger: Arc<PostgresAuditLogger>,
}

const DEFAULT_CONFIG_KEY: &str = "ranking:config:default";
const CONFIG_VERSION_KEY: &str = "ranking:config:version";
const NAMED_CONFIG_PREFIX: &str = "ranking:config:named";

impl RankingConfigStore {
    /// Create new ranking config store
    #[instrument(skip(redis_url, db_pool))]
    pub async fn new(redis_url: &str, db_pool: sqlx::PgPool) -> Result<Self> {
        info!("Initializing RankingConfigStore");

        let client = Client::open(redis_url).context("Failed to create Redis client")?;

        let redis = ConnectionManager::new(client)
            .await
            .context("Failed to create Redis connection manager")?;

        let audit_logger = Arc::new(PostgresAuditLogger::new(db_pool));

        Ok(Self {
            redis,
            audit_logger,
        })
    }

    /// Get current default ranking configuration
    #[instrument(skip(self))]
    pub async fn get_default_config(&self) -> Result<RankingConfig> {
        let mut conn = self.redis.clone();

        let config_json: Option<String> = conn
            .get(DEFAULT_CONFIG_KEY)
            .await
            .context("Failed to get default ranking config from Redis")?;

        match config_json {
            Some(json) => {
                let config: RankingConfig =
                    serde_json::from_str(&json).context("Failed to deserialize ranking config")?;
                debug!(version = config.version, "Retrieved default ranking config");
                Ok(config)
            }
            None => {
                info!("No default ranking config found, using built-in default");
                let default = RankingConfig::default();
                self.set_default_config(&default, None).await?;
                Ok(default)
            }
        }
    }

    /// Set default ranking configuration with versioning
    #[instrument(skip(self, config))]
    pub async fn set_default_config(
        &self,
        config: &RankingConfig,
        admin_id: Option<Uuid>,
    ) -> Result<()> {
        config.validate()?;

        let mut conn = self.redis.clone();

        // Increment version
        let version: u32 = conn
            .incr::<_, _, u32>(CONFIG_VERSION_KEY, 1)
            .await
            .context("Failed to increment config version")?;

        let mut versioned_config = config.clone();
        versioned_config.version = version;
        versioned_config.created_at = Utc::now();
        versioned_config.created_by = admin_id;

        let config_json = serde_json::to_string(&versioned_config)
            .context("Failed to serialize ranking config")?;

        conn.set::<_, _, ()>(DEFAULT_CONFIG_KEY, &config_json)
            .await
            .context("Failed to set default ranking config in Redis")?;

        // Store historical version
        let history_key = format!("ranking:config:history:{}", version);
        conn.set_ex::<_, _, ()>(&history_key, &config_json, 30 * 24 * 3600)
            .await
            .context("Failed to store ranking config history")?;

        info!(
            version = version,
            admin_id = ?admin_id,
            "Updated default ranking configuration"
        );

        // Audit log
        if let Some(admin_id) = admin_id {
            let metadata = serde_json::json!({
                "version": version,
                "vector_weight": versioned_config.vector_weight,
                "keyword_weight": versioned_config.keyword_weight,
                "quality_weight": versioned_config.quality_weight,
                "freshness_weight": versioned_config.freshness_weight,
                "description": versioned_config.description,
            });

            let event = AuditEvent::new(AuditAction::Update, "RankingConfig".to_string())
                .with_user_id(admin_id)
                .with_resource_id(format!("default:v{}", version))
                .with_details(metadata);

            if let Err(e) = self.audit_logger.log(event).await {
                warn!(error = %e, "Failed to log audit event for ranking config update");
            }
        }

        Ok(())
    }

    /// Get named ranking configuration for A/B testing
    #[instrument(skip(self))]
    pub async fn get_named_config(&self, name: &str) -> Result<Option<NamedRankingConfig>> {
        let mut conn = self.redis.clone();

        let key = format!("{}:{}", NAMED_CONFIG_PREFIX, name);
        let config_json: Option<String> = conn
            .get(&key)
            .await
            .context("Failed to get named ranking config from Redis")?;

        match config_json {
            Some(json) => {
                let config: NamedRankingConfig = serde_json::from_str(&json)
                    .context("Failed to deserialize named ranking config")?;
                debug!(name = name, "Retrieved named ranking config");
                Ok(Some(config))
            }
            None => {
                debug!(name = name, "Named ranking config not found");
                Ok(None)
            }
        }
    }

    /// Set named ranking configuration for A/B testing
    #[instrument(skip(self, config))]
    pub async fn set_named_config(
        &self,
        name: &str,
        config: &RankingConfig,
        is_active: bool,
        traffic_percentage: Option<u8>,
        admin_id: Option<Uuid>,
    ) -> Result<()> {
        config.validate()?;

        if let Some(pct) = traffic_percentage {
            if pct > 100 {
                return Err(anyhow::anyhow!(
                    "Traffic percentage must be between 0 and 100"
                ));
            }
        }

        let named_config = NamedRankingConfig {
            name: name.to_string(),
            config: config.clone(),
            is_active,
            traffic_percentage,
        };

        let mut conn = self.redis.clone();

        let key = format!("{}:{}", NAMED_CONFIG_PREFIX, name);
        let config_json = serde_json::to_string(&named_config)
            .context("Failed to serialize named ranking config")?;

        conn.set::<_, _, ()>(&key, &config_json)
            .await
            .context("Failed to set named ranking config in Redis")?;

        info!(
            name = name,
            is_active = is_active,
            traffic_percentage = ?traffic_percentage,
            admin_id = ?admin_id,
            "Updated named ranking configuration"
        );

        // Audit log
        if let Some(admin_id) = admin_id {
            let metadata = serde_json::json!({
                "name": name,
                "is_active": is_active,
                "traffic_percentage": traffic_percentage,
                "vector_weight": config.vector_weight,
                "keyword_weight": config.keyword_weight,
                "quality_weight": config.quality_weight,
                "freshness_weight": config.freshness_weight,
            });

            let event = AuditEvent::new(AuditAction::Update, "NamedRankingConfig".to_string())
                .with_user_id(admin_id)
                .with_resource_id(name.to_string())
                .with_details(metadata);

            if let Err(e) = self.audit_logger.log(event).await {
                warn!(error = %e, "Failed to log audit event for named ranking config update");
            }
        }

        Ok(())
    }

    /// List all named ranking configurations
    #[instrument(skip(self))]
    pub async fn list_named_configs(&self) -> Result<Vec<NamedRankingConfig>> {
        let mut conn = self.redis.clone();

        let pattern = format!("{}:*", NAMED_CONFIG_PREFIX);
        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .context("Failed to list named ranking configs")?;

        let mut configs = Vec::new();
        for key in keys {
            let config_json: Option<String> = conn
                .get(&key)
                .await
                .context("Failed to get named ranking config")?;

            if let Some(json) = config_json {
                if let Ok(config) = serde_json::from_str::<NamedRankingConfig>(&json) {
                    configs.push(config);
                }
            }
        }

        debug!(count = configs.len(), "Listed named ranking configs");
        Ok(configs)
    }

    /// Delete named ranking configuration
    #[instrument(skip(self))]
    pub async fn delete_named_config(&self, name: &str, admin_id: Option<Uuid>) -> Result<bool> {
        let mut conn = self.redis.clone();

        let key = format!("{}:{}", NAMED_CONFIG_PREFIX, name);
        let deleted: u64 = conn
            .del::<_, u64>(&key)
            .await
            .context("Failed to delete named ranking config")?;

        if deleted > 0 {
            info!(name = name, admin_id = ?admin_id, "Deleted named ranking configuration");

            // Audit log
            if let Some(admin_id) = admin_id {
                let metadata = serde_json::json!({
                    "name": name,
                });

                let event = AuditEvent::new(AuditAction::Delete, "NamedRankingConfig".to_string())
                    .with_user_id(admin_id)
                    .with_resource_id(name.to_string())
                    .with_details(metadata);

                if let Err(e) = self.audit_logger.log(event).await {
                    warn!(error = %e, "Failed to log audit event for named ranking config deletion");
                }
            }

            Ok(true)
        } else {
            debug!(name = name, "Named ranking config not found for deletion");
            Ok(false)
        }
    }

    /// Get ranking config for A/B test variant
    #[instrument(skip(self))]
    pub async fn get_config_for_variant(&self, variant: Option<&str>) -> Result<RankingConfig> {
        match variant {
            Some(name) => {
                if let Some(named_config) = self.get_named_config(name).await? {
                    if named_config.is_active {
                        debug!(variant = name, "Using named ranking config for A/B test");
                        return Ok(named_config.config);
                    } else {
                        warn!(
                            variant = name,
                            "Requested variant is not active, using default"
                        );
                    }
                } else {
                    warn!(variant = name, "Requested variant not found, using default");
                }
            }
            None => {
                debug!("No variant specified, using default ranking config");
            }
        }

        self.get_default_config().await
    }

    /// Get configuration version history
    #[instrument(skip(self))]
    pub async fn get_config_history(&self, version: u32) -> Result<Option<RankingConfig>> {
        let mut conn = self.redis.clone();

        let history_key = format!("ranking:config:history:{}", version);
        let config_json: Option<String> = conn
            .get(&history_key)
            .await
            .context("Failed to get ranking config history from Redis")?;

        match config_json {
            Some(json) => {
                let config: RankingConfig = serde_json::from_str(&json)
                    .context("Failed to deserialize ranking config history")?;
                debug!(version = version, "Retrieved ranking config history");
                Ok(Some(config))
            }
            None => {
                debug!(version = version, "Ranking config history not found");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranking_config_validation() {
        let valid = RankingConfig::new(0.35, 0.30, 0.20, 0.15, None, None);
        assert!(valid.is_ok());

        let invalid_sum = RankingConfig::new(0.5, 0.3, 0.2, 0.2, None, None);
        assert!(invalid_sum.is_err());

        let invalid_negative = RankingConfig::new(0.5, -0.1, 0.3, 0.3, None, None);
        assert!(invalid_negative.is_err());
    }

    #[test]
    fn test_default_config() {
        let config = RankingConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.total_weight(), 1.0);
    }

    #[test]
    fn test_update_request_validation() {
        let valid = UpdateRankingConfigRequest {
            vector_weight: 0.4,
            keyword_weight: 0.3,
            quality_weight: 0.2,
            freshness_weight: 0.1,
            description: Some("Test config".to_string()),
        };
        assert!(valid.validate().is_ok());

        let invalid = UpdateRankingConfigRequest {
            vector_weight: 0.5,
            keyword_weight: 0.5,
            quality_weight: 0.5,
            freshness_weight: 0.5,
            description: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[tokio::test]
    async fn test_config_store_lifecycle() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

        let db_pool = match sqlx::PgPool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(_) => {
                eprintln!("Skipping test: PostgreSQL not available");
                return;
            }
        };

        let store = match RankingConfigStore::new(&redis_url, db_pool).await {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        // Test default config
        let config = store.get_default_config().await.unwrap();
        assert!(config.validate().is_ok());

        // Test update config
        let new_config =
            RankingConfig::new(0.4, 0.3, 0.2, 0.1, None, Some("Test update".to_string())).unwrap();
        store.set_default_config(&new_config, None).await.unwrap();

        let retrieved = store.get_default_config().await.unwrap();
        assert_eq!(retrieved.vector_weight, 0.4);
        assert_eq!(retrieved.keyword_weight, 0.3);
    }

    #[tokio::test]
    async fn test_named_config() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

        let db_pool = match sqlx::PgPool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(_) => {
                eprintln!("Skipping test: PostgreSQL not available");
                return;
            }
        };

        let store = match RankingConfigStore::new(&redis_url, db_pool).await {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let config = RankingConfig::new(
            0.5,
            0.25,
            0.15,
            0.1,
            None,
            Some("High vector weight variant".to_string()),
        )
        .unwrap();

        store
            .set_named_config("high_vector", &config, true, Some(50), None)
            .await
            .unwrap();

        let named = store.get_named_config("high_vector").await.unwrap();
        assert!(named.is_some());
        assert_eq!(named.unwrap().config.vector_weight, 0.5);

        let configs = store.list_named_configs().await.unwrap();
        assert!(!configs.is_empty());

        let deleted = store
            .delete_named_config("high_vector", None)
            .await
            .unwrap();
        assert!(deleted);
    }
}
