//! Webhook receiver implementation

use crate::webhooks::{
    PlatformWebhookConfig, ProcessedWebhook, ProcessingStatus, WebhookDeduplicator, WebhookError,
    WebhookHandler, WebhookMetrics, WebhookPayload, WebhookQueue, WebhookResult,
};
use async_trait::async_trait;
use chrono::Utc;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use nonzero_ext::nonzero;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Webhook receiver
pub struct WebhookReceiver {
    handlers: Arc<RwLock<HashMap<String, Box<dyn WebhookHandler>>>>,
    configs: Arc<RwLock<HashMap<String, PlatformWebhookConfig>>>,
    rate_limiters:
        Arc<RwLock<HashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
    queue: Arc<dyn WebhookQueue>,
    deduplicator: Arc<WebhookDeduplicator>,
    metrics: Arc<WebhookMetrics>,
}

impl WebhookReceiver {
    /// Create a new webhook receiver
    pub fn new(
        queue: Arc<dyn WebhookQueue>,
        deduplicator: Arc<WebhookDeduplicator>,
        metrics: Arc<WebhookMetrics>,
    ) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            queue,
            deduplicator,
            metrics,
        }
    }

    /// Register a webhook handler for a platform
    pub async fn register_handler(
        &self,
        handler: Box<dyn WebhookHandler>,
        config: PlatformWebhookConfig,
    ) -> WebhookResult<()> {
        let platform = handler.platform_id().to_string();

        // Create rate limiter with runtime value
        let rate_limit_value = config.rate_limit;
        let quota = Quota::per_minute(
            std::num::NonZeroU32::new(rate_limit_value).unwrap_or(nonzero!(100u32)),
        );
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        // Store handler, config, and rate limiter
        self.handlers
            .write()
            .await
            .insert(platform.clone(), handler);
        self.configs.write().await.insert(platform.clone(), config);
        self.rate_limiters
            .write()
            .await
            .insert(platform, rate_limiter);

        Ok(())
    }

    /// Receive and process a webhook
    pub async fn receive(
        &self,
        platform: &str,
        body: &[u8],
        signature: &str,
    ) -> WebhookResult<String> {
        self.metrics.increment_received();

        // Check if platform is registered
        let handlers = self.handlers.read().await;
        let handler = handlers
            .get(platform)
            .ok_or_else(|| WebhookError::UnsupportedPlatform(platform.to_string()))?;

        // Get config
        let configs = self.configs.read().await;
        let config = configs
            .get(platform)
            .ok_or_else(|| WebhookError::UnsupportedPlatform(platform.to_string()))?;

        // Check if enabled
        if !config.enabled {
            return Err(WebhookError::ProcessingError(format!(
                "Webhooks disabled for platform: {}",
                platform
            )));
        }

        // Check rate limit
        let rate_limiters = self.rate_limiters.read().await;
        let rate_limiter = rate_limiters
            .get(platform)
            .ok_or_else(|| WebhookError::RateLimitExceeded(platform.to_string()))?;

        if rate_limiter.check().is_err() {
            self.metrics.increment_rate_limited();
            return Err(WebhookError::RateLimitExceeded(platform.to_string()));
        }

        // Verify signature
        let is_valid = handler.verify_signature(body, signature, &config.secret)?;
        if !is_valid {
            self.metrics.increment_failed();
            return Err(WebhookError::InvalidSignature(
                "Signature verification failed".to_string(),
            ));
        }

        // Parse payload
        let webhook = handler.parse_payload(body)?;

        // Check for duplicates
        if self.deduplicator.is_duplicate(&webhook).await? {
            self.metrics.increment_duplicates();
            let hash = WebhookDeduplicator::compute_hash(&webhook);
            return Ok(hash);
        }

        // Mark as processed for deduplication
        let event_id = self.deduplicator.mark_processed(&webhook).await?;

        // Enqueue for processing
        let message_id = self.queue.enqueue(webhook).await?;

        self.metrics.increment_processed();

        Ok(event_id)
    }

    /// Process webhooks from queue
    pub async fn process_queue(&self, consumer_name: &str) -> WebhookResult<()> {
        loop {
            // Dequeue next webhook
            let item = self.queue.dequeue(consumer_name).await?;

            if let Some((message_id, webhook)) = item {
                // Get handler
                let handlers = self.handlers.read().await;
                let handler = handlers
                    .get(&webhook.platform)
                    .ok_or_else(|| WebhookError::UnsupportedPlatform(webhook.platform.clone()))?;

                // Process event
                match handler.process_event(webhook.clone()).await {
                    Ok(processed) => {
                        // Acknowledge message
                        self.queue.ack(&message_id).await?;
                        tracing::info!(
                            "Processed webhook: platform={} event_id={}",
                            webhook.platform,
                            processed.event_id
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to process webhook: platform={} error={}",
                            webhook.platform,
                            e
                        );

                        // Move to dead letter queue
                        let processed = ProcessedWebhook {
                            event_id: WebhookDeduplicator::compute_hash(&webhook),
                            webhook,
                            processed_at: Utc::now(),
                            status: ProcessingStatus::Failed,
                            error: Some(e.to_string()),
                        };

                        self.queue.dead_letter(processed).await?;
                        self.metrics.increment_failed();
                    }
                }
            } else {
                // No messages available, wait a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> Arc<WebhookMetrics> {
        self.metrics.clone()
    }

    /// Get queue statistics
    pub async fn queue_stats(&self) -> WebhookResult<crate::webhooks::queue::QueueStats> {
        self.queue.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhooks::{queue::RedisWebhookQueue, WebhookEventType};

    struct MockHandler;

    #[async_trait]
    impl WebhookHandler for MockHandler {
        fn platform_id(&self) -> &'static str {
            "test-platform"
        }

        fn verify_signature(
            &self,
            _payload: &[u8],
            _signature: &str,
            _secret: &str,
        ) -> WebhookResult<bool> {
            Ok(true)
        }

        fn parse_payload(&self, body: &[u8]) -> WebhookResult<WebhookPayload> {
            let payload: WebhookPayload = serde_json::from_slice(body)
                .map_err(|e| WebhookError::InvalidPayload(e.to_string()))?;
            Ok(payload)
        }

        async fn process_event(&self, webhook: WebhookPayload) -> WebhookResult<ProcessedWebhook> {
            Ok(ProcessedWebhook {
                event_id: WebhookDeduplicator::compute_hash(&webhook),
                webhook,
                processed_at: Utc::now(),
                status: ProcessingStatus::Completed,
                error: None,
            })
        }
    }

    #[tokio::test]
    async fn test_register_handler() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
            Ok(q) => Arc::new(q) as Arc<dyn WebhookQueue>,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let deduplicator = Arc::new(WebhookDeduplicator::new(&redis_url, Some(1)).unwrap());
        let metrics = Arc::new(WebhookMetrics::new());

        let receiver = WebhookReceiver::new(queue, deduplicator, metrics);

        let handler = Box::new(MockHandler);
        let config = PlatformWebhookConfig {
            platform: "test-platform".to_string(),
            secret: "test-secret".to_string(),
            rate_limit: 100,
            enabled: true,
        };

        receiver.register_handler(handler, config).await.unwrap();

        let handlers = receiver.handlers.read().await;
        assert!(handlers.contains_key("test-platform"));
    }

    #[tokio::test]
    async fn test_receive_webhook() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
            Ok(q) => Arc::new(q) as Arc<dyn WebhookQueue>,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let deduplicator = Arc::new(WebhookDeduplicator::new(&redis_url, Some(1)).unwrap());
        let metrics = Arc::new(WebhookMetrics::new());

        let receiver = WebhookReceiver::new(queue, deduplicator, metrics);

        let handler = Box::new(MockHandler);
        let config = PlatformWebhookConfig {
            platform: "test-platform".to_string(),
            secret: "test-secret".to_string(),
            rate_limit: 100,
            enabled: true,
        };

        receiver.register_handler(handler, config).await.unwrap();

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "test-platform".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "test-receive"}),
            signature: "sha256=test".to_string(),
        };

        let body = serde_json::to_vec(&webhook).unwrap();
        let event_id = receiver
            .receive("test-platform", &body, "sha256=test")
            .await
            .unwrap();

        assert!(!event_id.is_empty());
    }
}
