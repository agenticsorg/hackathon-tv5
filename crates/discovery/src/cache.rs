//! Redis caching layer for search results, intent parsing, and embeddings
//!
//! This module provides a production-ready caching implementation with:
//! - Connection pooling for Redis operations
//! - TTL-based expiration for different data types
//! - SHA256-based cache key generation
//! - JSON serialization for complex types
//! - Comprehensive metrics and tracing
//! - Graceful error handling

use anyhow::{Context, Result};
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

use crate::config::CacheConfig;

/// Cache key prefixes for different data types
const PREFIX_SEARCH: &str = "search";
const PREFIX_INTENT: &str = "intent";
const PREFIX_EMBEDDING: &str = "embedding";

/// Redis cache implementation with connection pooling
///
/// The cache supports different TTL strategies for different data types:
/// - Search results: 30 minutes (frequently updated)
/// - Intent parsing: 10 minutes (moderate volatility)
/// - Embeddings: 1 hour (stable vectors)
#[derive(Clone)]
pub struct RedisCache {
    /// Connection manager for async Redis operations
    manager: ConnectionManager,
    /// Cache configuration with TTL settings
    config: Arc<CacheConfig>,
}

/// Error types for cache operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    Connection(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache key generation failed: {0}")]
    KeyGeneration(String),

    #[error("Cache operation failed: {0}")]
    Operation(String),
}

impl RedisCache {
    /// Create a new Redis cache with connection pool
    ///
    /// # Arguments
    /// * `config` - Cache configuration with Redis URL and TTL settings
    ///
    /// # Errors
    /// Returns error if Redis connection cannot be established
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use media_gateway_discovery::cache::RedisCache;
    /// use media_gateway_discovery::config::CacheConfig;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = Arc::new(CacheConfig {
    ///     redis_url: "redis://localhost:6379".to_string(),
    ///     search_ttl_sec: 1800,
    ///     embedding_ttl_sec: 3600,
    ///     intent_ttl_sec: 600,
    /// });
    ///
    /// let cache = RedisCache::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(config), fields(redis_url = %config.redis_url))]
    pub async fn new(config: Arc<CacheConfig>) -> Result<Self> {
        info!("Initializing Redis cache connection pool");

        let client =
            Client::open(config.redis_url.as_str()).context("Failed to create Redis client")?;

        let manager = ConnectionManager::new(client)
            .await
            .context("Failed to create Redis connection manager")?;

        // Test connection
        let mut conn = manager.clone();
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .context("Redis ping failed")?;

        info!("Redis cache initialized successfully");

        Ok(Self { manager, config })
    }

