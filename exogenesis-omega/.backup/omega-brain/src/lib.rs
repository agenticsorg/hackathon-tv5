pub mod config;
pub mod inference;
pub mod memory;
pub mod recommend;
pub mod vectors;

use anyhow::Result;
use omega_protocol::{
    GlobalPatterns, PatternDelta, Recommendation, ViewContext, ViewingEvent, ViewingPattern,
};
use std::time::Instant;

pub use config::BrainConfig;
use inference::InferenceEngine;
use memory::AgentMemory;
use recommend::{deduplicate, filter_by_context, rank_recommendations};
use vectors::VectorStore;

/// OmegaBrain - TV-side intelligence (~200MB footprint)
///
/// The OmegaBrain runs locally on each TV and provides:
/// - Sub-15ms recommendation latency
/// - Privacy-preserving pattern learning
/// - Offline-capable operation
/// - Minimal sync bandwidth (<6KB per sync)
pub struct OmegaBrain {
    /// Local vector database
    vectors: VectorStore,

    /// ONNX inference engine for embeddings
    inference: InferenceEngine,

    /// AgentDB memory systems (Reflexion, Skills, Reasoning)
    memory: AgentMemory,

    /// Configuration
    config: BrainConfig,
}

impl OmegaBrain {
    /// Initialize OmegaBrain on TV boot
    ///
    /// This loads the ONNX model, initializes the vector store,
    /// and restores memory from persistent storage.
    ///
    /// # Arguments
    /// * `config` - Brain configuration
    ///
    /// # Returns
    /// * `Result<Self>` - Initialized brain or error
    pub async fn init(config: BrainConfig) -> Result<Self> {
        tracing::info!("Initializing OmegaBrain...");
        let start = Instant::now();

        // Initialize vector store
        let vectors = VectorStore::new(config.dimensions, config.max_patterns)?;
        tracing::debug!(
            "Vector store initialized ({} dims, {} max patterns)",
            config.dimensions,
            config.max_patterns
        );

        // Initialize ONNX inference engine
        let inference = InferenceEngine::new(&config.model_path, config.dimensions)?;
        tracing::debug!("Inference engine initialized");

        // Load or create memory
        let memory = AgentMemory::load_or_create(&config.storage_path)?;
        tracing::debug!("Memory systems loaded");

        tracing::info!(
            "OmegaBrain initialized in {}ms",
            start.elapsed().as_millis()
        );

        Ok(Self {
            vectors,
            inference,
            memory,
            config,
        })
    }

    /// Get recommendations for current context
    ///
    /// **Target: <15ms total**
    ///
    /// # Arguments
    /// * `context` - Current viewing context (time, mood, recent watches)
    ///
    /// # Returns
    /// * `Vec<Recommendation>` - Top recommendations ranked by relevance
    ///
    /// # Performance Breakdown
    /// - Embed context: <10ms (ONNX inference)
    /// - Search similar: <1ms (vector search)
    /// - Rank & filter: <1ms
    /// - Total: <15ms
    pub fn recommend(&self, context: &ViewContext) -> Vec<Recommendation> {
        let start = Instant::now();

        // 1. Embed context (<10ms)
        let query_embedding = match self.embed_context(context) {
            Ok(emb) => emb,
            Err(e) => {
                tracing::error!("Failed to embed context: {}", e);
                return vec![];
            }
        };

        let embed_time = start.elapsed();

        // 2. Search similar patterns (<1ms)
        let similar = self.vectors.search(&query_embedding, 50);
        let search_time = start.elapsed();

        // 3. Rank by success rate (<1ms)
        let ranked = rank_recommendations(similar, context);

        // 4. Filter by context
        let filtered = filter_by_context(ranked, context);

        // 5. Deduplicate
        let deduped = deduplicate(filtered);

        let total_time = start.elapsed();

        // Log performance metrics
        metrics::histogram!("omega_brain_recommend_ms").record(total_time.as_secs_f64() * 1000.0);
        metrics::histogram!("omega_brain_embed_ms").record(embed_time.as_secs_f64() * 1000.0);
        metrics::histogram!("omega_brain_search_ms")
            .record((search_time - embed_time).as_secs_f64() * 1000.0);

        tracing::debug!(
            "Recommendations generated in {}ms (embed: {}ms, search: {}ms)",
            total_time.as_millis(),
            embed_time.as_millis(),
            (search_time - embed_time).as_millis()
        );

        // Return top 20 recommendations
        deduped.into_iter().take(20).collect()
    }

