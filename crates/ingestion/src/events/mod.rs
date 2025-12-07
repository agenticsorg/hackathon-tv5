//! Kafka Event Streaming for Content Lifecycle Events
//!
//! This module provides event streaming capabilities for tracking content lifecycle
//! events through Kafka, including ingestion, updates, availability changes, and
//! metadata enrichment.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub mod kafka_producer;
pub mod metrics;

#[cfg(test)]
mod mock;

#[cfg(test)]
pub use mock::MockEventProducer;

#[cfg(not(test))]
pub use kafka_producer::KafkaEventProducer;

/// Event streaming errors
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Broker unavailable: {0}")]
    BrokerUnavailable(String),

    #[error("Delivery confirmation failed: {0}")]
    DeliveryFailed(String),

    #[error("Invalid event data: {0}")]
    InvalidEvent(String),
}

pub type EventResult<T> = Result<T, EventError>;

/// Base event structure shared across all event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEvent {
    /// Type of the event (e.g., "content.ingested", "content.updated")
    pub event_type: String,

    /// Unique identifier for the content
    pub content_id: Uuid,

    /// Timestamp when the event was created (ISO8601 format)
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,
}

impl BaseEvent {
    /// Creates a new base event with the given type and content ID
    pub fn new(event_type: impl Into<String>, content_id: Uuid) -> Self {
        Self {
            event_type: event_type.into(),
            content_id,
            timestamp: Utc::now(),
            correlation_id: Uuid::new_v4(),
        }
    }

    /// Sets the correlation ID for tracing
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = correlation_id;
        self
    }
}

/// Content ingestion event - fired when new content is ingested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentIngestedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,

    /// Platform the content was ingested from
    pub platform: String,

    /// External ID from the platform
    pub external_id: String,

    /// Content type (movie, tv_show, music, etc.)
    pub content_type: String,

    /// Title of the content
    pub title: String,

    /// Metadata fields that were populated
    pub metadata_fields: Vec<String>,
}

impl ContentIngestedEvent {
    /// Creates a new content ingested event
    pub fn new(
        content_id: Uuid,
        platform: String,
        external_id: String,
        content_type: String,
        title: String,
        metadata_fields: Vec<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new("content.ingested", content_id),
            platform,
            external_id,
            content_type,
            title,
            metadata_fields,
        }
    }
}

/// Content update event - fired when existing content is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentUpdatedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,

    /// Fields that were updated
    pub updated_fields: Vec<String>,

    /// Source of the update (platform, enrichment, manual, etc.)
    pub update_source: String,

    /// Previous version timestamp
    pub previous_version: Option<DateTime<Utc>>,

    /// Reason for the update
    pub reason: Option<String>,
}

impl ContentUpdatedEvent {
    /// Creates a new content updated event
    pub fn new(content_id: Uuid, updated_fields: Vec<String>, update_source: String) -> Self {
        Self {
            base: BaseEvent::new("content.updated", content_id),
            updated_fields,
            update_source,
            previous_version: None,
            reason: None,
        }
    }

    /// Sets the previous version timestamp
    pub fn with_previous_version(mut self, timestamp: DateTime<Utc>) -> Self {
        self.previous_version = Some(timestamp);
        self
    }

    /// Sets the update reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

/// Availability change event - fired when content availability changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityChangedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,

    /// Platform where availability changed
    pub platform: String,

    /// New availability status
    pub is_available: bool,

    /// Previous availability status
    pub was_available: bool,

    /// Geographic regions affected (ISO 3166-1 alpha-2 codes)
    pub regions: Vec<String>,

    /// Expiration date if content is now available
    pub expires_at: Option<DateTime<Utc>>,
}

impl AvailabilityChangedEvent {
    /// Creates a new availability changed event
    pub fn new(
        content_id: Uuid,
        platform: String,
        is_available: bool,
        was_available: bool,
        regions: Vec<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new("content.availability_changed", content_id),
            platform,
            is_available,
            was_available,
            regions,
            expires_at: None,
        }
    }

    /// Sets the expiration date
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

/// Metadata enrichment event - fired when content metadata is enriched
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEnrichedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,

    /// Source of the enrichment (tmdb, imdb, embeddings, etc.)
    pub enrichment_source: String,

    /// Fields that were enriched
    pub enriched_fields: Vec<String>,

    /// Confidence score of the enrichment (0.0 - 1.0)
    pub confidence_score: f64,

    /// Whether the enrichment required entity resolution
    pub entity_resolved: bool,

    /// Number of candidates considered during resolution
    pub resolution_candidates: Option<usize>,
}

impl MetadataEnrichedEvent {
    /// Creates a new metadata enriched event
    pub fn new(
        content_id: Uuid,
        enrichment_source: String,
        enriched_fields: Vec<String>,
        confidence_score: f64,
    ) -> Self {
        Self {
            base: BaseEvent::new("content.metadata_enriched", content_id),
            enrichment_source,
            enriched_fields,
            confidence_score,
            entity_resolved: false,
            resolution_candidates: None,
        }
    }

    /// Marks the event as having gone through entity resolution
    pub fn with_entity_resolution(mut self, candidates: usize) -> Self {
        self.entity_resolved = true;
        self.resolution_candidates = Some(candidates);
        self
    }
}

/// Event payload enum for type-safe event publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentEvent {
    Ingested(ContentIngestedEvent),
    Updated(ContentUpdatedEvent),
    AvailabilityChanged(AvailabilityChangedEvent),
    MetadataEnriched(MetadataEnrichedEvent),
}

