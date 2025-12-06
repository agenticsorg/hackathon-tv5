//! # Omega TV Brain
//!
//! TV-side integration layer using omega-* crates from crates.io for intelligent recommendations.
//!
//! This crate integrates:
//! - `omega-agentdb` - SIMD-accelerated vector DB with HNSW indexing
//! - `omega-memory` - 12-tier cosmic memory system
//! - `omega-loops` - 7 temporal feedback loops
//! - `omega-persistence` - SQLite storage with ACID guarantees
//! - `omega-runtime` - Production orchestration
//!
//! ## Architecture
//!
//! ```text
//! OmegaTVBrain
//! ├── AgentDB (vector search, reflexion, skills)
//! ├── CosmicMemory (12-tier memory)
//! ├── LoopEngine (7 temporal loops)
//! ├── PersistenceManager (SQLite)
//! ├── RuntimeOrchestrator (health monitoring)
//! └── SyncClient (constellation sync)
//! ```

use std::path::PathBuf;
use tracing::{debug, info, warn};

// Re-exports from omega-* crates
pub use omega_agentdb::{AgentDB, AgentDBConfig, ReflexionEpisode, Skill, VectorResult};
pub use omega_core::{Intelligence, Architecture, Paradigm};
pub use omega_loops::{LoopEngine, LoopType, CycleInput, CycleOutput};
pub use omega_memory::{CosmicMemory, Memory, MemoryTier, MemoryContent, Query};
pub use omega_persistence::PersistenceManager;
pub use omega_runtime::RuntimeOrchestrator;

// Internal modules
pub mod config;
pub mod types;
pub mod embed;
pub mod recommend;
pub mod observe;
pub mod sync;

pub use config::TVBrainConfig;
pub use types::{ViewContext, ViewingEvent, Recommendation, SyncResult};
use embed::EmbeddingEngine;
use recommend::RecommendationEngine;
use observe::EventObserver;
use sync::SyncClient;

/// Main TV Brain integrating all omega-* crates
pub struct OmegaTVBrain {
    /// SIMD-accelerated vector DB (omega-agentdb)
    /// - 13-41x SIMD acceleration
    /// - HNSW indexing
    /// - ReflexionEpisode, CausalEdge, Skill storage
    agentdb: AgentDB,

    /// 12-tier cosmic memory (omega-memory)
    /// - Instant (1ms) to Omega (universe-age)
    /// - Automatic consolidation
    /// - Semantic retrieval
    memory: CosmicMemory,

    /// 7 temporal loops (omega-loops)
    /// - Reflexive (100ms) to Transcendent (10y)
    /// - Multi-scale learning
    loops: LoopEngine,

    /// SQLite persistence (omega-persistence)
    persistence: PersistenceManager,

    /// Production orchestration (omega-runtime)
    runtime: RuntimeOrchestrator,

    /// Sync client for constellation
    sync_client: SyncClient,

    /// Embedding engine (mock for now, ONNX later)
    embedder: EmbeddingEngine,

    /// Recommendation engine
    recommender: RecommendationEngine,

    /// Event observer
    observer: EventObserver,

    /// Configuration
    config: TVBrainConfig,
}

impl OmegaTVBrain {
    /// Initialize the TV Brain with all subsystems
    pub async fn init(config: TVBrainConfig) -> anyhow::Result<Self> {
        info!("Initializing Omega TV Brain with config: {:?}", config);

        // Validate configuration
        config.validate()?;

        // Initialize AgentDB with 384-dim embeddings (MiniLM)
        info!("Initializing AgentDB (dimension={})", config.dimension);
        let agentdb = AgentDB::new(AgentDBConfig {
            dimension: config.dimension,
            hnsw_m: config.hnsw_m,
            hnsw_ef: config.hnsw_ef,
            cache_size: config.max_patterns,
        })
        .await?;

        // Initialize cosmic memory (12 tiers)
        info!("Initializing CosmicMemory (12 tiers)");
        let memory = CosmicMemory::new().await?;

        // Initialize temporal loops (7 loops)
        info!("Initializing LoopEngine (7 temporal loops)");
        let mut loops = LoopEngine::new();
        loops.initialize().await?;

        // Initialize SQLite persistence
        info!("Initializing PersistenceManager (path: {:?})", config.storage_path);
        std::fs::create_dir_all(&config.storage_path)?;
        let persistence = PersistenceManager::new(&config.storage_path).await?;

        // Initialize runtime orchestrator
        info!("Initializing RuntimeOrchestrator");
        let runtime = RuntimeOrchestrator::new().await?;

        // Initialize sync client
        info!("Initializing SyncClient (url: {})", config.constellation_url);
        let sync_client = SyncClient::new(&config.constellation_url, config.device_id.clone())?;

        // Initialize embedding engine
        let embedder = EmbeddingEngine::new(config.dimension);

        // Initialize recommendation engine
        let recommender = RecommendationEngine::new();

        // Initialize event observer
        let observer = EventObserver::new();

        info!("Omega TV Brain initialized successfully");

        Ok(Self {
            agentdb,
            memory,
            loops,
            persistence,
            runtime,
            sync_client,
            embedder,
            recommender,
            observer,
            config,
        })
    }

