//! Circuit breaker integration for platform normalizers
//!
//! This module provides circuit breaker wrappers for platform API calls,
//! implementing fallback strategies and distributed state management.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use media_gateway_core::resilience::{CircuitBreaker, CircuitBreakerConfig};
use std::sync::Arc;
use tracing::{debug, warn};

use super::{CanonicalContent, PlatformNormalizer, RawContent};
use crate::Result;

/// Wrapper that adds circuit breaker protection to platform normalizers
pub struct CircuitBreakerNormalizer<N> {
    inner: N,
    circuit_breaker: Arc<CircuitBreaker>,
    cache: Option<Arc<dyn FallbackCache>>,
}

impl<N> CircuitBreakerNormalizer<N>
where
    N: PlatformNormalizer,
{
    /// Create a new circuit breaker normalizer
    ///
    /// # Arguments
    /// * `normalizer` - The underlying platform normalizer
    /// * `config` - Circuit breaker configuration
    pub fn new(normalizer: N, config: CircuitBreakerConfig) -> Self {
        let platform_id = normalizer.platform_id();
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            format!("normalizer-{}", platform_id),
            config,
        ));

        Self {
            inner: normalizer,
            circuit_breaker,
            cache: None,
        }
    }

    /// Create with Redis-backed distributed circuit breaker
    ///
    /// # Arguments
    /// * `normalizer` - The underlying platform normalizer
    /// * `config` - Circuit breaker configuration
    /// * `redis_client` - Redis client for distributed state
    pub fn with_redis(
        normalizer: N,
        config: CircuitBreakerConfig,
        redis_client: redis::Client,
    ) -> Self {
        let platform_id = normalizer.platform_id();
        let circuit_breaker = Arc::new(CircuitBreaker::with_redis(
            format!("normalizer-{}", platform_id),
            config,
            redis_client,
        ));

        Self {
            inner: normalizer,
            circuit_breaker,
            cache: None,
        }
    }

    /// Add fallback cache for when circuit is open
    pub fn with_cache(mut self, cache: Arc<dyn FallbackCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Get the underlying circuit breaker for metrics
    pub fn circuit_breaker(&self) -> Arc<CircuitBreaker> {
        Arc::clone(&self.circuit_breaker)
    }
}

#[async_trait]
impl<N> PlatformNormalizer for CircuitBreakerNormalizer<N>
where
    N: PlatformNormalizer,
{
    fn platform_id(&self) -> &'static str {
        self.inner.platform_id()
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>> {
        let platform_id = self.platform_id();

        // Try with circuit breaker and fallback to cache if available
        if let Some(cache) = &self.cache {
            let cache_clone = Arc::clone(cache);
            let region_owned = region.to_string();
            let since_clone = since;

            self.circuit_breaker
                .call_with_fallback(self.inner.fetch_catalog_delta(since, region), move || {
                    debug!(
                        platform = platform_id,
                        region = %region_owned,
                        "Circuit open, using cached catalog"
                    );
                    cache_clone.get_catalog(&region_owned, since_clone)
                })
                .await
                .map_err(|e| match e {
                    media_gateway_core::resilience::CircuitBreakerError::CircuitOpen { .. } => {
                        crate::Error::ServiceUnavailable(format!(
                            "Platform {} is unavailable",
                            platform_id
                        ))
                    }
                    media_gateway_core::resilience::CircuitBreakerError::OperationFailed(e) => e,
                    _ => crate::Error::Internal(format!("Circuit breaker error: {}", e)),
                })
        } else {
            // No cache, fail if circuit open
            self.circuit_breaker
                .call(self.inner.fetch_catalog_delta(since, region))
                .await
                .map_err(|e| match e {
                    media_gateway_core::resilience::CircuitBreakerError::CircuitOpen {
                        circuit_name,
                    } => {
                        warn!(circuit = %circuit_name, "Circuit open, no fallback available");
                        crate::Error::ServiceUnavailable(format!(
                            "Platform {} is unavailable",
                            platform_id
                        ))
                    }
                    media_gateway_core::resilience::CircuitBreakerError::OperationFailed(e) => e,
                    _ => crate::Error::Internal(format!("Circuit breaker error: {}", e)),
                })
        }
    }

    fn normalize(&self, raw: RawContent) -> Result<CanonicalContent> {
        // Normalization is typically fast and local, no circuit breaker needed
        self.inner.normalize(raw)
    }

    fn generate_deep_link(&self, content_id: &str) -> crate::deep_link::DeepLinkResult {
        // Deep link generation is deterministic, no circuit breaker needed
        self.inner.generate_deep_link(content_id)
    }

    fn rate_limit_config(&self) -> super::RateLimitConfig {
        self.inner.rate_limit_config()
    }
}

