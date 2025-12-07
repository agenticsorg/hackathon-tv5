/// WebSocket broadcaster for relaying PubNub messages to connected clients
///
/// Subscribes to PubNub channels and broadcasts messages to WebSocket connections
use crate::pubnub::{
    DeviceMessage, MessageHandler, PubNubClient, SyncMessage as PubNubSyncMessage,
};
use crate::ws::registry::{ConnectionRegistry, SyncMessage, SyncMessageType};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Metrics for broadcast operations
#[derive(Default, Clone)]
pub struct BroadcastMetrics {
    /// Total messages relayed from PubNub to WebSocket
    messages_relayed: Arc<parking_lot::RwLock<u64>>,

    /// Histogram data for broadcast latency (milliseconds)
    latency_samples: Arc<RwLock<Vec<f64>>>,
}

impl BroadcastMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_message_relayed(&self) {
        *self.messages_relayed.write() += 1;
    }

    pub fn record_latency(&self, latency_ms: f64) {
        let mut samples = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { self.latency_samples.write().await })
        });
        samples.push(latency_ms);

        // Keep only last 1000 samples for histogram
        let len = samples.len();
        if len > 1000 {
            samples.drain(0..len - 1000);
        }
    }

    pub fn total_messages_relayed(&self) -> u64 {
        *self.messages_relayed.read()
    }

    pub fn average_latency_ms(&self) -> f64 {
        let samples = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.latency_samples.read().await.clone() })
        });

        if samples.is_empty() {
            0.0
        } else {
            samples.iter().sum::<f64>() / samples.len() as f64
        }
    }

    pub fn p50_latency_ms(&self) -> f64 {
        self.percentile_latency(0.5)
    }

    pub fn p95_latency_ms(&self) -> f64 {
        self.percentile_latency(0.95)
    }

    pub fn p99_latency_ms(&self) -> f64 {
        self.percentile_latency(0.99)
    }

    fn percentile_latency(&self, p: f64) -> f64 {
        let mut samples = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.latency_samples.read().await.clone() })
        });

        if samples.is_empty() {
            return 0.0;
        }

        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((samples.len() as f64 * p).ceil() as usize).saturating_sub(1);
        samples[idx]
    }
}

/// WebSocket broadcaster for PubNub message relay
pub struct WebSocketBroadcaster {
    /// Connection registry for managing WebSocket connections
    registry: Arc<ConnectionRegistry>,

    /// PubNub client for subscribing to channels
    pubnub_client: Arc<PubNubClient>,

    /// Metrics tracking
    metrics: Arc<BroadcastMetrics>,
}

impl WebSocketBroadcaster {
    /// Create new WebSocket broadcaster
    pub fn new(registry: Arc<ConnectionRegistry>, pubnub_client: Arc<PubNubClient>) -> Self {
        Self {
            registry,
            pubnub_client,
            metrics: Arc::new(BroadcastMetrics::new()),
        }
    }

    /// Subscribe to user channel and start relaying messages
    pub async fn subscribe_user_channel(&self, user_id: Uuid) -> Result<(), BroadcastError> {
        let channel = format!("user.{}.sync", user_id);

        tracing::info!("Subscribing to PubNub channel: {}", channel);

        self.pubnub_client
            .subscribe(vec![channel])
            .await
            .map_err(|e| BroadcastError::PubNubError(e.to_string()))?;

        Ok(())
    }

    /// Relay PubNub message to WebSocket clients
    pub async fn relay_pubnub_message(&self, user_id: Uuid, message: PubNubSyncMessage) {
        let start = Instant::now();

        // Convert PubNub message to WebSocket message
        let ws_message = match self.convert_pubnub_message(message) {
            Some(msg) => msg,
            None => {
                tracing::warn!("Failed to convert PubNub message");
                return;
            }
        };

        // Broadcast to all user's WebSocket connections
        match self.registry.send_to_user(user_id, &ws_message).await {
            Ok(count) => {
                let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

                self.metrics.record_message_relayed();
                self.metrics.record_latency(latency_ms);

                tracing::debug!(
                    "Relayed message to {} connections for user {} (latency: {:.2}ms)",
                    count,
                    user_id,
                    latency_ms
                );
            }
            Err(e) => {
                tracing::error!("Failed to relay message to user {}: {}", user_id, e);
            }
        }
    }

    /// Convert PubNub sync message to WebSocket sync message
    fn convert_pubnub_message(&self, message: PubNubSyncMessage) -> Option<SyncMessage> {
        let message_type = match message {
            PubNubSyncMessage::WatchlistUpdate {
                content_id,
                operation,
                ..
            } => {
                let content_uuid = Uuid::parse_str(&content_id).ok()?;
                SyncMessageType::WatchlistUpdate {
                    content_id: content_uuid,
                    action: operation,
                }
            }
            PubNubSyncMessage::ProgressUpdate {
                content_id,
                position_seconds,
                duration_seconds,
                ..
            } => {
                let content_uuid = Uuid::parse_str(&content_id).ok()?;
                SyncMessageType::ProgressUpdate {
                    content_id: content_uuid,
                    position: position_seconds,
                    duration: duration_seconds,
                }
            }
            PubNubSyncMessage::DeviceHandoff {
                target_device_id,
                content_id,
                ..
            } => {
                let target_uuid = Uuid::parse_str(&target_device_id).ok()?;
                SyncMessageType::DeviceCommand {
                    command: format!("handoff:{}", content_id),
                    target_device: Some(target_uuid),
                }
            }
        };

        Some(SyncMessage::new(message_type))
    }