    /// Get recommendations (<15ms target)
    ///
    /// Flow:
    /// 1. Create embedding from context
    /// 2. Search AgentDB vectors (<1ms with SIMD)
    /// 3. Filter by context using memory recall
    /// 4. Rank and return recommendations
    pub async fn recommend(&self, context: &ViewContext) -> anyhow::Result<Vec<Recommendation>> {
        debug!("Getting recommendations for context: {}", context);

        let start = std::time::Instant::now();

        // 1. Create embedding from context
        let query_embedding = self.embedder.embed_context(context);

        // 2. Search AgentDB vectors (<1ms with SIMD)
        let results = self.agentdb.vector_search(&query_embedding, 50).await?;

        debug!("Vector search returned {} results in {:?}", results.len(), start.elapsed());

        // 3. Filter by context using memory recall
        let query = Query::semantic(&context.to_string());
        let memories = self
            .memory
            .recall(&query, &[MemoryTier::Semantic, MemoryTier::Episodic])
            .await
            .unwrap_or_default();

        debug!("Memory recall returned {} memories", memories.len());

        // 4. Rank and return recommendations
        let recommendations = self.recommender.rank_results(results, memories, context);

        let elapsed = start.elapsed();
        info!(
            "Generated {} recommendations in {:?} (<15ms target)",
            recommendations.len(),
            elapsed
        );

        if elapsed.as_millis() > self.config.recommend_timeout_ms as u128 {
            warn!(
                "Recommendation latency {}ms exceeded target {}ms",
                elapsed.as_millis(),
                self.config.recommend_timeout_ms
            );
        }

        Ok(recommendations)
    }

    /// Record viewing event
    ///
    /// Flow:
    /// 1. Create embedding
    /// 2. Store in AgentDB as ReflexionEpisode
    /// 3. Store in cosmic memory (Episodic tier)
    /// 4. Execute Reflexive loop (100ms feedback)
    pub async fn observe(&mut self, event: ViewingEvent) -> anyhow::Result<()> {
        info!(
            "Observing event: content={} watch={:.2}% engagement={:.2}",
            event.content_id,
            event.watch_percentage * 100.0,
            event.engagement_score
        );

        // 1. Create embedding
        let embedding = self.embedder.embed_event(&event);

        // 2. Store in AgentDB as ReflexionEpisode
        let episode = self.observer.to_reflexion_episode(&event, embedding.clone());
        self.agentdb.reflexion_store(episode).await?;

        // 3. Store in cosmic memory (Episodic tier)
        let memory = self.observer.to_memory(&event, embedding);
        self.memory.store(memory).await?;

        // 4. Execute Reflexive loop (100ms feedback)
        if self.config.enable_neural_training {
            let input = self.observer.to_cycle_input(&event);
            match self.loops.execute_cycle(LoopType::Reflexive, input).await {
                Ok(_) => debug!("Reflexive loop executed successfully"),
                Err(e) => warn!("Reflexive loop execution failed: {}", e),
            }
        }

        info!("Event observation completed");
        Ok(())
    }

