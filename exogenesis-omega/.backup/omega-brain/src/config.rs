use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration for OmegaBrain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    /// Vector embedding dimensions (default: 384)
    pub dimensions: usize,

    /// Maximum patterns to store (default: 10,000)
    pub max_patterns: usize,

    /// Sync interval in seconds (default: 300-900, 5-15 minutes)
    pub sync_interval_secs: u64,

    /// Path to ONNX model file
    pub model_path: PathBuf,

    /// Path for persistent storage
    pub storage_path: PathBuf,
}

impl BrainConfig {
    pub fn new(model_path: PathBuf, storage_path: PathBuf) -> Self {
        Self {
            dimensions: 384,
            max_patterns: 10_000,
            sync_interval_secs: 600, // 10 minutes
            model_path,
            storage_path,
        }
    }

    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.dimensions = dimensions;
        self
    }

    pub fn with_max_patterns(mut self, max_patterns: usize) -> Self {
        self.max_patterns = max_patterns;
        self
    }

    pub fn with_sync_interval(mut self, sync_interval_secs: u64) -> Self {
        self.sync_interval_secs = sync_interval_secs;
        self
    }
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            dimensions: 384,
            max_patterns: 10_000,
            sync_interval_secs: 600,
            model_path: PathBuf::from("/data/omega/model.onnx"),
            storage_path: PathBuf::from("/data/omega/brain.db"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = BrainConfig::default()
            .with_dimensions(512)
            .with_max_patterns(20_000)
            .with_sync_interval(300);

        assert_eq!(config.dimensions, 512);
        assert_eq!(config.max_patterns, 20_000);
        assert_eq!(config.sync_interval_secs, 300);
    }
}
