/// PubNub integration for real-time cross-device synchronization
///
/// Channel structure:
/// - user.{userId}.sync - Watchlist, preferences, progress
/// - user.{userId}.devices - Device presence, heartbeat
/// - user.{userId}.notifications - Alerts, recommendations
use crate::crdt::HLCTimestamp;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::time::{sleep, Duration};

/// PubNub client configuration
#[derive(Debug, Clone)]
pub struct PubNubConfig {
    /// PubNub publish key
    pub publish_key: String,

    /// PubNub subscribe key
    pub subscribe_key: String,

    /// API origin
    pub origin: String,
}

impl Default for PubNubConfig {
    fn default() -> Self {
        Self {
            publish_key: std::env::var("PUBNUB_PUBLISH_KEY").unwrap_or_else(|_| "demo".to_string()),
            subscribe_key: std::env::var("PUBNUB_SUBSCRIBE_KEY")
                .unwrap_or_else(|_| "demo".to_string()),
            origin: "ps.pndsn.com".to_string(),
        }
    }
}

/// PubNub client for Media Gateway
pub struct PubNubClient {
    config: PubNubConfig,
    http_client: Client,
    user_id: String,
    device_id: String,
}

impl PubNubClient {
    /// Create new PubNub client
    pub fn new(config: PubNubConfig, user_id: String, device_id: String) -> Self {
        Self {
            config,
            http_client: Client::new(),
            user_id,
            device_id,
        }
    }

    /// Get sync channel name for user
    pub fn sync_channel(&self) -> String {
        format!("user.{}.sync", self.user_id)
    }

    /// Get devices channel name for user
    pub fn devices_channel(&self) -> String {
        format!("user.{}.devices", self.user_id)
    }

    /// Get notifications channel name for user
    pub fn notifications_channel(&self) -> String {
        format!("user.{}.notifications", self.user_id)
    }

    /// Publish message to channel
    pub async fn publish<T: Serialize>(
        &self,
        channel: &str,
        message: &T,
    ) -> Result<PublishResponse, PubNubError> {
        let url = format!(
            "https://{}/publish/{}/{}/0/{}/0",
            self.config.origin, self.config.publish_key, self.config.subscribe_key, channel
        );

        let message_json = serde_json::to_string(message)
            .map_err(|e| PubNubError::SerializationError(e.to_string()))?;

        let response = self
            .http_client
            .post(&url)
            .json(&message_json)
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        let publish_response: PublishResponse = response
            .json()
            .await
            .map_err(|e| PubNubError::DeserializationError(e.to_string()))?;

        Ok(publish_response)
    }

    /// Subscribe to channels with message callback
    pub async fn subscribe_with_handler(
        self: Arc<Self>,
        channels: Vec<String>,
        handler: Arc<dyn MessageHandler>,
    ) -> Result<SubscriptionManager, PubNubError> {
        Ok(SubscriptionManager::new(self, handler, channels))
    }

    /// Subscribe to channels (establishes long-poll connection)
    pub async fn subscribe(&self, channels: Vec<String>) -> Result<(), PubNubError> {
        tracing::info!("Subscribing to channels: {:?}", channels);

        // Initial subscribe request to get timetoken
        let channels_str = channels.join(",");
        let url = format!(
            "https://{}/v2/subscribe/{}/{}/0/0",
            self.config.origin, self.config.subscribe_key, channels_str
        );

        self.http_client
            .get(&url)
            .query(&[("uuid", &self.device_id)])
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Unsubscribe from channels
    pub async fn unsubscribe(&self, channels: Vec<String>) -> Result<(), PubNubError> {
        let channels_str = channels.join(",");
        let url = format!(
            "https://{}/v2/presence/sub-key/{}/channel/{}/leave",
            self.config.origin, self.config.subscribe_key, channels_str
        );

        self.http_client
            .get(&url)
            .query(&[("uuid", &self.device_id)])
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        tracing::info!("Unsubscribed from channels: {:?}", channels);
        Ok(())
    }

    /// Publish presence heartbeat
    pub async fn heartbeat(&self) -> Result<(), PubNubError> {
        let url = format!(
            "https://{}/v2/presence/sub-key/{}/channel/{}/heartbeat",
            self.config.origin,
            self.config.subscribe_key,
            self.devices_channel()
        );

        self.http_client
            .get(&url)
            .query(&[("heartbeat", "300"), ("uuid", &self.device_id)])
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Get presence information for channel
    pub async fn here_now(&self, channel: &str) -> Result<HereNowResponse, PubNubError> {
        let url = format!(
            "https://{}/v2/presence/sub-key/{}/channel/{}",
            self.config.origin, self.config.subscribe_key, channel
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        let here_now: HereNowResponse = response
            .json()
            .await
            .map_err(|e| PubNubError::DeserializationError(e.to_string()))?;

        Ok(here_now)
    }

    /// Fetch message history from channel
    pub async fn history(
        &self,
        channel: &str,
        count: usize,
    ) -> Result<HistoryResponse, PubNubError> {
        let url = format!(
            "https://{}/v2/history/sub-key/{}/channel/{}",
            self.config.origin, self.config.subscribe_key, channel
        );

        let response = self
            .http_client
            .get(&url)
            .query(&[("count", count.to_string())])
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        let history: HistoryResponse = response
            .json()
            .await
            .map_err(|e| PubNubError::DeserializationError(e.to_string()))?;

        Ok(history)
    }
}

/// PubNub publish response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub status: i32,
    pub timetoken: String,
}

/// PubNub presence response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereNowResponse {
    pub status: i32,
    pub message: String,
    pub occupancy: usize,
    pub uuids: Vec<String>,
}

/// PubNub history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub status: i32,
    pub messages: Vec<serde_json::Value>,
}

