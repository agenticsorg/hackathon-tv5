//! Rate limiting for API calls with multi-key rotation

use crate::{IngestionError, Result};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Jitter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Rate limiter for a specific API
struct ApiRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    api_keys: Vec<String>,
    current_key_index: std::sync::atomic::AtomicUsize,
}

impl ApiRateLimiter {
    fn new(max_requests: u32, window: Duration, api_keys: Vec<String>) -> Self {
        let quota = Quota::with_period(window)
            .unwrap()
            .allow_burst(NonZeroU32::new(max_requests).unwrap());

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
            api_keys,
            current_key_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Get next API key in rotation
    fn get_next_key(&self) -> String {
        if self.api_keys.is_empty() {
            return String::new();
        }

        let index = self
            .current_key_index
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.api_keys[index % self.api_keys.len()].clone()
    }

    /// Check rate limit and wait if necessary
    async fn check_and_wait(&self) -> Result<()> {
        // Try to acquire permit with jitter to avoid thundering herd
        // until_ready_with_jitter returns () not Result, it always waits until ready
        self.limiter
            .until_ready_with_jitter(Jitter::up_to(Duration::from_millis(100)))
            .await;
        Ok(())
    }
}

/// Rate limit manager for all platform APIs
pub struct RateLimitManager {
    limiters: Arc<RwLock<HashMap<String, Arc<ApiRateLimiter>>>>,
}

impl RateLimitManager {
    /// Create a new rate limit manager
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a platform with rate limiting configuration
    ///
    /// # Arguments
    /// * `platform_id` - Platform identifier
    /// * `max_requests` - Maximum requests per time window
    /// * `window` - Time window duration
    /// * `api_keys` - API keys for rotation (empty for platforms without keys)
    pub async fn register_platform(
        &self,
        platform_id: String,
        max_requests: u32,
        window: Duration,
        api_keys: Vec<String>,
    ) {
        let limiter = Arc::new(ApiRateLimiter::new(max_requests, window, api_keys));

        let mut limiters = self.limiters.write().await;
        limiters.insert(platform_id.clone(), limiter);

        debug!(
            "Registered rate limiter for {} with {} req per {:?}",
            platform_id, max_requests, window
        );
    }

    /// Check rate limit and wait if necessary
    ///
    /// # Arguments
    /// * `platform_id` - Platform identifier
    ///
    /// # Returns
    /// Ok if ready to proceed, Err if rate limit exceeded
    pub async fn check_and_wait(&self, platform_id: &str) -> Result<()> {
        let limiters = self.limiters.read().await;

        if let Some(limiter) = limiters.get(platform_id) {
            limiter.check_and_wait().await
        } else {
            warn!("No rate limiter configured for {}", platform_id);
            Ok(())
        }
    }

    /// Get next API key for platform (with rotation)
    ///
    /// # Arguments
    /// * `platform_id` - Platform identifier
    ///
    /// # Returns
    /// Next API key in rotation, or empty string if no keys configured
    pub async fn get_api_key(&self, platform_id: &str) -> String {
        let limiters = self.limiters.read().await;

        if let Some(limiter) = limiters.get(platform_id) {
            limiter.get_next_key()
        } else {
            String::new()
        }
    }

    /// Initialize default rate limiters for known platforms
    pub async fn init_defaults(&self) {
        // Streaming Availability API: 100 req/min
        self.register_platform(
            "streaming_availability".to_string(),
            100,
            Duration::from_secs(60),
            vec![], // Keys provided separately
        )
        .await;

        // Watchmode API: 1000 req/day
        self.register_platform(
            "watchmode".to_string(),
            1000,
            Duration::from_secs(86400),
            vec![],
        )
        .await;

        // YouTube Data API: 100 searches per day per key
        // With 5 keys, we can do 500 searches per day
        self.register_platform(
            "youtube".to_string(),
            100,
            Duration::from_secs(86400),
            vec![], // Keys provided separately
        )
        .await;

        // TMDb API: 40 req per 10 seconds
        self.register_platform("tmdb".to_string(), 40, Duration::from_secs(10), vec![])
            .await;

        debug!("Initialized default rate limiters");
    }
}

impl Default for RateLimitManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_registration() {
        let manager = RateLimitManager::new();

        manager
            .register_platform(
                "test_platform".to_string(),
                10,
                Duration::from_secs(1),
                vec!["key1".to_string(), "key2".to_string()],
            )
            .await;

        // Should not fail
        let result = manager.check_and_wait("test_platform").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_key_rotation() {
        let manager = RateLimitManager::new();

        manager
            .register_platform(
                "test_platform".to_string(),
                10,
                Duration::from_secs(1),
                vec!["key1".to_string(), "key2".to_string(), "key3".to_string()],
            )
            .await;

        let key1 = manager.get_api_key("test_platform").await;
        let key2 = manager.get_api_key("test_platform").await;
        let key3 = manager.get_api_key("test_platform").await;
        let key4 = manager.get_api_key("test_platform").await;

        assert_eq!(key1, "key1");
        assert_eq!(key2, "key2");
        assert_eq!(key3, "key3");
        assert_eq!(key4, "key1"); // Should wrap around
    }

    #[tokio::test]
    async fn test_unknown_platform() {
        let manager = RateLimitManager::new();

        // Should not fail for unknown platform
        let result = manager.check_and_wait("unknown").await;
        assert!(result.is_ok());

        // Should return empty string for unknown platform
        let key = manager.get_api_key("unknown").await;
        assert_eq!(key, "");
    }

    #[tokio::test]
    async fn test_default_initialization() {
        let manager = RateLimitManager::new();
        manager.init_defaults().await;

        // Check that default platforms are registered
        let result = manager.check_and_wait("streaming_availability").await;
        assert!(result.is_ok());

        let result = manager.check_and_wait("youtube").await;
        assert!(result.is_ok());
    }
}
