//! Webhook queue implementation using Redis Streams

use crate::webhooks::{
    ProcessedWebhook, ProcessingStatus, WebhookError, WebhookPayload, WebhookResult,
};
use async_trait::async_trait;
use chrono::Utc;
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    AsyncCommands, Client,
};
use serde::{Deserialize, Serialize};

/// Webhook queue trait
#[async_trait]
pub trait WebhookQueue: Send + Sync {
    /// Enqueue a webhook for processing
    async fn enqueue(&self, webhook: WebhookPayload) -> WebhookResult<String>;

    /// Dequeue next webhook for processing
    async fn dequeue(&self, consumer_name: &str)
        -> WebhookResult<Option<(String, WebhookPayload)>>;

    /// Acknowledge successful processing
    async fn ack(&self, message_id: &str) -> WebhookResult<()>;

    /// Move to dead letter queue
    async fn dead_letter(&self, webhook: ProcessedWebhook) -> WebhookResult<()>;

    /// Get queue statistics
    async fn stats(&self) -> WebhookResult<QueueStats>;
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending_count: u64,
    pub processing_count: u64,
    pub dead_letter_count: u64,
    pub total_processed: u64,
}

/// Redis Streams webhook queue
pub struct RedisWebhookQueue {
    client: Client,
    stream_prefix: String,
    dlq_prefix: String,
    consumer_group: String,
    platforms: Vec<String>,
    processing_count: Arc<std::sync::atomic::AtomicU64>,
    total_processed: Arc<std::sync::atomic::AtomicU64>,
}

use std::sync::Arc;

