//! Platform webhook integration system
//!
//! This module provides webhook receiving, validation, processing, and queueing
//! for real-time content updates from streaming platforms.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod api;
pub mod deduplication;
pub mod handlers;
pub mod metrics;
pub mod processor;
pub mod queue;
pub mod receiver;
pub mod verification;

pub use api::configure_routes;
pub use deduplication::WebhookDeduplicator;
pub use metrics::WebhookMetrics;
pub use processor::WebhookProcessor;
pub use queue::{QueueStats, RedisWebhookQueue, WebhookQueue};
pub use receiver::WebhookReceiver;
pub use verification::verify_hmac_signature;

/// Webhook errors
#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid payload: {0}")]
    InvalidPayload(String),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Rate limit exceeded for platform {0}")]
    RateLimitExceeded(String),

    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("Deduplication error: {0}")]
    DeduplicationError(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type WebhookResult<T> = std::result::Result<T, WebhookError>;

/// Webhook event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    ContentAdded,
    ContentUpdated,
    ContentRemoved,
}

/// Webhook payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type
    pub event_type: WebhookEventType,
    /// Platform identifier
    pub platform: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Platform-specific content data
    pub payload: serde_json::Value,
    /// HMAC signature
    pub signature: String,
}

/// Processed webhook event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedWebhook {
    /// Event ID (content hash)
    pub event_id: String,
    /// Original webhook payload
    pub webhook: WebhookPayload,
    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
    /// Processing status
    pub status: ProcessingStatus,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    DeadLettered,
}

/// Platform webhook configuration
#[derive(Debug, Clone)]
pub struct PlatformWebhookConfig {
    /// Platform identifier
    pub platform: String,
    /// HMAC secret for signature verification
    pub secret: String,
    /// Rate limit (webhooks per minute)
    pub rate_limit: u32,
    /// Enable webhook processing
    pub enabled: bool,
}

/// Webhook registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRegistration {
    /// Platform identifier
    pub platform: String,
    /// Webhook URL to call
    pub url: String,
    /// Event types to subscribe to
    pub event_types: Vec<WebhookEventType>,
    /// HMAC secret
    pub secret: String,
}

/// Webhook receiver trait
///
/// Implementations handle platform-specific webhook payloads and convert them
/// to normalized format for processing.
#[async_trait]
pub trait WebhookHandler: Send + Sync {
    /// Get the platform identifier
    fn platform_id(&self) -> &'static str;

    /// Verify webhook signature
    fn verify_signature(
        &self,
        payload: &[u8],
        signature: &str,
        secret: &str,
    ) -> WebhookResult<bool>;

    /// Parse and validate webhook payload
    fn parse_payload(&self, body: &[u8]) -> WebhookResult<WebhookPayload>;

    /// Process webhook event
    async fn process_event(&self, webhook: WebhookPayload) -> WebhookResult<ProcessedWebhook>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_type_serialization() {
        let event = WebhookEventType::ContentAdded;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""content_added""#);
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "12345"}),
            signature: "sha256=abcd1234".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: WebhookPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.event_type, WebhookEventType::ContentAdded);
        assert_eq!(deserialized.platform, "netflix");
    }

    #[test]
    fn test_processing_status_serialization() {
        let status = ProcessingStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""completed""#);
    }
}