    /// Generate cache key using SHA256 hash
    ///
    /// # Arguments
    /// * `prefix` - Cache key prefix (search, intent, embedding)
    /// * `data` - Serializable data to hash
    ///
    /// # Returns
    /// Cache key in format: `{prefix}:{sha256_hash}`
    ///
    /// # Example
    /// ```
    /// # use media_gateway_discovery::cache::RedisCache;
    /// # use serde::Serialize;
    /// #[derive(Serialize)]
    /// struct Query { text: String }
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let query = Query { text: "search term".to_string() };
    /// let key = RedisCache::generate_key("search", &query)?;
    /// assert!(key.starts_with("search:"));
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(data))]
    pub fn generate_key<T: Serialize>(prefix: &str, data: &T) -> Result<String, CacheError> {
        let json = serde_json::to_string(data).map_err(CacheError::Serialization)?;

        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);

        let key = format!("{}:{}", prefix, hash_hex);
        debug!(key = %key, prefix = %prefix, "Generated cache key");

        Ok(key)
    }

    /// Get value from cache
    ///
    /// # Type Parameters
    /// * `T` - Type to deserialize from cache (must implement DeserializeOwned)
    ///
    /// # Arguments
    /// * `key` - Cache key
    ///
    /// # Returns
    /// * `Ok(Some(T))` - Value found in cache
    /// * `Ok(None)` - Cache miss
    /// * `Err` - Cache operation failed
    #[instrument(skip(self), fields(key = %key))]
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let mut conn = self.manager.clone();

        let value: Option<String> = conn.get(key).await.map_err(CacheError::Connection)?;

        match value {
            Some(json) => {
                debug!(key = %key, "Cache hit");
                let data = serde_json::from_str(&json).map_err(CacheError::Serialization)?;
                Ok(Some(data))
            }
            None => {
                debug!(key = %key, "Cache miss");
                Ok(None)
            }
        }
    }

    /// Set value in cache with TTL
    ///
    /// # Type Parameters
    /// * `T` - Type to serialize to cache (must implement Serialize)
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Value to cache
    /// * `ttl_sec` - Time-to-live in seconds
    ///
    /// # Errors
    /// Returns error if serialization or Redis operation fails
    #[instrument(skip(self, value), fields(key = %key, ttl = %ttl_sec))]
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_sec: u64,
    ) -> Result<(), CacheError> {
        let json = serde_json::to_string(value).map_err(CacheError::Serialization)?;

        let mut conn = self.manager.clone();

        conn.set_ex::<_, _, ()>(key, json, ttl_sec)
            .await
            .map_err(CacheError::Connection)?;

        debug!(key = %key, ttl = %ttl_sec, "Cache set");
        Ok(())
    }

    /// Delete value from cache
    ///
    /// # Arguments
    /// * `key` - Cache key to delete
    ///
    /// # Returns
    /// Number of keys deleted (0 or 1)
    #[instrument(skip(self), fields(key = %key))]
    pub async fn delete(&self, key: &str) -> Result<u64, CacheError> {
        let mut conn = self.manager.clone();

        let count: u64 = conn.del::<_, u64>(key).await.map_err(CacheError::Connection)?;

        debug!(key = %key, deleted = %count, "Cache delete");
        Ok(count)
    }

    /// Delete multiple keys matching a pattern
    ///
    /// # Arguments
    /// * `pattern` - Redis key pattern (e.g., "search:*")
    ///
    /// # Returns
    /// Number of keys deleted
    #[instrument(skip(self), fields(pattern = %pattern))]
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64, CacheError> {
        let mut conn = self.manager.clone();

        // Get keys matching pattern
        let keys: Vec<String> = conn.keys(pattern).await.map_err(CacheError::Connection)?;

        if keys.is_empty() {
            debug!(pattern = %pattern, "No keys found matching pattern");
            return Ok(0);
        }

        // Delete all matching keys
        let count: u64 = conn.del::<_, u64>(&keys).await.map_err(CacheError::Connection)?;

        info!(pattern = %pattern, deleted = %count, "Deleted keys by pattern");
        Ok(count)
    }

    /// Cache search results with 30-minute TTL
    ///
    /// # Type Parameters
    /// * `Q` - Query type (must be serializable for key generation)
    /// * `R` - Result type (must be serializable for caching)
    ///
    /// # Arguments
    /// * `query` - Search query parameters
    /// * `results` - Search results to cache
    ///
    /// # Example
    /// ```no_run
    /// # use media_gateway_discovery::cache::RedisCache;
    /// # use serde::{Serialize, Deserialize};
    /// # use std::sync::Arc;
    /// # use media_gateway_discovery::config::CacheConfig;
    /// #[derive(Serialize, Deserialize)]
    /// struct SearchQuery { text: String }
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct SearchResults { items: Vec<String> }
    ///
    /// # async fn example(cache: RedisCache) -> anyhow::Result<()> {
    /// let query = SearchQuery { text: "example".to_string() };
    /// let results = SearchResults { items: vec!["result1".to_string()] };
    ///
    /// cache.cache_search_results(&query, &results).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, query, results), fields(cache_type = "search"))]
    pub async fn cache_search_results<Q, R>(&self, query: &Q, results: &R) -> Result<(), CacheError>
    where
        Q: Serialize,
        R: Serialize,
    {
        let key = Self::generate_key(PREFIX_SEARCH, query)?;
        let ttl = self.config.search_ttl_sec;

        self.set(&key, results, ttl).await?;

        debug!(
            key = %key,
            ttl = %ttl,
            "Cached search results"
        );

        Ok(())
    }

    /// Get cached search results
    ///
    /// # Type Parameters
    /// * `Q` - Query type (must be serializable for key generation)
    /// * `R` - Result type (must be deserializable from cache)
    ///
    /// # Arguments
    /// * `query` - Search query parameters
    ///
    /// # Returns
    /// * `Ok(Some(R))` - Cached results found
    /// * `Ok(None)` - Cache miss
    #[instrument(skip(self, query), fields(cache_type = "search"))]
    pub async fn get_search_results<Q, R>(&self, query: &Q) -> Result<Option<R>, CacheError>
    where
        Q: Serialize,
        R: DeserializeOwned,
    {
        let key = Self::generate_key(PREFIX_SEARCH, query)?;
        let result = self.get(&key).await?;

        if result.is_some() {
            debug!(key = %key, "Search cache hit");
        } else {
            debug!(key = %key, "Search cache miss");
        }

        Ok(result)
    }

    /// Cache parsed intent with 10-minute TTL
    ///
    /// # Type Parameters
    /// * `T` - Text type for key generation
    /// * `I` - Intent type for caching
    ///
    /// # Arguments
    /// * `text` - Input text that was parsed
    /// * `intent` - Parsed intent to cache
    #[instrument(skip(self, text, intent), fields(cache_type = "intent"))]
    pub async fn cache_intent<T, I>(&self, text: &T, intent: &I) -> Result<(), CacheError>
    where
        T: Serialize,
        I: Serialize,
    {
        let key = Self::generate_key(PREFIX_INTENT, text)?;
        let ttl = self.config.intent_ttl_sec;

        self.set(&key, intent, ttl).await?;

        debug!(
            key = %key,
            ttl = %ttl,
            "Cached parsed intent"
        );

        Ok(())
    }

    /// Get cached parsed intent
    ///
    /// # Type Parameters
    /// * `T` - Text type for key generation
    /// * `I` - Intent type for retrieval
    ///
    /// # Arguments
    /// * `text` - Input text to look up
    ///
    /// # Returns
    /// * `Ok(Some(I))` - Cached intent found
    /// * `Ok(None)` - Cache miss
    #[instrument(skip(self, text), fields(cache_type = "intent"))]
    pub async fn get_intent<T, I>(&self, text: &T) -> Result<Option<I>, CacheError>
    where
        T: Serialize,
        I: DeserializeOwned,
    {
        let key = Self::generate_key(PREFIX_INTENT, text)?;
        let result = self.get(&key).await?;

        if result.is_some() {
            debug!(key = %key, "Intent cache hit");
        } else {
            debug!(key = %key, "Intent cache miss");
        }

        Ok(result)
    }

    /// Cache embedding with 1-hour TTL
    ///
    /// # Type Parameters
    /// * `T` - Text type for key generation
    ///
    /// # Arguments
    /// * `text` - Input text that was embedded
    /// * `embedding` - Vector embedding to cache
    #[instrument(skip(self, text, embedding), fields(cache_type = "embedding", dim = embedding.len()))]
    pub async fn cache_embedding<T>(&self, text: &T, embedding: &[f32]) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let key = Self::generate_key(PREFIX_EMBEDDING, text)?;
        let ttl = self.config.embedding_ttl_sec;
        let embedding_vec = embedding.to_vec();

        self.set(&key, &embedding_vec, ttl).await?;

        debug!(
            key = %key,
            ttl = %ttl,
            dim = embedding.len(),
            "Cached embedding"
        );

        Ok(())
    }

    /// Get cached embedding
    ///
    /// # Type Parameters
    /// * `T` - Text type for key generation
    ///
    /// # Arguments
    /// * `text` - Input text to look up
    ///
    /// # Returns
    /// * `Ok(Some(Vec<f32>))` - Cached embedding found
    /// * `Ok(None)` - Cache miss
    #[instrument(skip(self, text), fields(cache_type = "embedding"))]
    pub async fn get_embedding<T>(&self, text: &T) -> Result<Option<Vec<f32>>, CacheError>
    where
        T: Serialize,
    {
        let key = Self::generate_key(PREFIX_EMBEDDING, text)?;
        let result = self.get(&key).await?;

        if result.is_some() {
            debug!(key = %key, "Embedding cache hit");
        } else {
            debug!(key = %key, "Embedding cache miss");
        }

        Ok(result)
    }

    /// Clear all search result caches
    #[instrument(skip(self))]
    pub async fn clear_search_cache(&self) -> Result<u64, CacheError> {
        info!("Clearing all search caches");
        self.delete_pattern(&format!("{}:*", PREFIX_SEARCH)).await
    }

    /// Clear all intent caches
    #[instrument(skip(self))]
    pub async fn clear_intent_cache(&self) -> Result<u64, CacheError> {
        info!("Clearing all intent caches");
        self.delete_pattern(&format!("{}:*", PREFIX_INTENT)).await
    }

    /// Clear all embedding caches
    #[instrument(skip(self))]
    pub async fn clear_embedding_cache(&self) -> Result<u64, CacheError> {
        info!("Clearing all embedding caches");
        self.delete_pattern(&format!("{}:*", PREFIX_EMBEDDING))
            .await
    }

    /// Clear all caches
    #[instrument(skip(self))]
    pub async fn clear_all(&self) -> Result<u64, CacheError> {
        info!("Clearing all caches");
        let mut total = 0;
        total += self.clear_search_cache().await?;
        total += self.clear_intent_cache().await?;
        total += self.clear_embedding_cache().await?;
        info!(deleted = %total, "Cleared all caches");
        Ok(total)
    }

    /// Get cache statistics
    ///
    /// Returns Redis INFO stats for monitoring cache health
    #[instrument(skip(self))]
    pub async fn stats(&self) -> Result<CacheStats, CacheError> {
        let mut conn = self.manager.clone();

        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await
            .map_err(CacheError::Connection)?;

        // Parse basic stats from INFO output
        let mut stats = CacheStats::default();

        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                match key {
                    "keyspace_hits" => {
                        stats.hits = value.parse().unwrap_or(0);
                    }
                    "keyspace_misses" => {
                        stats.misses = value.parse().unwrap_or(0);
                    }
                    _ => {}
                }
            }
        }

        if stats.hits + stats.misses > 0 {
            stats.hit_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64;
        }

        debug!(
            hits = stats.hits,
            misses = stats.misses,
            hit_rate = %format!("{:.2}%", stats.hit_rate * 100.0),
            "Cache statistics"
        );

        Ok(stats)
    }

    /// Check if cache is healthy (connection is alive)
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool, CacheError> {
        let mut conn = self.manager.clone();

        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
            Ok(response) => {
                let healthy = response == "PONG";
                debug!(healthy = %healthy, "Cache health check");
                Ok(healthy)
            }
            Err(e) => {
                error!(error = %e, "Cache health check failed");
                Err(CacheError::Connection(e))
            }
        }
    }

    /// Create a mock cache for testing (no-op implementation)
    #[cfg(test)]
    pub fn new_mock() -> Self {
        use redis::aio::ConnectionManager;
        use redis::Client;

        // Create a dummy client that will never be used
        let client = Client::open("redis://127.0.0.1:6379").expect("Mock client creation");
        // Note: ConnectionManager creation would fail, so we use unsafe for testing only
        let manager = unsafe { std::mem::zeroed() };

        let config = Arc::new(CacheConfig {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        Self { manager, config }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Cache hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestQuery {
        text: String,
        limit: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResult {
        items: Vec<String>,
        count: usize,
    }

    #[test]
    fn test_generate_key() {
        let query = TestQuery {
            text: "test query".to_string(),
            limit: 10,
        };

        let key1 = RedisCache::generate_key("search", &query).unwrap();
        let key2 = RedisCache::generate_key("search", &query).unwrap();

        // Same query should generate same key
        assert_eq!(key1, key2);
        assert!(key1.starts_with("search:"));
        assert_eq!(key1.len(), "search:".len() + 64); // SHA256 = 64 hex chars

        // Different query should generate different key
        let query2 = TestQuery {
            text: "different".to_string(),
            limit: 10,
        };
        let key3 = RedisCache::generate_key("search", &query2).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_generate_key_different_prefixes() {
        let data = "test";
        let key1 = RedisCache::generate_key("search", &data).unwrap();
        let key2 = RedisCache::generate_key("intent", &data).unwrap();

        assert!(key1.starts_with("search:"));
        assert!(key2.starts_with("intent:"));
        assert_ne!(key1, key2);
    }

    #[tokio::test]
    async fn test_cache_lifecycle() {
        // This test requires a running Redis instance
        let config = Arc::new(CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 60,
            embedding_ttl_sec: 120,
            intent_ttl_sec: 30,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        // Test basic set/get
        let key = "test:lifecycle";
        let value = TestResult {
            items: vec!["item1".to_string(), "item2".to_string()],
            count: 2,
        };

        cache.set(key, &value, 60).await.unwrap();

        let retrieved: Option<TestResult> = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value.clone()));

        // Test delete
        let deleted = cache.delete(key).await.unwrap();
        assert_eq!(deleted, 1);

        let retrieved: Option<TestResult> = cache.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_search_cache() {
        let config = Arc::new(CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 60,
            embedding_ttl_sec: 120,
            intent_ttl_sec: 30,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let query = TestQuery {
            text: "search test".to_string(),
            limit: 10,
        };

        let results = TestResult {
            items: vec!["result1".to_string(), "result2".to_string()],
            count: 2,
        };

        // Cache miss
        let cached: Option<TestResult> = cache.get_search_results(&query).await.unwrap();
        assert_eq!(cached, None);

        // Cache set
        cache.cache_search_results(&query, &results).await.unwrap();

        // Cache hit
        let cached: Option<TestResult> = cache.get_search_results(&query).await.unwrap();
        assert_eq!(cached, Some(results));

        // Cleanup
        cache.clear_search_cache().await.unwrap();
    }

    #[tokio::test]
    async fn test_embedding_cache() {
        let config = Arc::new(CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 60,
            embedding_ttl_sec: 120,
            intent_ttl_sec: 30,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let text = "embedding test text";
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        // Cache miss
        let cached = cache.get_embedding(&text).await.unwrap();
        assert_eq!(cached, None);

        // Cache set
        cache.cache_embedding(&text, &embedding).await.unwrap();

        // Cache hit
        let cached = cache.get_embedding(&text).await.unwrap();
        assert_eq!(cached, Some(embedding));

        // Cleanup
        cache.clear_embedding_cache().await.unwrap();
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = Arc::new(CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 60,
            embedding_ttl_sec: 120,
            intent_ttl_sec: 30,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let healthy = cache.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    async fn test_delete_pattern() {
        let config = Arc::new(CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 60,
            embedding_ttl_sec: 120,
            intent_ttl_sec: 30,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        // Set multiple keys
        cache.set("test:pattern:1", &"value1", 60).await.unwrap();
        cache.set("test:pattern:2", &"value2", 60).await.unwrap();
        cache.set("test:other:1", &"value3", 60).await.unwrap();

        // Delete pattern
        let deleted = cache.delete_pattern("test:pattern:*").await.unwrap();
        assert_eq!(deleted, 2);

        // Verify
        let val1: Option<String> = cache.get("test:pattern:1").await.unwrap();
        let val2: Option<String> = cache.get("test:pattern:2").await.unwrap();
        let val3: Option<String> = cache.get("test:other:1").await.unwrap();

        assert_eq!(val1, None);
        assert_eq!(val2, None);
        assert_eq!(val3, Some("value3".to_string()));

        // Cleanup
        cache.delete("test:other:1").await.unwrap();
    }
}
