use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for TV Brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TVBrainConfig {
    /// Embedding dimension (384 for MiniLM, not default 4096)
    pub dimension: usize,

    /// HNSW graph connectivity (32 recommended)
    pub hnsw_m: usize,

    /// HNSW search accuracy (100 recommended)
    pub hnsw_ef: usize,

    /// Maximum patterns stored per TV (10,000 recommended)
    pub max_patterns: usize,

    /// Sync interval in seconds (300-900 recommended)
    pub sync_interval_secs: u64,

    /// Storage path for SQLite and data
    pub storage_path: PathBuf,

    /// Constellation server URL
    pub constellation_url: String,

    /// Device identifier
    pub device_id: String,

    /// Enable neural training
    pub enable_neural_training: bool,

    /// Recommendation timeout in milliseconds (15ms target)
    pub recommend_timeout_ms: u64,
}

impl Default for TVBrainConfig {
    fn default() -> Self {
        Self {
            dimension: 384,                 // MiniLM embeddings
            hnsw_m: 32,                     // Graph connectivity
            hnsw_ef: 100,                   // Search accuracy
            max_patterns: 10_000,           // Max patterns per TV
            sync_interval_secs: 600,        // 10 minutes
            storage_path: PathBuf::from("/data/omega"),
            constellation_url: "https://constellation.exogenesis-omega.io".to_string(),
            device_id: uuid::Uuid::new_v4().to_string(),
            enable_neural_training: true,
            recommend_timeout_ms: 15,       // 15ms target for recommendations
        }
    }
}

impl TVBrainConfig {
    /// Create production configuration
    pub fn production() -> Self {
        Self {
            sync_interval_secs: 900,        // 15 minutes in production
            ..Default::default()
        }
    }

    /// Create development configuration
    pub fn development() -> Self {
        Self {
            storage_path: PathBuf::from("./data/omega-dev"),
            constellation_url: "http://localhost:8080".to_string(),
            sync_interval_secs: 300,        // 5 minutes in dev
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.dimension == 0 {
            anyhow::bail!("dimension must be > 0");
        }
        if self.hnsw_m == 0 {
            anyhow::bail!("hnsw_m must be > 0");
        }
        if self.hnsw_ef == 0 {
            anyhow::bail!("hnsw_ef must be > 0");
        }
        if self.max_patterns == 0 {
            anyhow::bail!("max_patterns must be > 0");
        }
        if self.sync_interval_secs == 0 {
            anyhow::bail!("sync_interval_secs must be > 0");
        }
        if self.device_id.is_empty() {
            anyhow::bail!("device_id cannot be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TVBrainConfig::default();
        assert_eq!(config.dimension, 384);
        assert_eq!(config.hnsw_m, 32);
        assert_eq!(config.hnsw_ef, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = TVBrainConfig::default();
        config.dimension = 0;
        assert!(config.validate().is_err());
    }
}
