//! SONA (Self-Optimizing Neural Architecture) Personalization Engine
//!
//! This module implements the personalization layer for Media Gateway,
//! providing user embeddings, LoRA adaptation, and hybrid recommendations.

pub mod ab_testing;
pub mod cold_start;
pub mod collaborative;
pub mod content_based;
pub mod context;
pub mod diversity;
pub mod experiment_repository;
pub mod graph;
pub mod inference;
pub mod lora;
pub mod lora_storage;
pub mod matrix_factorization;
pub mod profile;
pub mod recommendation;
pub mod types;

// Re-export key types
pub use ab_testing::{
    ABTestingService, Assignment, Experiment, ExperimentMetrics, ExperimentStatus, Variant,
    VariantMetrics,
};
pub use cold_start::HandleColdStartUser;
pub use collaborative::{CollaborativeFilteringEngine, Interaction, InteractionType};
pub use context::ContextAwareFilter;
pub use diversity::ApplyDiversityFilter;
pub use experiment_repository::{ExperimentRepository, PostgresExperimentRepository};
pub use inference::ONNXInference;
pub use lora::{ComputeLoRAForward, UpdateUserLoRA, UserLoRAAdapter};
pub use lora_storage::{LoRAAdapterMetadata, LoRAStorage, StorageStats};
pub use matrix_factorization::{ALSConfig, MatrixFactorization, SparseMatrix};
pub use profile::{BuildUserPreferenceVector, UserProfile};
pub use recommendation::GenerateRecommendations;
pub use types::*;

use anyhow::Result;
use std::sync::Arc;

/// SONA engine configuration
#[derive(Debug, Clone)]
pub struct SonaConfig {
    /// Embedding dimensionality (default: 512)
    pub embedding_dim: usize,
    /// LoRA rank (default: 8)
    pub lora_rank: usize,
    /// LoRA alpha scaling (default: 16)
    pub lora_alpha: f32,
    /// Temporal decay rate (default: 0.95)
    pub decay_rate: f32,
    /// Minimum watch threshold (default: 0.3)
    pub min_watch_threshold: f32,
    /// Minimum interactions before LoRA training (default: 10)
    pub min_training_events: usize,
    /// ONNX model path (default: from env SONA_MODEL_PATH)
    pub model_path: Option<String>,
}

impl Default for SonaConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 512,
            lora_rank: 8,
            lora_alpha: 16.0,
            decay_rate: 0.95,
            min_watch_threshold: 0.3,
            min_training_events: 10,
            model_path: None,
        }
    }
}

/// SONA engine instance
pub struct SonaEngine {
    config: SonaConfig,
    inference: Option<Arc<ONNXInference>>,
}

impl SonaEngine {
    pub fn new(config: SonaConfig) -> Self {
        Self {
            config,
            inference: None,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(SonaConfig::default())
    }

    /// Initialize with ONNX inference engine
    pub fn with_inference(mut self, inference: ONNXInference) -> Self {
        self.inference = Some(Arc::new(inference));
        self
    }

    /// Get or create inference engine
    pub fn inference(&self) -> Result<Arc<ONNXInference>> {
        if let Some(ref inference) = self.inference {
            return Ok(Arc::clone(inference));
        }

        // Create from config or env
        let inference = if let Some(ref path) = self.config.model_path {
            ONNXInference::new(path, self.config.embedding_dim)?
        } else {
            ONNXInference::from_env()?
        };

        Ok(Arc::new(inference))
    }

    pub fn config(&self) -> &SonaConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_sona_engine_creation() {
        let engine = SonaEngine::with_default_config();
        assert_eq!(engine.config().embedding_dim, 512);
        assert_eq!(engine.config().lora_rank, 8);
    }
}
