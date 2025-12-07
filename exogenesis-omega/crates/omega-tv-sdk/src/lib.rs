//! # Omega TV SDK
//!
//! C FFI bindings for TV manufacturers to integrate the Omega Brain recommendation system.
//!
//! ## Features
//! - Thread-safe global state management
//! - JSON-based data exchange
//! - Async runtime management
//! - Error handling with detailed messages
//!
//! ## Example Usage (C)
//! ```c
//! #include "omega_sdk.h"
//!
//! int main() {
//!     // Initialize
//!     if (omega_init("/data/omega", "https://constellation.example.com") != OMEGA_SUCCESS) {
//!         fprintf(stderr, "Init failed: %s\n", omega_get_last_error());
//!         return 1;
//!     }
//!
//!     // Get recommendations
//!     const char* context = "{\"genre\":\"action\",\"time\":\"evening\"}";
//!     char buffer[8192];
//!     if (omega_recommend(context, buffer, sizeof(buffer)) == OMEGA_SUCCESS) {
//!         printf("Recommendations: %s\n", buffer);
//!     }
//!
//!     // Cleanup
//!     omega_shutdown();
//!     return 0;
//! }
//! ```

pub mod error;
pub mod ffi;
pub mod runtime;

pub use error::{Error, Result};
pub use ffi::*;

use omega_tv_brain::{OmegaTVBrain, TVBrainConfig, ViewContext, ViewingEvent, Recommendation};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Internal SDK state
pub struct SdkState {
    pub brain: OmegaTVBrain,
    pub runtime: tokio::runtime::Runtime,
    pub last_error: Option<String>,
}

impl SdkState {
    /// Create new SDK state
    pub fn new(
        storage_path: impl Into<PathBuf>,
        constellation_url: impl Into<String>,
    ) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Init(format!("Failed to create runtime: {}", e)))?;

        let config = TVBrainConfig {
            dimension: 384, // MiniLM embedding size
            hnsw_m: 32,
            hnsw_ef: 100,
            max_patterns: 10_000,
            sync_interval_secs: 600, // 10 minutes
            storage_path: storage_path.into(),
            constellation_url: constellation_url.into(),
            device_id: uuid::Uuid::new_v4().to_string(),
            enable_neural_training: true,
            recommend_timeout_ms: 15, // 15ms target latency
        };

        let brain = runtime
            .block_on(OmegaTVBrain::init(config))
            .map_err(|e| Error::Init(format!("Failed to initialize brain: {}", e)))?;

        Ok(Self {
            brain,
            runtime,
            last_error: None,
        })
    }

    /// Get recommendations
    pub fn recommend(&self, context: &ViewContext) -> Result<Vec<Recommendation>> {
        self.runtime
            .block_on(self.brain.recommend(context))
            .map_err(|e| Error::Recommend(format!("Recommendation failed: {}", e)))
    }

    /// Record viewing event
    pub fn observe(&mut self, event: ViewingEvent) -> Result<()> {
        self.runtime
            .block_on(self.brain.observe(event))
            .map_err(|e| Error::Observe(format!("Observation failed: {}", e)))
    }

    /// Sync with constellation
    pub fn sync(&mut self) -> Result<()> {
        self.runtime
            .block_on(self.brain.sync())
            .map_err(|e| Error::Sync(format!("Sync failed: {}", e)))?;
        Ok(())
    }

    /// Set last error message
    pub fn set_error(&mut self, error: String) {
        self.last_error = Some(error);
    }

    /// Get last error message
    pub fn get_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sdk_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let result = SdkState::new(
            temp_dir.path().to_path_buf(),
            "http://localhost:8080",
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling() {
        let mut state = SdkState::new("/tmp/omega_test", "http://localhost:8080").unwrap();
        state.set_error("Test error".to_string());
        assert_eq!(state.get_error(), Some("Test error"));
    }
}