impl RedisWebhookQueue {
    /// Create a new Redis webhook queue
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL
    /// * `stream_prefix` - Prefix for stream keys (default: "webhooks:incoming")
    /// * `dlq_prefix` - Prefix for dead letter queue (default: "webhooks:dlq")
    /// * `consumer_group` - Consumer group name (default: "webhook-processors")
    pub fn new(
        redis_url: &str,
        stream_prefix: Option<String>,
        dlq_prefix: Option<String>,
        consumer_group: Option<String>,
    ) -> WebhookResult<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| WebhookError::RedisError(format!("Failed to connect: {}", e)))?;

        let platforms = std::env::var("WEBHOOK_PLATFORMS")
            .unwrap_or_else(|_| {
                "netflix,hulu,disney_plus,prime_video,hbo_max,apple_tv_plus,paramount_plus,peacock"
                    .to_string()
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            client,
            stream_prefix: stream_prefix.unwrap_or_else(|| "webhooks:incoming".to_string()),
            dlq_prefix: dlq_prefix.unwrap_or_else(|| "webhooks:dlq".to_string()),
            consumer_group: consumer_group.unwrap_or_else(|| "webhook-processors".to_string()),
            platforms,
            processing_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_processed: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    pub fn with_platforms(mut self, platforms: Vec<String>) -> Self {
        self.platforms = platforms;
        self
    }

    /// Initialize consumer group for a platform
    async fn ensure_consumer_group(&self, platform: &str) -> WebhookResult<()> {
        let stream_key = format!("{}:{}", self.stream_prefix, platform);
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        // Create stream if it doesn't exist
        let _: Result<(), redis::RedisError> = conn
            .xgroup_create_mkstream(&stream_key, &self.consumer_group, "0")
            .await;

        Ok(())
    }

    fn stream_key(&self, platform: &str) -> String {
        format!("{}:{}", self.stream_prefix, platform)
    }

    fn dlq_key(&self, platform: &str) -> String {
        format!("{}:{}", self.dlq_prefix, platform)
    }
}

#[async_trait]
impl WebhookQueue for RedisWebhookQueue {
    async fn enqueue(&self, webhook: WebhookPayload) -> WebhookResult<String> {
        let stream_key = self.stream_key(&webhook.platform);
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        // Ensure consumer group exists
        self.ensure_consumer_group(&webhook.platform).await?;

        // Serialize webhook payload
        let payload_json =
            serde_json::to_string(&webhook).map_err(|e| WebhookError::SerializationError(e))?;

        // Add to stream
        let message_id: String = conn
            .xadd(&stream_key, "*", &[("payload", payload_json)])
            .await
            .map_err(|e| WebhookError::QueueError(format!("Failed to enqueue: {}", e)))?;

        Ok(message_id)
    }

    async fn dequeue(
        &self,
        _consumer_name: &str,
    ) -> WebhookResult<Option<(String, WebhookPayload)>> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        for platform in &self.platforms {
            let stream_key = self.stream_key(platform);

            // Ensure consumer group exists
            self.ensure_consumer_group(platform).await?;

            // Read from stream using consumer group
            let opts = StreamReadOptions::default().count(1).block(100); // 100ms block

            let result: StreamReadReply =
                conn.xread_options(&[&stream_key], &[">"], &opts)
                    .await
                    .map_err(|e| WebhookError::QueueError(format!("Failed to dequeue: {}", e)))?;

            if !result.keys.is_empty() && !result.keys[0].ids.is_empty() {
                let stream_id = &result.keys[0].ids[0];
                let message_id = stream_id.id.clone();

                // Extract payload
                if let Some(payload_value) = stream_id.map.get("payload") {
                    if let redis::Value::Data(payload_bytes) = payload_value {
                        let payload_str = String::from_utf8_lossy(payload_bytes);
                        let webhook: WebhookPayload = serde_json::from_str(&payload_str)
                            .map_err(|e| WebhookError::SerializationError(e))?;

                        self.processing_count
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        return Ok(Some((message_id, webhook)));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn ack(&self, message_id: &str) -> WebhookResult<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        for platform in &self.platforms {
            let stream_key = self.stream_key(platform);

            let _: Result<i32, redis::RedisError> = conn
                .xack(&stream_key, &self.consumer_group, &[message_id])
                .await;
        }

        self.processing_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        self.total_processed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    async fn dead_letter(&self, webhook: ProcessedWebhook) -> WebhookResult<()> {
        let dlq_key = self.dlq_key(&webhook.webhook.platform);
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        // Serialize processed webhook
        let payload_json =
            serde_json::to_string(&webhook).map_err(|e| WebhookError::SerializationError(e))?;

        // Add to dead letter queue stream
        let _: String = conn
            .xadd(&dlq_key, "*", &[("payload", payload_json)])
            .await
            .map_err(|e| WebhookError::QueueError(format!("Failed to dead letter: {}", e)))?;

        Ok(())
    }

    async fn stats(&self) -> WebhookResult<QueueStats> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        let mut total_pending = 0u64;
        let mut total_dlq = 0u64;

        for platform in &self.platforms {
            let stream_key = self.stream_key(platform);
            let dlq_key = self.dlq_key(platform);

            // Get stream length
            let pending: u64 = conn.xlen(&stream_key).await.unwrap_or(0);
            total_pending += pending;

            // Get DLQ length
            let dlq_count: u64 = conn.xlen(&dlq_key).await.unwrap_or(0);
            total_dlq += dlq_count;
        }

        Ok(QueueStats {
            pending_count: total_pending,
            processing_count: self
                .processing_count
                .load(std::sync::atomic::Ordering::Relaxed),
            dead_letter_count: total_dlq,
            total_processed: self
                .total_processed
                .load(std::sync::atomic::Ordering::Relaxed),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhooks::WebhookEventType;

    #[tokio::test]
    async fn test_queue_enqueue_dequeue() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
            Ok(q) => q,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "test-queue"}),
            signature: "sha256=test".to_string(),
        };

        // Enqueue
        let message_id = queue.enqueue(webhook.clone()).await.unwrap();
        assert!(!message_id.is_empty());

        // Dequeue
        let result = queue.dequeue("test-consumer").await.unwrap();
        assert!(result.is_some());

        let (dequeued_id, dequeued_webhook) = result.unwrap();
        assert_eq!(dequeued_webhook.platform, "netflix");
        assert_eq!(dequeued_webhook.event_type, WebhookEventType::ContentAdded);

        // Ack
        queue.ack(&dequeued_id).await.unwrap();

        // Clean up
        let mut conn = queue.client.get_async_connection().await.unwrap();
        let stream_key = queue.stream_key("netflix");
        let _: () = redis::cmd("DEL")
            .arg(&stream_key)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_dead_letter_queue() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
            Ok(q) => q,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "test-dlq"}),
            signature: "sha256=test".to_string(),
        };

        let processed = ProcessedWebhook {
            event_id: "test-event-id".to_string(),
            webhook,
            processed_at: Utc::now(),
            status: ProcessingStatus::Failed,
            error: Some("Test error".to_string()),
        };

        // Add to DLQ
        queue.dead_letter(processed).await.unwrap();

        // Verify stats
        let stats = queue.stats().await.unwrap();
        assert!(stats.dead_letter_count > 0);

        // Clean up
        let mut conn = queue.client.get_async_connection().await.unwrap();
        let dlq_key = queue.dlq_key("netflix");
        let _: () = redis::cmd("DEL")
            .arg(&dlq_key)
            .query_async(&mut conn)
            .await
            .unwrap();
    }
}
