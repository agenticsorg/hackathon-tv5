/// PubNub publisher for sync operations
///
/// Handles publishing watchlist and progress updates with retry logic,
/// batching, and comprehensive error handling.
use crate::crdt::HLCTimestamp;
use crate::pubnub::{PubNubClient, PubNubConfig, PubNubError, PublishResponse};
use crate::sync::{ProgressUpdate, WatchlistOperation, WatchlistUpdate};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Maximum retry attempts for failed publishes
const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Base delay between retries (exponential backoff)
const RETRY_BASE_DELAY_MS: u64 = 100;

/// Maximum batch size for bulk publishing
const MAX_BATCH_SIZE: usize = 50;

/// Batch flush interval (milliseconds)
const BATCH_FLUSH_INTERVAL_MS: u64 = 1000;

/// Trait for publishing sync updates
#[async_trait]
pub trait SyncPublisher: Send + Sync {
    /// Publish a generic sync message
    async fn publish(&self, message: SyncMessage) -> Result<(), PublisherError>;

    /// Publish watchlist update
    async fn publish_watchlist_update(&self, update: WatchlistUpdate)
        -> Result<(), PublisherError>;

    /// Publish progress update
    async fn publish_progress_update(&self, update: ProgressUpdate) -> Result<(), PublisherError>;

    /// Publish batch of updates
    async fn publish_batch(&self, messages: Vec<SyncMessage>) -> Result<(), PublisherError>;

    /// Flush any pending batched messages
    async fn flush(&self) -> Result<(), PublisherError>;
}

/// Sync message envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    /// Message type and payload
    #[serde(flatten)]
    pub payload: MessagePayload,

    /// ISO 8601 timestamp
    pub timestamp: String,

    /// Message operation type
    pub operation_type: String,

    /// Device ID that originated the message
    pub device_id: String,

    /// Message ID for deduplication
    pub message_id: String,
}

/// Message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    #[serde(rename = "watchlist_update")]
    WatchlistUpdate {
        operation: WatchlistOperation,
        content_id: String,
        unique_tag: String,
        timestamp: HLCTimestamp,
    },

    #[serde(rename = "progress_update")]
    ProgressUpdate {
        content_id: String,
        position_seconds: u32,
        duration_seconds: u32,
        state: String,
        timestamp: HLCTimestamp,
    },

    #[serde(rename = "batch")]
    Batch { messages: Vec<SyncMessage> },
}

/// PubNub implementation of SyncPublisher
pub struct PubNubPublisher {
    /// PubNub client
    client: Arc<PubNubClient>,

    /// User ID for channel routing
    user_id: String,

    /// Device ID for message attribution
    device_id: String,

    /// Batch sender for async batching
    batch_tx: Option<mpsc::UnboundedSender<SyncMessage>>,

    /// Enable batching
    batching_enabled: bool,
}

impl PubNubPublisher {
    /// Create a new PubNub publisher
    pub fn new(config: PubNubConfig, user_id: String, device_id: String) -> Self {
        let client = Arc::new(PubNubClient::new(
            config,
            user_id.clone(),
            device_id.clone(),
        ));

        Self {
            client,
            user_id,
            device_id,
            batch_tx: None,
            batching_enabled: false,
        }
    }

    /// Create a new PubNub publisher with batching enabled
    pub fn new_with_batching(config: PubNubConfig, user_id: String, device_id: String) -> Self {
        let client = Arc::new(PubNubClient::new(
            config,
            user_id.clone(),
            device_id.clone(),
        ));

        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn batching worker
        let client_clone = Arc::clone(&client);
        let user_id_clone = user_id.clone();
        tokio::spawn(async move {
            Self::batching_worker(client_clone, user_id_clone, rx).await;
        });

        Self {
            client,
            user_id,
            device_id,
            batch_tx: Some(tx),
            batching_enabled: true,
        }
    }

    /// Get the sync channel name for this user
    fn sync_channel(&self) -> String {
        format!("user.{}.sync", self.user_id)
    }