/// PubNub errors
#[derive(Debug, Error)]
pub enum PubNubError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),
}

/// Message types for PubNub channels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    #[serde(rename = "watchlist_update")]
    WatchlistUpdate {
        operation: String,
        content_id: String,
        unique_tag: String,
        timestamp: HLCTimestamp,
        device_id: String,
    },

    #[serde(rename = "progress_update")]
    ProgressUpdate {
        content_id: String,
        position_seconds: u32,
        duration_seconds: u32,
        timestamp: HLCTimestamp,
        device_id: String,
    },

    #[serde(rename = "device_handoff")]
    DeviceHandoff {
        target_device_id: String,
        content_id: String,
        position_seconds: Option<u32>,
        timestamp: HLCTimestamp,
    },
}

/// Device message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeviceMessage {
    #[serde(rename = "device_heartbeat")]
    Heartbeat {
        device_id: String,
        capabilities: DeviceCapabilities,
        timestamp: HLCTimestamp,
    },

    #[serde(rename = "device_command")]
    Command {
        target_device_id: String,
        command: RemoteCommand,
        timestamp: HLCTimestamp,
    },
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub max_resolution: String,
    pub hdr_support: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub can_cast: bool,
}

/// Remote control commands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command_type")]
pub enum RemoteCommand {
    #[serde(rename = "play")]
    Play,

    #[serde(rename = "pause")]
    Pause,

    #[serde(rename = "seek")]
    Seek { position_seconds: u32 },

    #[serde(rename = "cast")]
    Cast { content_id: String },
}

/// Message handler callback trait
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_sync_message(&self, message: SyncMessage);
    async fn handle_device_message(&self, message: DeviceMessage);
    async fn handle_raw_message(&self, channel: &str, message: serde_json::Value);
}

/// Subscription manager for handling real-time messages
pub struct SubscriptionManager {
    client: Arc<PubNubClient>,
    handler: Arc<dyn MessageHandler>,
    channels: Vec<String>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl SubscriptionManager {
    pub fn new(
        client: Arc<PubNubClient>,
        handler: Arc<dyn MessageHandler>,
        channels: Vec<String>,
    ) -> Self {
        Self {
            client,
            handler,
            channels,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start subscription loop
    pub async fn start(&self) -> Result<(), PubNubError> {
        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let mut timetoken = "0".to_string();

        tracing::info!(
            "Starting PubNub subscription for channels: {:?}",
            self.channels
        );

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            match self.poll_messages(&timetoken).await {
                Ok((messages, new_timetoken)) => {
                    timetoken = new_timetoken;

                    for (channel, message) in messages {
                        self.dispatch_message(&channel, message).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Subscription error: {}. Reconnecting in 5s...", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }

        Ok(())
    }

    /// Stop subscription loop
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Poll for new messages (long-poll)
    async fn poll_messages(
        &self,
        timetoken: &str,
    ) -> Result<(Vec<(String, serde_json::Value)>, String), PubNubError> {
        let channels = self.channels.join(",");
        let url = format!(
            "https://{}/v2/subscribe/{}/{}/0/{}",
            self.client.config.origin, self.client.config.subscribe_key, channels, timetoken
        );

        let response = self
            .client
            .http_client
            .get(&url)
            .query(&[("uuid", &self.client.device_id)])
            .timeout(Duration::from_secs(310)) // PubNub long-poll timeout
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        let body: SubscribeResponse = response
            .json()
            .await
            .map_err(|e| PubNubError::DeserializationError(e.to_string()))?;

        let mut messages = Vec::new();
        for (i, msg) in body.messages.into_iter().enumerate() {
            if let Some(channel) = body.channels.get(i) {
                messages.push((channel.clone(), msg));
            }
        }

        Ok((messages, body.timetoken.t))
    }

    /// Dispatch message to appropriate handler
    async fn dispatch_message(&self, channel: &str, message: serde_json::Value) {
        // Try to parse as SyncMessage
        if channel.contains(".sync") {
            if let Ok(sync_msg) = serde_json::from_value::<SyncMessage>(message.clone()) {
                self.handler.handle_sync_message(sync_msg).await;
                return;
            }
        }

        // Try to parse as DeviceMessage
        if channel.contains(".devices") {
            if let Ok(device_msg) = serde_json::from_value::<DeviceMessage>(message.clone()) {
                self.handler.handle_device_message(device_msg).await;
                return;
            }
        }

        // Fall back to raw handler
        self.handler.handle_raw_message(channel, message).await;
    }
}

/// PubNub subscribe response structure
#[derive(Debug, Deserialize)]
struct SubscribeResponse {
    #[serde(rename = "t")]
    timetoken: Timetoken,
    #[serde(rename = "m", default)]
    messages: Vec<serde_json::Value>,
    #[serde(default)]
    channels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Timetoken {
    t: String,
    r: Option<u32>,
}
