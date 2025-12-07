use super::{ContentEvent, EventError, EventProducer, EventResult, KafkaConfig};
use std::sync::Arc;
use tracing::{debug, info};

/// Mock event producer for testing and development
pub struct MockEventProducer {
    config: KafkaConfig,
    published_events: Arc<tokio::sync::Mutex<Vec<ContentEvent>>>,
}

impl MockEventProducer {
    /// Creates a new mock event producer
    pub fn new(config: KafkaConfig) -> Self {
        Self {
            config,
            published_events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Gets all published events (for testing)
    pub async fn get_published_events(&self) -> Vec<ContentEvent> {
        self.published_events.lock().await.clone()
    }

    /// Clears all published events (for testing)
    pub async fn clear_events(&self) {
        self.published_events.lock().await.clear();
    }
}

#[async_trait::async_trait]
impl EventProducer for MockEventProducer {
    async fn publish_event(&self, event: ContentEvent) -> EventResult<()> {
        let topic = self.config.topic_for_event(event.event_type());

        info!(
            content_id = %event.content_id(),
            event_type = %event.event_type(),
            correlation_id = %event.correlation_id(),
            topic = %topic,
            "Publishing event (mock)"
        );

        // Simulate serialization
        let _payload = serde_json::to_string(&event)?;

        // Store event for testing
        self.published_events.lock().await.push(event);

        debug!(topic = %topic, "Event published successfully (mock)");
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<ContentEvent>) -> EventResult<()> {
        for event in events {
            self.publish_event(event).await?;
        }
        Ok(())
    }

    async fn health_check(&self) -> EventResult<bool> {
        // Mock producer is always healthy
        Ok(true)
    }
}