/// Fallback cache interface for circuit breaker
#[async_trait]
pub trait FallbackCache: Send + Sync {
    /// Get cached catalog data
    fn get_catalog(&self, region: &str, since: DateTime<Utc>) -> Vec<RawContent>;

    /// Update cache with fresh data
    async fn update_catalog(&self, region: &str, data: Vec<RawContent>) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct MockNormalizer {
        should_fail: Arc<tokio::sync::RwLock<bool>>,
    }

    #[async_trait]
    impl PlatformNormalizer for MockNormalizer {
        fn platform_id(&self) -> &'static str {
            "mock"
        }

        async fn fetch_catalog_delta(
            &self,
            _since: DateTime<Utc>,
            _region: &str,
        ) -> Result<Vec<RawContent>> {
            if *self.should_fail.read().await {
                Err(crate::Error::External("API failed".to_string()))
            } else {
                Ok(vec![])
            }
        }

        fn normalize(&self, raw: RawContent) -> Result<CanonicalContent> {
            // Mock implementation
            Ok(CanonicalContent {
                platform_content_id: raw.id,
                platform_id: raw.platform,
                entity_id: None,
                title: "Test".to_string(),
                overview: None,
                content_type: super::super::ContentType::Movie,
                release_year: None,
                runtime_minutes: None,
                genres: vec![],
                external_ids: std::collections::HashMap::new(),
                availability: super::super::AvailabilityInfo {
                    regions: vec![],
                    subscription_required: false,
                    purchase_price: None,
                    rental_price: None,
                    currency: None,
                    available_from: None,
                    available_until: None,
                },
                images: super::super::ImageSet::default(),
                rating: None,
                user_rating: None,
                embedding: None,
                updated_at: Utc::now(),
            })
        }

        fn generate_deep_link(&self, _content_id: &str) -> crate::deep_link::DeepLinkResult {
            crate::deep_link::DeepLinkResult {
                mobile_url: None,
                web_url: None,
                universal_link: None,
            }
        }

        fn rate_limit_config(&self) -> super::super::RateLimitConfig {
            super::super::RateLimitConfig {
                max_requests: 100,
                window: Duration::from_secs(60),
                api_keys: vec![],
            }
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_normalizer_allows_calls_when_closed() {
        let mock = MockNormalizer {
            should_fail: Arc::new(tokio::sync::RwLock::new(false)),
        };

        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_duration: Duration::from_secs(1),
            half_open_max_calls: 2,
        };

        let normalizer = CircuitBreakerNormalizer::new(mock, config);

        let result = normalizer.fetch_catalog_delta(Utc::now(), "US").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_normalizer_opens_on_failures() {
        let should_fail = Arc::new(tokio::sync::RwLock::new(true));
        let mock = MockNormalizer {
            should_fail: Arc::clone(&should_fail),
        };

        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_duration: Duration::from_secs(10),
            half_open_max_calls: 2,
        };

        let normalizer = CircuitBreakerNormalizer::new(mock, config);

        // Trigger failures to open circuit
        for _ in 0..2 {
            let _ = normalizer.fetch_catalog_delta(Utc::now(), "US").await;
        }

        // Next call should fail immediately (circuit open)
        let result = normalizer.fetch_catalog_delta(Utc::now(), "US").await;
        assert!(result.is_err());
    }
}
