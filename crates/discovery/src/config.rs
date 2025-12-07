use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Discovery Service Configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    /// HTTP server configuration
    pub server: ServerConfig,

    /// Search configuration
    pub search: SearchConfig,

    /// Vector search configuration
    pub vector: VectorConfig,

    /// Keyword search configuration
    pub keyword: KeywordConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Embedding API configuration
    pub embedding: EmbeddingConfig,

    /// Personalization configuration
    #[serde(default)]
    pub personalization: PersonalizationConfig,
}

/// Personalization configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PersonalizationConfig {
    /// SONA service URL
    pub sona_url: String,

    /// Personalization weight in final score (0.0-1.0)
    pub boost_weight: f32,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,

    /// Cache TTL for user preferences (seconds)
    pub cache_ttl_sec: u64,

    /// Enable/disable personalization
    pub enabled: bool,
}

impl Default for PersonalizationConfig {
    fn default() -> Self {
        Self {
            sona_url: "http://localhost:8082".to_string(),
            boost_weight: 0.25,
            timeout_ms: 50,
            cache_ttl_sec: 300,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Server host
    pub host: String,

    /// Server port (default: 8081)
    pub port: u16,

    /// Worker threads
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchConfig {
    /// Maximum candidates to retrieve per strategy
    pub max_candidates: usize,

    /// Default page size
    pub page_size: usize,

    /// Maximum page size
    pub max_page_size: usize,

    /// Strategy timeout in milliseconds
    pub strategy_timeout_ms: u64,

    /// Total search timeout in milliseconds
    pub total_timeout_ms: u64,

    /// Reciprocal Rank Fusion K parameter
    pub rrf_k: f32,

    /// Search strategy weights
    pub weights: StrategyWeights,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StrategyWeights {
    /// Vector search weight (default: 0.35)
    pub vector: f32,

    /// Graph search weight (default: 0.30)
    pub graph: f32,

    /// Keyword search weight (default: 0.20)
    pub keyword: f32,

    /// Popularity weight (default: 0.15)
    pub popularity: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VectorConfig {
    /// Qdrant server URL
    pub qdrant_url: String,

    /// Collection name
    pub collection_name: String,

    /// Embedding dimension (default: 768)
    pub dimension: usize,

    /// HNSW ef_search parameter
    pub ef_search: usize,

    /// Top-K results to retrieve
    pub top_k: usize,

    /// Similarity threshold
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeywordConfig {
    /// Tantivy index path
    pub index_path: String,

    /// Top-K results
    pub top_k: usize,

    /// Minimum score threshold
    pub min_score: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,

    /// Connection pool size
    pub max_connections: u32,

    /// Connection timeout
    pub connect_timeout_sec: u64,

    /// Query timeout
    pub query_timeout_sec: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Redis connection URL
    pub redis_url: String,

    /// Cache TTL for search results (seconds)
    pub search_ttl_sec: u64,

    /// Cache TTL for embeddings (seconds)
    pub embedding_ttl_sec: u64,

    /// Cache TTL for intent parsing (seconds)
    pub intent_ttl_sec: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingConfig {
    /// Embedding model name
    pub model: String,

    /// API endpoint
    pub api_url: String,

    /// API key
    pub api_key: String,

    /// Request timeout
    pub timeout_ms: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8081,
                workers: None,
            },
            search: SearchConfig {
                max_candidates: 1000,
                page_size: 20,
                max_page_size: 100,
                strategy_timeout_ms: 300,
                total_timeout_ms: 450,
                rrf_k: 60.0,
                weights: StrategyWeights {
                    vector: 0.35,
                    graph: 0.30,
                    keyword: 0.20,
                    popularity: 0.15,
                },
            },
            vector: VectorConfig {
                qdrant_url: "http://localhost:6333".to_string(),
                collection_name: "media_embeddings".to_string(),
                dimension: 768,
                ef_search: 64,
                top_k: 50,
                similarity_threshold: 0.7,
            },
            keyword: KeywordConfig {
                index_path: "./data/tantivy".to_string(),
                top_k: 50,
                min_score: 0.5,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/media_gateway".to_string(),
                max_connections: 10,
                connect_timeout_sec: 10,
                query_timeout_sec: 5,
            },
            cache: CacheConfig {
                redis_url: "redis://localhost:6379".to_string(),
                search_ttl_sec: 1800,    // 30 minutes
                embedding_ttl_sec: 3600, // 1 hour
                intent_ttl_sec: 600,     // 10 minutes
            },
            embedding: EmbeddingConfig {
                model: "text-embedding-3-small".to_string(),
                api_url: "https://api.openai.com/v1/embeddings".to_string(),
                api_key: String::new(),
                timeout_ms: 5000,
            },
            personalization: PersonalizationConfig::default(),
        }
    }
}

impl DiscoveryConfig {
    /// Load configuration from environment and config file
    pub fn load() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config/discovery").required(false))
            .add_source(config::Environment::with_prefix("DISCOVERY"))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    /// Get strategy timeout as Duration
    pub fn strategy_timeout(&self) -> Duration {
        Duration::from_millis(self.search.strategy_timeout_ms)
    }

    /// Get total timeout as Duration
    pub fn total_timeout(&self) -> Duration {
        Duration::from_millis(self.search.total_timeout_ms)
    }
}