    /// Sync with constellation
    ///
    /// Flow:
    /// 1. Get high-quality patterns from AgentDB (quality ≥ 0.7)
    /// 2. Create compressed delta (~1KB)
    /// 3. Send to constellation, receive global patterns (~5KB)
    /// 4. Apply global patterns to AgentDB and memory
    pub async fn sync(&mut self) -> anyhow::Result<SyncResult> {
        info!("Starting sync with constellation");

        // 1. Get high-quality patterns from AgentDB
        let skills = self.agentdb.skill_list(1000).await?;
        let high_quality: Vec<_> = skills
            .into_iter()
            .filter(|s| s.success_rate >= 0.7)
            .collect();

        info!("Found {} high-quality patterns to sync", high_quality.len());

        // 2-3. Sync cycle (prepare delta, send, receive global)
        let result = self.sync_client.sync_cycle(&high_quality).await?;

        // 4. Apply global patterns (second call to get patterns for storage)
        let delta = self.sync_client.prepare_delta(&high_quality)?;
        let global = self.sync_client.sync(delta).await?;

        for pattern in global.patterns {
            // Store in AgentDB as Skill
            let skill = Skill {
                id: None,
                name: pattern.name.clone(),
                description: pattern.description.clone(),
                embedding: pattern.embedding.clone(),
                usage_count: pattern.usage_count,
                success_rate: pattern.success_rate,
                created_at: chrono::Utc::now(),
            };

            match self.agentdb.skill_store(skill).await {
                Ok(_) => debug!("Stored global pattern: {}", pattern.name),
                Err(e) => warn!("Failed to store pattern {}: {}", pattern.name, e),
            }

            // Store in CosmicMemory (Collective tier)
            let memory = Memory::new(
                MemoryTier::Collective,
                MemoryContent::MultiModal {
                    text: Some(pattern.description.clone()),
                    embedding: pattern.embedding.clone(),
                    metadata: pattern.metadata.clone(),
                },
                pattern.embedding,
                pattern.success_rate,
            );

            match self.memory.store(memory).await {
                Ok(_) => debug!("Stored global pattern in memory: {}", pattern.name),
                Err(e) => warn!("Failed to store pattern in memory {}: {}", pattern.name, e),
            }
        }

        info!(
            "Sync completed: pushed {} patterns, received {} global patterns",
            result.patterns_pushed,
            result.patterns_received
        );

        Ok(result)
    }

    /// Get runtime health status
    pub async fn health(&self) -> anyhow::Result<serde_json::Value> {
        let health = self.runtime.health_check().await?;
        Ok(serde_json::to_value(health)?)
    }

    /// Shutdown gracefully
    pub async fn shutdown(self) -> anyhow::Result<()> {
        info!("Shutting down Omega TV Brain");

        // Shutdown loops
        self.loops.shutdown().await?;

        // Shutdown runtime
        self.runtime.shutdown().await?;

        info!("Shutdown complete");
        Ok(())
    }

    /// Get the AgentDB instance
    pub fn agentdb(&self) -> &AgentDB {
        &self.agentdb
    }

    /// Get the CosmicMemory instance
    pub fn memory(&self) -> &CosmicMemory {
        &self.memory
    }

    /// Get the LoopEngine instance
    pub fn loops(&self) -> &LoopEngine {
        &self.loops
    }

    /// Get the PersistenceManager instance
    pub fn persistence(&self) -> &PersistenceManager {
        &self.persistence
    }

    /// Get the RuntimeOrchestrator instance
    pub fn runtime(&self) -> &RuntimeOrchestrator {
        &self.runtime
    }

    /// Get the configuration
    pub fn config(&self) -> &TVBrainConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_default() {
        let config = TVBrainConfig::default();
        assert_eq!(config.dimension, 384);
        assert_eq!(config.hnsw_m, 32);
        assert_eq!(config.hnsw_ef, 100);
        assert_eq!(config.max_patterns, 10_000);
    }

    #[tokio::test]
    async fn test_init() {
        let config = TVBrainConfig::development();
        let brain = OmegaTVBrain::init(config).await;
        assert!(brain.is_ok());
    }

    #[tokio::test]
    async fn test_recommend() {
        let config = TVBrainConfig::development();
        let brain = OmegaTVBrain::init(config).await.unwrap();

        let context = ViewContext {
            user_id: "test-user".to_string(),
            device_id: "test-device".to_string(),
            time_of_day: 20,
            day_of_week: 5,
            current_genre: Some("action".to_string()),
            previous_content: vec![],
            session_duration_mins: 30,
        };

        let recommendations = brain.recommend(&context).await;
        assert!(recommendations.is_ok());
    }
}
