//! Builder pattern API for Omega Brain initialization
//!
//! Provides a fluent interface for configuring and building an OmegaBrain instance

use std::path::PathBuf;
use std::time::Duration;
use crate::error::{OmegaError, Result};

/// Builder for OmegaBrain configuration
#[derive(Debug, Clone)]
pub struct OmegaBuilder {
    /// Path to ONNX model file
    model_path: Option<PathBuf>,

    /// Path for persistent storage
    storage_path: Option<PathBuf>,

    /// Constellation URL for sync
    constellation_url: Option<String>,

    /// Sync interval (default: 5 minutes)
    sync_interval: Duration,

    /// Maximum patterns to store (default: 10,000)
    max_patterns: usize,

    /// Vector dimensions (default: 384 for MiniLM)
    dimensions: usize,
}

impl Default for OmegaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OmegaBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            model_path: None,
            storage_path: None,
            constellation_url: None,
            sync_interval: Duration::from_secs(300), // 5 minutes
            max_patterns: 10_000,
            dimensions: 384,
        }
    }

    /// Set the path to the ONNX model file
    ///
    /// # Example
    /// ```no_run
    /// # use omega_sdk::OmegaBuilder;
    /// let builder = OmegaBuilder::new()
    ///     .model_path("/data/omega/model.onnx");
    /// ```
    pub fn model_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.model_path = Some(path.into());
        self
    }

    /// Set the storage path for persistent data
    ///
    /// # Example
    /// ```no_run
    /// # use omega_sdk::OmegaBuilder;
    /// let builder = OmegaBuilder::new()
    ///     .storage_path("/data/omega/brain.db");
    /// ```
    pub fn storage_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.storage_path = Some(path.into());
        self
    }

    /// Set the Constellation URL for pattern sync
    ///
    /// # Example
    /// ```no_run
    /// # use omega_sdk::OmegaBuilder;
    /// let builder = OmegaBuilder::new()
    ///     .constellation_url("https://constellation.omega.tv");
    /// ```
    pub fn constellation_url(mut self, url: impl Into<String>) -> Self {
        self.constellation_url = Some(url.into());
        self
    }

    /// Set the sync interval
    ///
    /// Recommended: 5-15 minutes to balance freshness and bandwidth
    ///
    /// # Example
    /// ```no_run
    /// # use omega_sdk::OmegaBuilder;
    /// # use std::time::Duration;
    /// let builder = OmegaBuilder::new()
    ///     .sync_interval(Duration::from_secs(600)); // 10 minutes
    /// ```
    pub fn sync_interval(mut self, interval: Duration) -> Self {
        self.sync_interval = interval;
        self
    }

    /// Set maximum number of patterns to store
    ///
    /// Default: 10,000 patterns (~50MB)
    ///
    /// # Example
    /// ```no_run
    /// # use omega_sdk::OmegaBuilder;
    /// let builder = OmegaBuilder::new()
    ///     .max_patterns(20_000);
    /// ```
    pub fn max_patterns(mut self, max: usize) -> Self {
        self.max_patterns = max;
        self
    }

    /// Set vector dimensions
    ///
    /// Default: 384 (for MiniLM model)
    /// Only change if using a different model
    ///
    /// # Example
    /// ```no_run
    /// # use omega_sdk::OmegaBuilder;
    /// let builder = OmegaBuilder::new()
    ///     .dimensions(768); // For larger models
    /// ```
    pub fn dimensions(mut self, dim: usize) -> Self {
        self.dimensions = dim;
        self
    }

    /// Build the OmegaBrain instance
    ///
    /// # Errors
    /// Returns an error if:
    /// - Model path is not set or invalid
    /// - Storage path is not set or invalid
    /// - Initialization fails
    ///
    /// # Example
    /// ```no_run
    /// # use omega_sdk::OmegaBuilder;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let brain = OmegaBuilder::new()
    ///     .model_path("/data/omega/model.onnx")
    ///     .storage_path("/data/omega/brain.db")
    ///     .constellation_url("https://constellation.omega.tv")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(self) -> Result<OmegaBrain> {
        // Validate required fields
        let model_path = self.model_path
            .ok_or_else(|| OmegaError::InitError("Model path is required".to_string()))?;

        let storage_path = self.storage_path
            .ok_or_else(|| OmegaError::InitError("Storage path is required".to_string()))?;

        // Validate paths exist
        if !model_path.exists() {
            return Err(OmegaError::InitError(
                format!("Model file not found: {}", model_path.display())
            ));
        }

        // Create storage directory if needed
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OmegaError::StorageError(format!("Failed to create storage directory: {}", e)))?;
        }

        OmegaBrain::init(BrainConfig {
            model_path,
            storage_path,
            constellation_url: self.constellation_url,
            sync_interval: self.sync_interval,
            max_patterns: self.max_patterns,
            dimensions: self.dimensions,
        }).await
    }
}

