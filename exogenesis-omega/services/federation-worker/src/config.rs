//! Federation worker configuration

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Federation worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Federation round interval (in seconds)
    pub federation_interval_secs: u64,

    /// Minimum quality threshold for pattern inclusion
    pub min_quality_threshold: f32,

    /// Number of top patterns to collect from each shard
    pub patterns_per_shard: i64,

    /// Trend decay rate (0.0-1.0, per hour)
    pub trend_decay_rate: f32,

    /// PostgreSQL connection URLs for all shards
    pub shard_postgres_urls: Vec<String>,

    /// Worker region identifier
    pub region: String,
}

impl FederationConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        // Load shard URLs from comma-separated list
        let shard_urls_str = env::var("SHARD_POSTGRES_URLS")
            .map_err(|_| anyhow::anyhow!("SHARD_POSTGRES_URLS environment variable is required"))?;

        let shard_postgres_urls: Vec<String> = shard_urls_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if shard_postgres_urls.is_empty() {
            anyhow::bail!("SHARD_POSTGRES_URLS must contain at least one URL");
        }

        Ok(Self {
            federation_interval_secs: env::var("FEDERATION_INTERVAL_SECS")
                .unwrap_or_else(|_| "3600".to_string()) // Default: 1 hour
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid FEDERATION_INTERVAL_SECS: {}", e))?,

            min_quality_threshold: env::var("MIN_QUALITY_THRESHOLD")
                .unwrap_or_else(|_| "0.8".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid MIN_QUALITY_THRESHOLD: {}", e))?,

            patterns_per_shard: env::var("PATTERNS_PER_SHARD")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid PATTERNS_PER_SHARD: {}", e))?,

            trend_decay_rate: env::var("TREND_DECAY_RATE")
                .unwrap_or_else(|_| "0.95".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid TREND_DECAY_RATE: {}", e))?,

            shard_postgres_urls,

            region: env::var("REGION")
                .unwrap_or_else(|_| "global".to_string()),
        })
    }

    /// Get federation interval as Duration
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.federation_interval_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federation_config_defaults() {
        env::set_var("SHARD_POSTGRES_URLS", "postgresql://localhost/shard1");

        let config = FederationConfig::from_env().unwrap();

        assert_eq!(config.federation_interval_secs, 3600);
        assert!((config.min_quality_threshold - 0.8).abs() < 0.001);
        assert_eq!(config.patterns_per_shard, 1000);
        assert_eq!(config.shard_postgres_urls.len(), 1);

        env::remove_var("SHARD_POSTGRES_URLS");
    }

    #[test]
    fn test_multiple_shards() {
        env::set_var(
            "SHARD_POSTGRES_URLS",
            "postgresql://localhost/shard1, postgresql://localhost/shard2, postgresql://localhost/shard3"
        );

        let config = FederationConfig::from_env().unwrap();

        assert_eq!(config.shard_postgres_urls.len(), 3);

        env::remove_var("SHARD_POSTGRES_URLS");
    }
}