impl ContentEvent {
    /// Gets the content ID from the event
    pub fn content_id(&self) -> Uuid {
        match self {
            ContentEvent::Ingested(e) => e.base.content_id,
            ContentEvent::Updated(e) => e.base.content_id,
            ContentEvent::AvailabilityChanged(e) => e.base.content_id,
            ContentEvent::MetadataEnriched(e) => e.base.content_id,
        }
    }

    /// Gets the event type string
    pub fn event_type(&self) -> &str {
        match self {
            ContentEvent::Ingested(e) => &e.base.event_type,
            ContentEvent::Updated(e) => &e.base.event_type,
            ContentEvent::AvailabilityChanged(e) => &e.base.event_type,
            ContentEvent::MetadataEnriched(e) => &e.base.event_type,
        }
    }

    /// Gets the correlation ID for tracing
    pub fn correlation_id(&self) -> Uuid {
        match self {
            ContentEvent::Ingested(e) => e.base.correlation_id,
            ContentEvent::Updated(e) => e.base.correlation_id,
            ContentEvent::AvailabilityChanged(e) => e.base.correlation_id,
            ContentEvent::MetadataEnriched(e) => e.base.correlation_id,
        }
    }
}

/// Trait for event producer implementations
#[async_trait::async_trait]
pub trait EventProducer: Send + Sync {
    /// Publishes an event to the event stream
    async fn publish_event(&self, event: ContentEvent) -> EventResult<()>;

    /// Publishes multiple events in a batch
    async fn publish_batch(&self, events: Vec<ContentEvent>) -> EventResult<()>;

    /// Checks if the producer is healthy and can publish events
    async fn health_check(&self) -> EventResult<bool>;
}

/// Configuration for the Kafka event producer
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    /// Comma-separated list of Kafka broker addresses
    pub brokers: String,

    /// Topic prefix for all events
    pub topic_prefix: String,

    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,

    /// Message timeout in milliseconds
    pub message_timeout_ms: u64,

    /// Whether to enable delivery confirmation
    pub enable_idempotence: bool,
}

impl KafkaConfig {
    /// Creates a new Kafka configuration from environment variables
    pub fn from_env() -> EventResult<Self> {
        const DEFAULT_TOPIC_PREFIX: &str = "media-gateway";

        let brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

        let topic_prefix =
            env::var("KAFKA_TOPIC_PREFIX").unwrap_or_else(|_| DEFAULT_TOPIC_PREFIX.to_string());

        let request_timeout_ms = env::var("KAFKA_REQUEST_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30000);

        let message_timeout_ms = env::var("KAFKA_MESSAGE_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60000);

        let enable_idempotence = env::var("KAFKA_ENABLE_IDEMPOTENCE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true);

        Ok(Self {
            brokers,
            topic_prefix,
            request_timeout_ms,
            message_timeout_ms,
            enable_idempotence,
        })
    }

    /// Gets the full topic name for an event type
    pub fn topic_for_event(&self, event_type: &str) -> String {
        format!("{}.{}", self.topic_prefix, event_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_event_creation() {
        let content_id = Uuid::new_v4();
        let event = BaseEvent::new("test.event", content_id);

        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.content_id, content_id);
        assert!(event.correlation_id != Uuid::nil());
    }

    #[test]
    fn test_content_ingested_event() {
        let content_id = Uuid::new_v4();
        let event = ContentIngestedEvent::new(
            content_id,
            "netflix".to_string(),
            "ext-123".to_string(),
            "movie".to_string(),
            "Test Movie".to_string(),
            vec!["title".to_string(), "year".to_string()],
        );

        assert_eq!(event.base.content_id, content_id);
        assert_eq!(event.platform, "netflix");
        assert_eq!(event.metadata_fields.len(), 2);
    }

    #[test]
    fn test_kafka_config_from_env() {
        env::set_var("KAFKA_BROKERS", "broker1:9092,broker2:9092");
        env::set_var("KAFKA_TOPIC_PREFIX", "test-prefix");

        let config = KafkaConfig::from_env().expect("Failed to create config");

        assert_eq!(config.brokers, "broker1:9092,broker2:9092");
        assert_eq!(config.topic_prefix, "test-prefix");

        env::remove_var("KAFKA_BROKERS");
        env::remove_var("KAFKA_TOPIC_PREFIX");
    }

    #[test]
    fn test_topic_naming() {
        let config = KafkaConfig {
            brokers: "localhost:9092".to_string(),
            topic_prefix: "media-gateway".to_string(),
            request_timeout_ms: 30000,
            message_timeout_ms: 60000,
            enable_idempotence: true,
        };

        assert_eq!(
            config.topic_for_event("content.ingested"),
            "media-gateway.content.ingested"
        );
    }

    #[tokio::test]
    async fn test_mock_producer_publish() {
        let config = KafkaConfig {
            brokers: "localhost:9092".to_string(),
            topic_prefix: "test".to_string(),
            request_timeout_ms: 30000,
            message_timeout_ms: 60000,
            enable_idempotence: true,
        };

        let producer = MockEventProducer::new(config);
        let content_id = Uuid::new_v4();

        let event = ContentEvent::Ingested(ContentIngestedEvent::new(
            content_id,
            "netflix".to_string(),
            "ext-123".to_string(),
            "movie".to_string(),
            "Test Movie".to_string(),
            vec!["title".to_string()],
        ));

        producer.publish_event(event.clone()).await.unwrap();

        let published = producer.get_published_events().await;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].content_id(), content_id);
    }

    #[tokio::test]
    async fn test_event_serialization() {
        let content_id = Uuid::new_v4();
        let event = ContentEvent::Updated(ContentUpdatedEvent::new(
            content_id,
            vec!["title".to_string(), "description".to_string()],
            "enrichment".to_string(),
        ));

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ContentEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content_id(), content_id);
    }
}