    /// Observe viewing event (LOCAL)
    ///
    /// Records a viewing event, updates local patterns, and stores in memory.
    /// No network operations - purely local.
    ///
    /// # Arguments
    /// * `event` - Viewing event to record
    pub fn observe(&mut self, event: ViewingEvent) {
        let start = Instant::now();

        // 1. Embed event
        let embedding = match self.embed_event(&event) {
            Ok(emb) => emb,
            Err(e) => {
                tracing::error!("Failed to embed event: {}", e);
                return;
            }
        };

        // 2. Create pattern from event
        let pattern = ViewingPattern::from_event(event.clone(), embedding.clone());

        // 3. Store in vector database
        let metadata = serde_json::json!({
            "title": event.content_title,
            "success_rate": pattern.success_rate,
            "watch_pct": event.watch_percentage,
            "timestamp": event.timestamp,
        });

        if let Err(e) = self
            .vectors
            .insert(pattern.id.clone(), embedding, Some(metadata))
        {
            tracing::error!("Failed to insert vector: {}", e);
            return;
        }

        // 4. Update memory systems
        self.memory.record(pattern);

        metrics::counter!("omega_brain_observations").increment(1);

        tracing::debug!(
            "Event observed in {}ms (total patterns: {})",
            start.elapsed().as_millis(),
            self.vectors.count()
        );
    }

    /// Prepare delta for constellation sync
    ///
    /// Extracts high-quality patterns that have changed since the last sync.
    /// Target payload: ~1KB compressed.
    ///
    /// # Arguments
    /// * `since_version` - Version number of last successful sync
    ///
    /// # Returns
    /// * `PatternDelta` - Delta containing added/updated/removed patterns
    pub fn prepare_sync_delta(&self, since_version: u64) -> PatternDelta {
        self.memory.get_changes_since(since_version)
    }

    /// Apply patterns received from constellation
    ///
    /// Merges global patterns into local memory to benefit from
    /// collective intelligence across all TVs.
    ///
    /// # Arguments
    /// * `patterns` - Global patterns from constellation
    pub fn apply_global_patterns(&mut self, patterns: &GlobalPatterns) {
        tracing::debug!(
            "Applying {} global patterns",
            patterns.trending.len() + patterns.similar.len()
        );

        for pattern in &patterns.trending {
            self.memory.merge_global_pattern(pattern);
        }

        for pattern in &patterns.similar {
            self.memory.merge_global_pattern(pattern);
        }

        metrics::counter!("omega_brain_global_patterns_applied")
            .increment((patterns.trending.len() + patterns.similar.len()) as u64);
    }

    /// Get current brain statistics
    pub fn stats(&self) -> BrainStats {
        BrainStats {
            total_patterns: self.vectors.count(),
            memory_version: self.memory.version(),
            episode_count: self.memory.episode_count(),
            skill_count: self.memory.skill_count(),
        }
    }

    // Private helper methods

    fn embed_context(&self, context: &ViewContext) -> Result<Vec<f32>> {
        let text = context.to_embedding_text();
        self.inference.embed_text(&text)
    }

    fn embed_event(&self, event: &ViewingEvent) -> Result<Vec<f32>> {
        let text = event.to_embedding_text();
        self.inference.embed_text(&text)
    }
}

/// Statistics about the brain state
#[derive(Debug, Clone)]
pub struct BrainStats {
    pub total_patterns: usize,
    pub memory_version: u64,
    pub episode_count: usize,
    pub skill_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_config() -> BrainConfig {
        BrainConfig {
            dimensions: 128,
            max_patterns: 1000,
            sync_interval_secs: 300,
            model_path: PathBuf::from("/tmp/test_model.onnx"),
            storage_path: PathBuf::from("/tmp/test_brain.db"),
        }
    }

    #[tokio::test]
    async fn test_brain_init() {
        let config = test_config();
        let brain = OmegaBrain::init(config).await;

        assert!(brain.is_ok());
    }

    #[tokio::test]
    async fn test_recommend() {
        let config = test_config();
        let brain = OmegaBrain::init(config).await.unwrap();

        // Create some viewing events
        let context = ViewContext::default();

        let start = Instant::now();
        let recommendations = brain.recommend(&context);
        let latency = start.elapsed();

        // Initially empty, so no recommendations
        assert_eq!(recommendations.len(), 0);

        // But should be fast
        assert!(
            latency.as_millis() < 100,
            "Recommendation took {}ms",
            latency.as_millis()
        );
    }

    #[tokio::test]
    async fn test_observe_and_recommend() {
        let config = test_config();
        let mut brain = OmegaBrain::init(config).await.unwrap();

        // Observe some events
        for i in 0..10 {
            let event = ViewingEvent::new(
                format!("content-{}", i),
                format!("Movie {}", i),
                0.85 + i as f32 * 0.01,
                5400,
            );
            brain.observe(event);
        }

        let stats = brain.stats();
        assert_eq!(stats.total_patterns, 10);

        // Now get recommendations
        let context = ViewContext::default();
        let recommendations = brain.recommend(&context);

        assert!(recommendations.len() > 0);
        assert!(recommendations.len() <= 10);
    }

    #[tokio::test]
    async fn test_sync_delta() {
        let config = test_config();
        let mut brain = OmegaBrain::init(config).await.unwrap();

        // Observe some events
        for i in 0..5 {
            let event = ViewingEvent::new(
                format!("content-{}", i),
                format!("Movie {}", i),
                0.9,
                5400,
            );
            brain.observe(event);
        }

        // Get delta
        let delta = brain.prepare_sync_delta(0);

        assert!(delta.patterns_added.len() > 0);
        assert_eq!(delta.local_version, 5);
    }
}