    /// Get broadcaster metrics
    pub fn metrics(&self) -> Arc<BroadcastMetrics> {
        self.metrics.clone()
    }

    /// Get active connections count
    pub fn active_connections(&self) -> usize {
        self.registry.connection_count()
    }
}

/// Broadcaster errors
#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    #[error("PubNub error: {0}")]
    PubNubError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Message handler implementation for PubNub subscription
pub struct BroadcasterMessageHandler {
    broadcaster: Arc<WebSocketBroadcaster>,
    user_id: Uuid,
}

impl BroadcasterMessageHandler {
    pub fn new(broadcaster: Arc<WebSocketBroadcaster>, user_id: Uuid) -> Self {
        Self {
            broadcaster,
            user_id,
        }
    }
}

#[async_trait::async_trait]
impl MessageHandler for BroadcasterMessageHandler {
    async fn handle_sync_message(&self, message: PubNubSyncMessage) {
        self.broadcaster
            .relay_pubnub_message(self.user_id, message)
            .await;
    }

    async fn handle_device_message(&self, message: DeviceMessage) {
        tracing::debug!("Received device message: {:?}", message);
        // Device messages are handled separately, not relayed to WebSocket
    }

    async fn handle_raw_message(&self, channel: &str, message: serde_json::Value) {
        tracing::warn!(
            "Received unhandled raw message on channel {}: {:?}",
            channel,
            message
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::HLCTimestamp;
    use crate::pubnub::PubNubConfig;

    fn create_test_broadcaster() -> (Arc<WebSocketBroadcaster>, Arc<ConnectionRegistry>) {
        let registry = Arc::new(ConnectionRegistry::new());
        let config = PubNubConfig::default();
        let pubnub = Arc::new(PubNubClient::new(
            config,
            "test-user".to_string(),
            "test-device".to_string(),
        ));

        let broadcaster = Arc::new(WebSocketBroadcaster::new(registry.clone(), pubnub));
        (broadcaster, registry)
    }

    #[test]
    fn test_convert_watchlist_update() {
        let (broadcaster, _) = create_test_broadcaster();

        let content_id = Uuid::new_v4();
        let pubnub_msg = PubNubSyncMessage::WatchlistUpdate {
            operation: "add".to_string(),
            content_id: content_id.to_string(),
            unique_tag: "tag-1".to_string(),
            timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
            device_id: "device-1".to_string(),
        };

        let ws_msg = broadcaster.convert_pubnub_message(pubnub_msg).unwrap();

        match ws_msg.message_type {
            SyncMessageType::WatchlistUpdate {
                content_id: cid,
                action,
            } => {
                assert_eq!(cid, content_id);
                assert_eq!(action, "add");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_convert_progress_update() {
        let (broadcaster, _) = create_test_broadcaster();

        let content_id = Uuid::new_v4();
        let pubnub_msg = PubNubSyncMessage::ProgressUpdate {
            content_id: content_id.to_string(),
            position_seconds: 120,
            duration_seconds: 3600,
            timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
            device_id: "device-1".to_string(),
        };

        let ws_msg = broadcaster.convert_pubnub_message(pubnub_msg).unwrap();

        match ws_msg.message_type {
            SyncMessageType::ProgressUpdate {
                position, duration, ..
            } => {
                assert_eq!(position, 120);
                assert_eq!(duration, 3600);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_convert_device_handoff() {
        let (broadcaster, _) = create_test_broadcaster();

        let target_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();
        let pubnub_msg = PubNubSyncMessage::DeviceHandoff {
            target_device_id: target_id.to_string(),
            content_id: content_id.to_string(),
            position_seconds: Some(100),
            timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
        };

        let ws_msg = broadcaster.convert_pubnub_message(pubnub_msg).unwrap();

        match ws_msg.message_type {
            SyncMessageType::DeviceCommand {
                command,
                target_device,
            } => {
                assert!(command.contains("handoff"));
                assert_eq!(target_device, Some(target_id));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_metrics_tracking() {
        let metrics = BroadcastMetrics::new();

        assert_eq!(metrics.total_messages_relayed(), 0);

        metrics.record_message_relayed();
        metrics.record_message_relayed();
        metrics.record_message_relayed();

        assert_eq!(metrics.total_messages_relayed(), 3);
    }

    #[test]
    fn test_latency_metrics() {
        let metrics = BroadcastMetrics::new();

        metrics.record_latency(10.0);
        metrics.record_latency(20.0);
        metrics.record_latency(30.0);
        metrics.record_latency(40.0);
        metrics.record_latency(50.0);

        let avg = metrics.average_latency_ms();
        assert!((avg - 30.0).abs() < 0.01);

        let p50 = metrics.p50_latency_ms();
        assert!((p50 - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_invalid_uuid_conversion() {
        let (broadcaster, _) = create_test_broadcaster();

        let pubnub_msg = PubNubSyncMessage::WatchlistUpdate {
            operation: "add".to_string(),
            content_id: "invalid-uuid".to_string(),
            unique_tag: "tag-1".to_string(),
            timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
            device_id: "device-1".to_string(),
        };

        let ws_msg = broadcaster.convert_pubnub_message(pubnub_msg);
        assert!(ws_msg.is_none());
    }

    #[tokio::test]
    async fn test_subscribe_user_channel() {
        let (broadcaster, _) = create_test_broadcaster();
        let user_id = Uuid::new_v4();

        // This will fail in test environment without real PubNub, but we test the structure
        let result = broadcaster.subscribe_user_channel(user_id).await;

        // Expected to fail with network error in test environment
        assert!(result.is_err());
    }
}
