//! User Activity Event Stream
//!
//! Unified activity event system for tracking user interactions across the platform:
//! - Discovery: Searches, views, ratings
//! - Playback: Start, pause, complete, abandon
//! - Auth: Login, logout, profile updates
//!
//! Events are published to Kafka for downstream processing (analytics, recommendations, SONA).

use chrono::{DateTime, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Activity event errors
#[derive(Debug, thiserror::Error)]
pub enum ActivityEventError {
    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Duplicate event: {0}")]
    DuplicateEvent(String),

    #[error("Invalid event data: {0}")]
    InvalidEvent(String),
}

pub type ActivityEventResult<T> = Result<T, ActivityEventError>;

/// Types of user activity events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActivityEventType {
    // Discovery events
    SearchQuery,
    SearchResultClick,
    ContentView,
    ContentRating,

    // Playback events
    PlaybackStart,
    PlaybackPause,
    PlaybackResume,
    PlaybackComplete,
    PlaybackAbandon,

    // Auth events
    UserLogin,
    UserLogout,
    ProfileUpdate,
    PreferenceChange,
}

impl ActivityEventType {
    /// Get the string representation for Kafka topic naming
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityEventType::SearchQuery => "search_query",
            ActivityEventType::SearchResultClick => "search_result_click",
            ActivityEventType::ContentView => "content_view",
            ActivityEventType::ContentRating => "content_rating",
            ActivityEventType::PlaybackStart => "playback_start",
            ActivityEventType::PlaybackPause => "playback_pause",
            ActivityEventType::PlaybackResume => "playback_resume",
            ActivityEventType::PlaybackComplete => "playback_complete",
            ActivityEventType::PlaybackAbandon => "playback_abandon",
            ActivityEventType::UserLogin => "user_login",
            ActivityEventType::UserLogout => "user_logout",
            ActivityEventType::ProfileUpdate => "profile_update",
            ActivityEventType::PreferenceChange => "preference_change",
        }
    }
}

/// User activity event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivityEvent {
    /// Unique event identifier for deduplication
    pub event_id: Uuid,

    /// User who performed the activity
    pub user_id: Uuid,

    /// Type of activity event
    pub event_type: ActivityEventType,

    /// Content ID if applicable (None for auth events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,

    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,

    /// Additional metadata (query, results_count, position, etc.)
    pub metadata: serde_json::Value,

    /// Device ID or session context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,

    /// Geographic region (ISO 3166-1 alpha-2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