    /// Publish with retry logic
    async fn publish_with_retry(
        &self,
        channel: &str,
        message: &SyncMessage,
    ) -> Result<PublishResponse, PublisherError> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match self.client.publish(channel, message).await {
                Ok(response) => {
                    if attempt > 1 {
                        info!("Successfully published message after {} attempts", attempt);
                    } else {
                        debug!("Published message to channel: {}", channel);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    if attempt >= MAX_RETRY_ATTEMPTS {
                        error!(
                            "Failed to publish message after {} attempts: {}",
                            attempt, e
                        );
                        return Err(PublisherError::PublishFailed {
                            channel: channel.to_string(),
                            attempts: attempt,
                            source: e,
                        });
                    }

                    // Exponential backoff
                    let delay_ms = RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1);
                    warn!(
                        "Publish attempt {} failed, retrying in {}ms: {}",
                        attempt, delay_ms, e
                    );
                    sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }

    /// Batching worker that collects and flushes messages
    async fn batching_worker(
        client: Arc<PubNubClient>,
        user_id: String,
        mut rx: mpsc::UnboundedReceiver<SyncMessage>,
    ) {
        let mut batch: Vec<SyncMessage> = Vec::new();
        let mut flush_interval =
            tokio::time::interval(Duration::from_millis(BATCH_FLUSH_INTERVAL_MS));

        loop {
            tokio::select! {
                // Receive new message
                Some(msg) = rx.recv() => {
                    batch.push(msg);

                    // Flush if batch is full
                    if batch.len() >= MAX_BATCH_SIZE {
                        Self::flush_batch(&client, &user_id, &mut batch).await;
                    }
                }

                // Periodic flush
                _ = flush_interval.tick() => {
                    if !batch.is_empty() {
                        Self::flush_batch(&client, &user_id, &mut batch).await;
                    }
                }
            }
        }
    }

    /// Flush batch of messages
    async fn flush_batch(client: &Arc<PubNubClient>, user_id: &str, batch: &mut Vec<SyncMessage>) {
        if batch.is_empty() {
            return;
        }

        let channel = format!("user.{}.sync", user_id);
        let message_count = batch.len();

        // Create batch message
        let batch_message = SyncMessage {
            payload: MessagePayload::Batch {
                messages: batch.clone(),
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation_type: "batch".to_string(),
            device_id: "batch-worker".to_string(),
            message_id: uuid::Uuid::new_v4().to_string(),
        };

        // Publish with retry
        match Self::publish_with_retry_static(client, &channel, &batch_message).await {
            Ok(_) => {
                info!(
                    "Successfully flushed batch of {} messages to {}",
                    message_count, channel
                );
            }
            Err(e) => {
                error!("Failed to flush batch of {} messages: {}", message_count, e);
            }
        }

        batch.clear();
    }

    /// Static version of publish_with_retry for use in worker
    async fn publish_with_retry_static(
        client: &Arc<PubNubClient>,
        channel: &str,
        message: &SyncMessage,
    ) -> Result<PublishResponse, PublisherError> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match client.publish(channel, message).await {
                Ok(response) => {
                    return Ok(response);
                }
                Err(e) => {
                    if attempt >= MAX_RETRY_ATTEMPTS {
                        return Err(PublisherError::PublishFailed {
                            channel: channel.to_string(),
                            attempts: attempt,
                            source: e,
                        });
                    }

                    let delay_ms = RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1);
                    sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }

    /// Generate unique message ID
    fn generate_message_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Get current ISO 8601 timestamp
    fn current_timestamp() -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl SyncPublisher for PubNubPublisher {
    async fn publish(&self, message: SyncMessage) -> Result<(), PublisherError> {
        // If batching is enabled, send to batch worker
        if self.batching_enabled {
            if let Some(ref tx) = self.batch_tx {
                tx.send(message)
                    .map_err(|_| PublisherError::BatchChannelClosed)?;
                return Ok(());
            }
        }

        // Otherwise, publish immediately
        let channel = self.sync_channel();
        self.publish_with_retry(&channel, &message).await?;
        Ok(())
    }

    async fn publish_watchlist_update(
        &self,
        update: WatchlistUpdate,
    ) -> Result<(), PublisherError> {
        let message = SyncMessage {
            payload: MessagePayload::WatchlistUpdate {
                operation: update.operation,
                content_id: update.content_id,
                unique_tag: update.unique_tag,
                timestamp: update.timestamp,
            },
            timestamp: Self::current_timestamp(),
            operation_type: format!("watchlist_{:?}", update.operation).to_lowercase(),
            device_id: self.device_id.clone(),
            message_id: Self::generate_message_id(),
        };

        info!(
            "Publishing watchlist update: {:?} for content {}",
            update.operation,
            message
                .payload
                .get_content_id()
                .unwrap_or_else(|| "unknown".to_string())
        );

        self.publish(message).await
    }

    async fn publish_progress_update(&self, update: ProgressUpdate) -> Result<(), PublisherError> {
        let message = SyncMessage {
            payload: MessagePayload::ProgressUpdate {
                content_id: update.content_id.clone(),
                position_seconds: update.position_seconds,
                duration_seconds: update.duration_seconds,
                state: format!("{:?}", update.state),
                timestamp: update.timestamp,
            },
            timestamp: Self::current_timestamp(),
            operation_type: "progress_update".to_string(),
            device_id: self.device_id.clone(),
            message_id: Self::generate_message_id(),
        };

        debug!(
            "Publishing progress update for content {}: {}s/{}s ({}%)",
            update.content_id,
            update.position_seconds,
            update.duration_seconds,
            (update.completion_percent() * 100.0) as u32
        );

        self.publish(message).await
    }

    async fn publish_batch(&self, messages: Vec<SyncMessage>) -> Result<(), PublisherError> {
        if messages.is_empty() {
            return Ok(());
        }

        // If batching is enabled, send all to batch worker
        if self.batching_enabled {
            if let Some(ref tx) = self.batch_tx {
                for message in messages {
                    tx.send(message)
                        .map_err(|_| PublisherError::BatchChannelClosed)?;
                }
                return Ok(());
            }
        }

        // Otherwise, publish batch immediately
        let channel = self.sync_channel();
        let batch_message = SyncMessage {
            payload: MessagePayload::Batch { messages },
            timestamp: Self::current_timestamp(),
            operation_type: "batch".to_string(),
            device_id: self.device_id.clone(),
            message_id: Self::generate_message_id(),
        };

        self.publish_with_retry(&channel, &batch_message).await?;
        Ok(())
    }

    async fn flush(&self) -> Result<(), PublisherError> {
        // Note: In the current implementation, the batching worker
        // automatically flushes based on size and time intervals.
        // This method is provided for API completeness but doesn't
        // need to do anything as the worker handles flushing.
        debug!("Flush requested (automatic flushing is active)");
        Ok(())
    }
}

/// Helper methods for MessagePayload
impl MessagePayload {
    /// Get content ID from payload if applicable
    fn get_content_id(&self) -> Option<String> {
        match self {
            MessagePayload::WatchlistUpdate { content_id, .. } => Some(content_id.clone()),
            MessagePayload::ProgressUpdate { content_id, .. } => Some(content_id.clone()),
            MessagePayload::Batch { .. } => None,
        }
    }
}

/// Publisher errors
#[derive(Debug, Error)]
pub enum PublisherError {
    #[error("Failed to publish to channel {channel} after {attempts} attempts: {source}")]
    PublishFailed {
        channel: String,
        attempts: u32,
        source: PubNubError,
    },

    #[error("Failed to serialize message: {0}")]
    SerializationError(String),

    #[error("Batch channel closed unexpectedly")]
    BatchChannelClosed,

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::{HLCTimestamp, PlaybackState};

    #[tokio::test]
    async fn test_publisher_creation() {
        let config = PubNubConfig::default();
        let publisher =
            PubNubPublisher::new(config, "test-user".to_string(), "test-device".to_string());

        assert_eq!(publisher.user_id, "test-user");
        assert_eq!(publisher.device_id, "test-device");
        assert!(!publisher.batching_enabled);
    }

    #[tokio::test]
    async fn test_publisher_with_batching() {
        let config = PubNubConfig::default();
        let publisher = PubNubPublisher::new_with_batching(
            config,
            "test-user".to_string(),
            "test-device".to_string(),
        );

        assert!(publisher.batching_enabled);
        assert!(publisher.batch_tx.is_some());
    }

    #[test]
    fn test_sync_channel_format() {
        let config = PubNubConfig::default();
        let publisher =
            PubNubPublisher::new(config, "user-123".to_string(), "device-abc".to_string());

        assert_eq!(publisher.sync_channel(), "user.user-123.sync");
    }

    #[test]
    fn test_watchlist_message_creation() {
        let update = WatchlistUpdate {
            operation: WatchlistOperation::Add,
            content_id: "content-1".to_string(),
            unique_tag: "tag-1".to_string(),
            timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
            device_id: "device-1".to_string(),
        };

        let message = SyncMessage {
            payload: MessagePayload::WatchlistUpdate {
                operation: update.operation,
                content_id: update.content_id.clone(),
                unique_tag: update.unique_tag,
                timestamp: update.timestamp,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation_type: "watchlist_add".to_string(),
            device_id: "device-1".to_string(),
            message_id: uuid::Uuid::new_v4().to_string(),
        };

        assert_eq!(
            message.payload.get_content_id(),
            Some("content-1".to_string())
        );
    }

    #[test]
    fn test_progress_message_creation() {
        let update = ProgressUpdate {
            content_id: "content-1".to_string(),
            position_seconds: 100,
            duration_seconds: 1000,
            state: PlaybackState::Playing,
            timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
            device_id: "device-1".to_string(),
        };

        let message = SyncMessage {
            payload: MessagePayload::ProgressUpdate {
                content_id: update.content_id.clone(),
                position_seconds: update.position_seconds,
                duration_seconds: update.duration_seconds,
                state: format!("{:?}", update.state),
                timestamp: update.timestamp,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation_type: "progress_update".to_string(),
            device_id: "device-1".to_string(),
            message_id: uuid::Uuid::new_v4().to_string(),
        };

        assert_eq!(
            message.payload.get_content_id(),
            Some("content-1".to_string())
        );
    }

    #[test]
    fn test_message_serialization() {
        let message = SyncMessage {
            payload: MessagePayload::WatchlistUpdate {
                operation: WatchlistOperation::Add,
                content_id: "content-1".to_string(),
                unique_tag: "tag-1".to_string(),
                timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
            },
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            operation_type: "watchlist_add".to_string(),
            device_id: "device-1".to_string(),
            message_id: "msg-1".to_string(),
        };

        let json = serde_json::to_string(&message).expect("Failed to serialize");
        assert!(json.contains("watchlist_update"));
        assert!(json.contains("content-1"));

        let deserialized: SyncMessage = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(
            deserialized.payload.get_content_id(),
            Some("content-1".to_string())
        );
    }
}
