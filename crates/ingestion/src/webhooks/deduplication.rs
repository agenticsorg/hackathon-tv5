//! Webhook deduplication using content hash and Redis

use crate::webhooks::{WebhookError, WebhookEventType, WebhookPayload, WebhookResult};
use chrono::{DateTime, Duration, Utc};
use redis::{AsyncCommands, Client};
use sha2::{Digest, Sha256};

/// Webhook deduplicator
pub struct WebhookDeduplicator {
    client: Client,
    ttl_seconds: i64,
}

impl WebhookDeduplicator {
    /// Create a new webhook deduplicator
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL
    /// * `ttl_hours` - TTL for deduplication entries (default: 24 hours)
    pub fn new(redis_url: &str, ttl_hours: Option<i64>) -> WebhookResult<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| WebhookError::RedisError(format!("Failed to connect: {}", e)))?;

        let ttl_seconds = ttl_hours.unwrap_or(24) * 3600;

        Ok(Self {
            client,
            ttl_seconds,
        })
    }

    /// Compute content hash for webhook payload
    ///
    /// # Arguments
    /// * `webhook` - Webhook payload
    ///
    /// # Returns
    /// SHA-256 hash of the payload
    pub fn compute_hash(webhook: &WebhookPayload) -> String {
        let mut hasher = Sha256::new();

        // Hash relevant fields (excluding signature)
        hasher.update(webhook.platform.as_bytes());

        let event_type_str = match &webhook.event_type {
            WebhookEventType::ContentAdded => "content_added",
            WebhookEventType::ContentUpdated => "content_updated",
            WebhookEventType::ContentRemoved => "content_removed",
        };
        hasher.update(event_type_str.as_bytes());

        hasher.update(webhook.payload.to_string().as_bytes());

        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Check if webhook is a duplicate
    ///
    /// # Arguments
    /// * `webhook` - Webhook payload
    ///
    /// # Returns
    /// True if duplicate, false if new
    pub async fn is_duplicate(&self, webhook: &WebhookPayload) -> WebhookResult<bool> {
        let hash = Self::compute_hash(webhook);
        let key = format!("webhook:hash:{}", hash);

        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| WebhookError::DeduplicationError(format!("Redis exists failed: {}", e)))?;

        Ok(exists)
    }

    /// Mark webhook as processed
    ///
    /// # Arguments
    /// * `webhook` - Webhook payload
    ///
    /// # Returns
    /// Content hash of the webhook
    pub async fn mark_processed(&self, webhook: &WebhookPayload) -> WebhookResult<String> {
        let hash = Self::compute_hash(webhook);
        let key = format!("webhook:hash:{}", hash);

        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        // Store hash with TTL
        let _: () = conn
            .set_ex(
                &key,
                webhook.timestamp.to_rfc3339(),
                self.ttl_seconds as u64,
            )
            .await
            .map_err(|e| WebhookError::DeduplicationError(format!("Redis set failed: {}", e)))?;

        Ok(hash)
    }

    /// Get deduplication statistics
    ///
    /// # Returns
    /// Number of unique webhooks processed in last 24 hours
    pub async fn get_stats(&self) -> WebhookResult<u64> {
        let pattern = "webhook:hash:*";

        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WebhookError::RedisError(format!("Connection failed: {}", e)))?;

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| WebhookError::RedisError(format!("Redis keys failed: {}", e)))?;

        Ok(keys.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhooks::WebhookEventType;

    #[test]
    fn test_compute_hash_deterministic() {
        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "12345"}),
            signature: "sha256=abcd1234".to_string(),
        };

        let hash1 = WebhookDeduplicator::compute_hash(&webhook);
        let hash2 = WebhookDeduplicator::compute_hash(&webhook);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_compute_hash_different_payloads() {
        let webhook1 = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "12345"}),
            signature: "sha256=abcd1234".to_string(),
        };

        let webhook2 = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "67890"}),
            signature: "sha256=abcd1234".to_string(),
        };

        let hash1 = WebhookDeduplicator::compute_hash(&webhook1);
        let hash2 = WebhookDeduplicator::compute_hash(&webhook2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_ignores_signature() {
        let webhook1 = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "12345"}),
            signature: "sha256=signature1".to_string(),
        };

        let webhook2 = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: webhook1.timestamp,
            payload: serde_json::json!({"content_id": "12345"}),
            signature: "sha256=signature2".to_string(),
        };

        let hash1 = WebhookDeduplicator::compute_hash(&webhook1);
        let hash2 = WebhookDeduplicator::compute_hash(&webhook2);

        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_deduplication_with_redis() {
        // Skip if Redis not available
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let deduplicator = match WebhookDeduplicator::new(&redis_url, Some(1)) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "test-dedup"}),
            signature: "sha256=test".to_string(),
        };

        // First check should return false (not duplicate)
        let is_dup = deduplicator.is_duplicate(&webhook).await.unwrap();
        assert!(!is_dup);

        // Mark as processed
        let hash = deduplicator.mark_processed(&webhook).await.unwrap();
        assert!(!hash.is_empty());

        // Second check should return true (duplicate)
        let is_dup = deduplicator.is_duplicate(&webhook).await.unwrap();
        assert!(is_dup);

        // Clean up
        let mut conn = deduplicator.client.get_async_connection().await.unwrap();
        let key = format!("webhook:hash:{}", hash);
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();
    }
}