impl UserActivityEvent {
    /// Create a new user activity event
    pub fn new(user_id: Uuid, event_type: ActivityEventType, metadata: serde_json::Value) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            user_id,
            event_type,
            content_id: None,
            timestamp: Utc::now(),
            metadata,
            device_id: None,
            region: None,
        }
    }

    /// Set the content ID for content-related events
    pub fn with_content_id(mut self, content_id: impl Into<String>) -> Self {
        self.content_id = Some(content_id.into());
        self
    }

    /// Set the device ID
    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    /// Set the region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Validate the event data
    pub fn validate(&self) -> ActivityEventResult<()> {
        // Content events should have content_id
        match self.event_type {
            ActivityEventType::SearchResultClick
            | ActivityEventType::ContentView
            | ActivityEventType::ContentRating
            | ActivityEventType::PlaybackStart
            | ActivityEventType::PlaybackPause
            | ActivityEventType::PlaybackResume
            | ActivityEventType::PlaybackComplete
            | ActivityEventType::PlaybackAbandon => {
                if self.content_id.is_none() {
                    return Err(ActivityEventError::InvalidEvent(format!(
                        "Event type {:?} requires content_id",
                        self.event_type
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Trait for user activity event producers
#[async_trait::async_trait]
pub trait UserActivityProducer: Send + Sync {
    /// Publish a user activity event
    async fn publish_activity(&self, event: UserActivityEvent) -> ActivityEventResult<()>;

    /// Publish multiple events in batch
    async fn publish_batch(&self, events: Vec<UserActivityEvent>) -> ActivityEventResult<()>;

    /// Health check
    async fn is_healthy(&self) -> bool;
}

/// Kafka producer for user activity events
#[derive(Clone)]
pub struct KafkaActivityProducer {
    producer: FutureProducer,
    topic: String,
    seen_events: Arc<Mutex<HashSet<Uuid>>>,
}

impl KafkaActivityProducer {
    /// Create a new Kafka activity producer
    pub fn new(brokers: &str, topic: String) -> ActivityEventResult<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.messages", "100000")
            .set("queue.buffering.max.kbytes", "1048576")
            .set("batch.num.messages", "10000")
            .set("linger.ms", "10")
            .set("compression.type", "snappy")
            .set("acks", "1")
            .set("enable.idempotence", "true")
            .create()
            .map_err(|e| ActivityEventError::ConfigError(e.to_string()))?;

        info!(
            topic = %topic,
            brokers = %brokers,
            "Initialized Kafka activity event producer"
        );

        Ok(Self {
            producer,
            topic,
            seen_events: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    /// Create from environment variables
    pub fn from_env() -> ActivityEventResult<Self> {
        let brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

        let topic_prefix =
            env::var("KAFKA_TOPIC_PREFIX").unwrap_or_else(|_| "media-gateway".to_string());

        let topic = format!("{}.user-activity", topic_prefix);

        Self::new(&brokers, topic)
    }

    /// Check if an event has been seen (for deduplication)
    async fn is_duplicate(&self, event_id: Uuid) -> bool {
        let mut seen = self.seen_events.lock().await;

        if seen.contains(&event_id) {
            true
        } else {
            seen.insert(event_id);

            // Limit memory: keep only last 10000 event IDs
            if seen.len() > 10000 {
                // Clear half to avoid frequent cleanups
                let to_remove: Vec<_> = seen.iter().take(5000).copied().collect();
                for id in to_remove {
                    seen.remove(&id);
                }
            }

            false
        }
    }

    /// Publish an event to Kafka
    async fn publish_to_kafka(&self, event: &UserActivityEvent) -> ActivityEventResult<()> {
        // Validate event
        event.validate()?;

        // Check for duplicates
        if self.is_duplicate(event.event_id).await {
            warn!(
                event_id = %event.event_id,
                user_id = %event.user_id,
                event_type = ?event.event_type,
                "Duplicate event detected, skipping"
            );
            return Err(ActivityEventError::DuplicateEvent(
                event.event_id.to_string(),
            ));
        }

        // Serialize event
        let payload = serde_json::to_vec(&event)?;
        let user_id_string = event.user_id.to_string();

        // Create Kafka record
        let record = FutureRecord::to(&self.topic)
            .key(&user_id_string)
            .payload(&payload);

        // Send with timeout
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| ActivityEventError::PublishFailed(err.to_string()))?;

        debug!(
            event_id = %event.event_id,
            user_id = %event.user_id,
            event_type = ?event.event_type,
            topic = %self.topic,
            "Published user activity event"
        );

        Ok(())
    }
}

#[async_trait::async_trait]
impl UserActivityProducer for KafkaActivityProducer {
    async fn publish_activity(&self, event: UserActivityEvent) -> ActivityEventResult<()> {
        self.publish_to_kafka(&event).await
    }

    async fn publish_batch(&self, events: Vec<UserActivityEvent>) -> ActivityEventResult<()> {
        info!(count = events.len(), "Publishing activity event batch");

        let mut errors = Vec::new();

        for event in events {
            if let Err(e) = self.publish_to_kafka(&event).await {
                error!(
                    event_id = %event.event_id,
                    error = %e,
                    "Failed to publish event in batch"
                );
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            return Err(ActivityEventError::PublishFailed(format!(
                "{} events failed to publish",
                errors.len()
            )));
        }

        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Kafka producer doesn't expose direct health check,
        // but we can assume it's healthy if initialized
        true
    }
}

/// No-op producer for testing
pub struct NoOpActivityProducer;

#[async_trait::async_trait]
impl UserActivityProducer for NoOpActivityProducer {
    async fn publish_activity(&self, event: UserActivityEvent) -> ActivityEventResult<()> {
        debug!(
            event_id = %event.event_id,
            user_id = %event.user_id,
            event_type = ?event.event_type,
            "NoOp: Would publish activity event"
        );
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<UserActivityEvent>) -> ActivityEventResult<()> {
        debug!(count = events.len(), "NoOp: Would publish activity batch");
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_event_creation() {
        let user_id = Uuid::new_v4();
        let metadata = serde_json::json!({
            "query": "action movies",
            "results_count": 42
        });

        let event = UserActivityEvent::new(user_id, ActivityEventType::SearchQuery, metadata)
            .with_device_id("device-123")
            .with_region("US");

        assert_eq!(event.user_id, user_id);
        assert_eq!(event.event_type, ActivityEventType::SearchQuery);
        assert_eq!(event.device_id, Some("device-123".to_string()));
        assert_eq!(event.region, Some("US".to_string()));
        assert!(event.content_id.is_none());
    }

    #[test]
    fn test_content_event_validation() {
        let user_id = Uuid::new_v4();
        let metadata = serde_json::json!({
            "position_seconds": 120
        });

        // PlaybackStart requires content_id
        let event = UserActivityEvent::new(user_id, ActivityEventType::PlaybackStart, metadata);

        assert!(event.validate().is_err());

        // With content_id should be valid
        let event = event.with_content_id("content-123");
        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_auth_event_no_content_id() {
        let user_id = Uuid::new_v4();
        let metadata = serde_json::json!({
            "device": "iPhone 12",
            "ip": "192.168.1.1"
        });

        let event = UserActivityEvent::new(user_id, ActivityEventType::UserLogin, metadata);

        // Auth events don't require content_id
        assert!(event.validate().is_ok());
        assert!(event.content_id.is_none());
    }

    #[test]
    fn test_event_serialization() {
        let user_id = Uuid::new_v4();
        let metadata = serde_json::json!({
            "rating": 4.5,
            "review": "Great movie!"
        });

        let event = UserActivityEvent::new(user_id, ActivityEventType::ContentRating, metadata)
            .with_content_id("movie-456");

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: UserActivityEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.user_id, event.user_id);
        assert_eq!(deserialized.event_type, event.event_type);
        assert_eq!(deserialized.content_id, event.content_id);
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(ActivityEventType::SearchQuery.as_str(), "search_query");
        assert_eq!(ActivityEventType::PlaybackStart.as_str(), "playback_start");
        assert_eq!(ActivityEventType::UserLogin.as_str(), "user_login");
    }

    #[tokio::test]
    async fn test_noop_producer() {
        let producer = NoOpActivityProducer;
        let user_id = Uuid::new_v4();
        let metadata = serde_json::json!({"test": true});

        let event = UserActivityEvent::new(user_id, ActivityEventType::SearchQuery, metadata);

        assert!(producer.publish_activity(event).await.is_ok());
        assert!(producer.is_healthy().await);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        // Create a mock producer for testing
        let producer = KafkaActivityProducer {
            producer: ClientConfig::new()
                .set("bootstrap.servers", "localhost:9092")
                .create()
                .unwrap(),
            topic: "test-topic".to_string(),
            seen_events: Arc::new(Mutex::new(HashSet::new())),
        };

        let event_id = Uuid::new_v4();

        // First check should return false (not a duplicate)
        assert!(!producer.is_duplicate(event_id).await);

        // Second check should return true (is a duplicate)
        assert!(producer.is_duplicate(event_id).await);
    }
}
