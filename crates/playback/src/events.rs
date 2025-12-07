//! Playback event publishing
//!
//! Publishes playback events to Kafka for downstream processing
//! (analytics, recommendations, sync, etc.)
//!
//! Also publishes unified user activity events via core event system.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use media_gateway_core::{
    ActivityEventType, KafkaActivityProducer, UserActivityEvent, UserActivityProducer,
};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Playback event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum PlaybackEvent {
    SessionCreated(SessionCreatedEvent),
    PositionUpdated(PositionUpdatedEvent),
    SessionEnded(SessionEndedEvent),
}

/// Session created event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreatedEvent {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub device_id: String,
    pub duration_seconds: u32,
    pub quality: String,
    pub timestamp: DateTime<Utc>,
}

/// Position updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdatedEvent {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub device_id: String,
    pub position_seconds: u32,
    pub playback_state: String,
    pub timestamp: DateTime<Utc>,
}

/// Session ended event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndedEvent {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub device_id: String,
    pub final_position_seconds: u32,
    pub duration_seconds: u32,
    pub completion_rate: f32,
    pub timestamp: DateTime<Utc>,
}

/// Playback event producer trait
#[async_trait::async_trait]
pub trait PlaybackEventProducer: Send + Sync {
    async fn publish_session_created(&self, event: SessionCreatedEvent) -> Result<()>;
    async fn publish_position_updated(&self, event: PositionUpdatedEvent) -> Result<()>;
    async fn publish_session_ended(&self, event: SessionEndedEvent) -> Result<()>;
}

/// Kafka-based playback event producer
pub struct KafkaPlaybackProducer {
    producer: FutureProducer,
    topic_prefix: String,
    activity_producer: Option<KafkaActivityProducer>,
}

impl KafkaPlaybackProducer {
    /// Create new Kafka producer
    pub fn new(brokers: &str, topic_prefix: String) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.messages", "10000")
            .set("queue.buffering.max.kbytes", "1048576")
            .set("batch.num.messages", "10000")
            .set("linger.ms", "10")
            .set("compression.type", "snappy")
            .set("acks", "1") // Wait for leader acknowledgment only
            .create()
            .context("Failed to create Kafka producer")?;

        // Initialize unified activity producer (optional)
        let activity_producer = match KafkaActivityProducer::from_env() {
            Ok(producer) => Some(producer),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to initialize activity event producer");
                None
            }
        };

        Ok(Self {
            producer,
            topic_prefix,
            activity_producer,
        })
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let brokers =
            std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        let topic_prefix =
            std::env::var("KAFKA_TOPIC_PREFIX").unwrap_or_else(|_| "playback".to_string());

        Self::new(&brokers, topic_prefix)
    }

    /// Publish event to Kafka
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> Result<()> {
        let record = FutureRecord::to(topic).key(key).payload(payload);

        // Send with timeout
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("Kafka send error: {}", err))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl PlaybackEventProducer for KafkaPlaybackProducer {
    async fn publish_session_created(&self, event: SessionCreatedEvent) -> Result<()> {
        let topic = format!("{}.session-created", self.topic_prefix);
        let key = event.session_id.to_string();
        let payload =
            serde_json::to_vec(&event).context("Failed to serialize session created event")?;

        self.publish(&topic, &key, &payload).await?;

        tracing::debug!(
            "Published session created event: session_id={}, user_id={}",
            event.session_id,
            event.user_id
        );

        // Publish unified user activity event (non-blocking, fire and forget)
        if let Some(producer) = self.activity_producer.clone() {
            let user_id = event.user_id;
            let content_id = event.content_id.to_string();
            let device_id = event.device_id.clone();
            let session_id = event.session_id.to_string();
            let duration = event.duration_seconds;
            let quality = event.quality.clone();

            tokio::spawn(async move {
                let metadata = serde_json::json!({
                    "session_id": session_id,
                    "device_id": &device_id,
                    "duration_seconds": duration,
                    "quality": quality,
                });

                let activity_event =
                    UserActivityEvent::new(user_id, ActivityEventType::PlaybackStart, metadata)
                        .with_content_id(content_id)
                        .with_device_id(device_id);

                if let Err(e) = producer.publish_activity(activity_event).await {
                    tracing::warn!(error = %e, "Failed to publish playback start activity event");
                }
            });
        }

        Ok(())
    }

    async fn publish_position_updated(&self, event: PositionUpdatedEvent) -> Result<()> {
        let topic = format!("{}.position-updated", self.topic_prefix);
        let key = event.session_id.to_string();
        let payload =
            serde_json::to_vec(&event).context("Failed to serialize position updated event")?;

        self.publish(&topic, &key, &payload).await?;

        tracing::trace!(
            "Published position updated event: session_id={}, position={}",
            event.session_id,
            event.position_seconds
        );

        Ok(())
    }

    async fn publish_session_ended(&self, event: SessionEndedEvent) -> Result<()> {
        let topic = format!("{}.session-ended", self.topic_prefix);
        let key = event.session_id.to_string();
        let payload =
            serde_json::to_vec(&event).context("Failed to serialize session ended event")?;

        self.publish(&topic, &key, &payload).await?;

        tracing::info!(
            "Published session ended event: session_id={}, completion_rate={}",
            event.session_id,
            event.completion_rate
        );

        // Publish unified user activity event (non-blocking, fire and forget)
        if let Some(producer) = self.activity_producer.clone() {
            let user_id = event.user_id;
            let content_id = event.content_id.to_string();
            let device_id = event.device_id.clone();
            let session_id = event.session_id.to_string();
            let final_position = event.final_position_seconds;
            let duration = event.duration_seconds;
            let completion = event.completion_rate;

            tokio::spawn(async move {
                let activity_type = if completion >= 0.9 {
                    ActivityEventType::PlaybackComplete
                } else {
                    ActivityEventType::PlaybackAbandon
                };

                let metadata = serde_json::json!({
                    "session_id": session_id,
                    "device_id": &device_id,
                    "final_position_seconds": final_position,
                    "duration_seconds": duration,
                    "completion_rate": completion,
                });

                let activity_event = UserActivityEvent::new(user_id, activity_type, metadata)
                    .with_content_id(content_id)
                    .with_device_id(device_id);

                if let Err(e) = producer.publish_activity(activity_event).await {
                    tracing::warn!(error = %e, "Failed to publish playback end activity event");
                }
            });
        }

        Ok(())
    }
}

/// No-op producer for testing
pub struct NoOpProducer;

#[async_trait::async_trait]
impl PlaybackEventProducer for NoOpProducer {
    async fn publish_session_created(&self, _event: SessionCreatedEvent) -> Result<()> {
        Ok(())
    }

    async fn publish_position_updated(&self, _event: PositionUpdatedEvent) -> Result<()> {
        Ok(())
    }

    async fn publish_session_ended(&self, _event: SessionEndedEvent) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_producer() {
        let producer = NoOpProducer;

        let event = SessionCreatedEvent {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content_id: Uuid::new_v4(),
            device_id: "test-device".to_string(),
            duration_seconds: 3600,
            quality: "high".to_string(),
            timestamp: Utc::now(),
        };

        assert!(producer.publish_session_created(event).await.is_ok());
    }

    #[test]
    fn test_event_serialization() {
        let event = PositionUpdatedEvent {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content_id: Uuid::new_v4(),
            device_id: "test-device".to_string(),
            position_seconds: 120,
            playback_state: "playing".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("position_seconds"));
        assert!(json.contains("playback_state"));
    }
}