/// Configuration for OmegaBrain
#[derive(Debug, Clone)]
pub struct BrainConfig {
    pub model_path: PathBuf,
    pub storage_path: PathBuf,
    pub constellation_url: Option<String>,
    pub sync_interval: Duration,
    pub max_patterns: usize,
    pub dimensions: usize,
}

/// OmegaBrain - The TV-side intelligence agent
///
/// This is a placeholder structure. The actual implementation would include:
/// - RuVector for local vector database
/// - ONNX Runtime for embeddings
/// - AgentDB for memory systems
/// - Sync client for constellation communication
#[derive(Debug)]
pub struct OmegaBrain {
    config: BrainConfig,
    // TODO: Add actual fields when dependencies are integrated:
    // vectors: ruvector::VectorDB,
    // onnx: ort::Session,
    // memory: agentdb::AgentMemory,
    // sync: omega_sync::SyncClient,
}

impl OmegaBrain {
    /// Initialize the Omega Brain
    pub async fn init(config: BrainConfig) -> Result<Self> {
        // TODO: Actual initialization with RuVector, ONNX, AgentDB
        // For now, just validate config

        Ok(Self {
            config,
        })
    }

    /// Get recommendations based on viewing context
    ///
    /// Target: <15ms latency
    pub async fn recommend(&self, context: &str) -> Result<Vec<Recommendation>> {
        // TODO: Implement actual recommendation logic:
        // 1. Parse context JSON
        // 2. Embed context with ONNX (<10ms)
        // 3. Search RuVector for similar patterns (<1ms)
        // 4. Rank by success rate (<1ms)
        // 5. Return top recommendations

        Ok(vec![])
    }

    /// Observe a viewing event
    pub async fn observe(&mut self, event: &str) -> Result<()> {
        // TODO: Implement actual observation:
        // 1. Parse event JSON
        // 2. Embed event with ONNX
        // 3. Store in RuVector
        // 4. Update AgentDB memory

        Ok(())
    }

    /// Sync with constellation
    pub async fn sync(&mut self) -> Result<SyncResult> {
        // TODO: Implement actual sync:
        // 1. Prepare delta (changed patterns since last sync)
        // 2. Compress delta (~1KB)
        // 3. Send to constellation
        // 4. Receive global patterns (~5KB)
        // 5. Merge into local database

        Ok(SyncResult {
            patterns_pushed: 0,
            patterns_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
        })
    }

    /// Shutdown and persist state
    pub async fn shutdown(&mut self) -> Result<()> {
        // TODO: Graceful shutdown:
        // 1. Flush pending writes
        // 2. Close database connections
        // 3. Save state

        Ok(())
    }
}

/// Recommendation result
#[derive(Debug, Clone)]
pub struct Recommendation {
    pub content_id: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

/// Sync operation result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub patterns_pushed: usize,
    pub patterns_received: usize,
    pub bytes_sent: usize,
    pub bytes_received: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = OmegaBuilder::new();
        assert_eq!(builder.sync_interval, Duration::from_secs(300));
        assert_eq!(builder.max_patterns, 10_000);
        assert_eq!(builder.dimensions, 384);
    }

    #[test]
    fn test_builder_fluent_api() {
        let builder = OmegaBuilder::new()
            .model_path("/test/model.onnx")
            .storage_path("/test/storage")
            .constellation_url("https://test.com")
            .sync_interval(Duration::from_secs(600))
            .max_patterns(20_000)
            .dimensions(768);

        assert_eq!(builder.model_path, Some(PathBuf::from("/test/model.onnx")));
        assert_eq!(builder.sync_interval, Duration::from_secs(600));
        assert_eq!(builder.max_patterns, 20_000);
    }
}
