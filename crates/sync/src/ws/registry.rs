/// WebSocket connection registry for managing per-user connection pools
///
/// Tracks active WebSocket connections and provides efficient broadcast mechanisms
use actix::{Addr, Message as ActixMessage};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::websocket::SyncWebSocket;

/// Unique identifier for a WebSocket connection
pub type ConnectionId = Uuid;

/// Message to send to WebSocket client
#[derive(Debug, Clone, Serialize, Deserialize, ActixMessage)]
#[rtype(result = "()")]
pub struct SyncMessage {
    #[serde(flatten)]
    pub message_type: SyncMessageType,
}

impl SyncMessage {
    pub fn new(message_type: SyncMessageType) -> Self {
        Self { message_type }
    }

    pub fn watchlist_update(content_id: Uuid, action: String) -> Self {
        Self::new(SyncMessageType::WatchlistUpdate { content_id, action })
    }

    pub fn progress_update(content_id: Uuid, position: u32, duration: u32) -> Self {
        Self::new(SyncMessageType::ProgressUpdate {
            content_id,
            position,
            duration,
        })
    }

    pub fn device_command(command: String, target_device: Option<Uuid>) -> Self {
        Self::new(SyncMessageType::DeviceCommand {
            command,
            target_device,
        })
    }

    /// Serialize to JSON text for WebSocket transmission
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Sync message types for WebSocket broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessageType {
    #[serde(rename = "watchlist_update")]
    WatchlistUpdate { content_id: Uuid, action: String },

    #[serde(rename = "progress_update")]
    ProgressUpdate {
        content_id: Uuid,
        position: u32,
        duration: u32,
    },

    #[serde(rename = "device_command")]
    DeviceCommand {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_device: Option<Uuid>,
    },
}

/// WebSocket connection information
#[derive(Clone)]
struct ConnectionInfo {
    pub conn_id: ConnectionId,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub addr: Addr<SyncWebSocket>,
}

/// Registry for tracking active WebSocket connections
pub struct ConnectionRegistry {
    /// Map: user_id -> Vec<ConnectionInfo>
    user_connections: Arc<DashMap<Uuid, Vec<ConnectionInfo>>>,

    /// Map: connection_id -> ConnectionInfo
    connections: Arc<DashMap<ConnectionId, ConnectionInfo>>,

    /// Metrics
    metrics: ConnectionMetrics,
}

#[derive(Default, Clone)]
struct ConnectionMetrics {
    total_connections: Arc<parking_lot::RwLock<usize>>,
    messages_sent: Arc<parking_lot::RwLock<usize>>,
}

impl ConnectionRegistry {
    /// Create new connection registry
    pub fn new() -> Self {
        Self {
            user_connections: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
            metrics: ConnectionMetrics::default(),
        }
    }

    /// Register a new WebSocket connection
    pub fn register(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        addr: Addr<SyncWebSocket>,
    ) -> ConnectionId {
        let conn_id = Uuid::new_v4();

        let info = ConnectionInfo {
            conn_id,
            user_id,
            device_id,
            addr,
        };

        // Add to global connections map
        self.connections.insert(conn_id, info.clone());

        // Add to user connections
        self.user_connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(info);

        // Update metrics
        *self.metrics.total_connections.write() += 1;

        tracing::info!(
            "Registered WebSocket connection {} for user {} device {}",
            conn_id,
            user_id,
            device_id
        );

        conn_id
    }

    /// Unregister a WebSocket connection
    pub fn unregister(&self, conn_id: ConnectionId) {
        if let Some((_, info)) = self.connections.remove(&conn_id) {
            // Remove from user connections
            if let Some(mut conns) = self.user_connections.get_mut(&info.user_id) {
                conns.retain(|c| c.conn_id != conn_id);

                // Remove user entry if no more connections
                if conns.is_empty() {
                    drop(conns);
                    self.user_connections.remove(&info.user_id);
                }
            }

            // Update metrics
            let mut total = self.metrics.total_connections.write();
            if *total > 0 {
                *total -= 1;
            }

            tracing::info!(
                "Unregistered WebSocket connection {} for user {} device {}",
                conn_id,
                info.user_id,
                info.device_id
            );
        }
    }

    /// Get all connection IDs for a user
    pub fn get_user_connections(&self, user_id: Uuid) -> Vec<ConnectionId> {
        self.user_connections
            .get(&user_id)
            .map(|conns| conns.iter().map(|c| c.conn_id).collect())
            .unwrap_or_default()
    }

    /// Send message to all connections for a specific user
    pub async fn send_to_user(
        &self,
        user_id: Uuid,
        message: &SyncMessage,
    ) -> Result<usize, BroadcastError> {
        let conns = match self.user_connections.get(&user_id) {
            Some(conns) => conns.clone(),
            None => return Ok(0),
        };

        let json = message
            .to_json()
            .map_err(|e| BroadcastError::SerializationError(e.to_string()))?;

        let mut sent_count = 0;

        for conn in conns.iter() {
            conn.addr.do_send(BroadcastMessage(json.clone()));
            sent_count += 1;
        }

        // Update metrics
        *self.metrics.messages_sent.write() += sent_count;

        tracing::debug!(
            "Broadcast message to {} connections for user {}",
            sent_count,
            user_id
        );

        Ok(sent_count)
    }

    /// Broadcast message to all active connections
    pub async fn broadcast_to_all(&self, message: &SyncMessage) -> Result<usize, BroadcastError> {
        let json = message
            .to_json()
            .map_err(|e| BroadcastError::SerializationError(e.to_string()))?;

        let mut sent_count = 0;

        for conn in self.connections.iter() {
            conn.value().addr.do_send(BroadcastMessage(json.clone()));
            sent_count += 1;
        }

        // Update metrics
        *self.metrics.messages_sent.write() += sent_count;

        tracing::debug!("Broadcast message to {} total connections", sent_count);

        Ok(sent_count)
    }

    /// Get total number of active connections
    pub fn connection_count(&self) -> usize {
        *self.metrics.total_connections.read()
    }

    /// Get total messages sent
    pub fn messages_sent(&self) -> usize {
        *self.metrics.messages_sent.read()
    }

    /// Get number of users with active connections
    pub fn active_users_count(&self) -> usize {
        self.user_connections.len()
    }
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Message for broadcasting to WebSocket
#[derive(ActixMessage, Clone)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub String);

/// Broadcast errors
#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(Uuid),

    #[error("Send error: {0}")]
    SendError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic_metrics() {
        let registry = ConnectionRegistry::new();

        assert_eq!(registry.connection_count(), 0);
        assert_eq!(registry.active_users_count(), 0);
        assert_eq!(registry.messages_sent(), 0);

        let user_id = Uuid::new_v4();
        let conns = registry.get_user_connections(user_id);
        assert_eq!(conns.len(), 0);
    }

    #[test]
    fn test_sync_message_serialization() {
        let msg = SyncMessage::watchlist_update(Uuid::new_v4(), "add".to_string());

        let json = msg.to_json().unwrap();
        assert!(json.contains("\"type\":\"watchlist_update\""));
        assert!(json.contains("\"action\":\"add\""));

        let deserialized: SyncMessage = serde_json::from_str(&json).unwrap();
        match deserialized.message_type {
            SyncMessageType::WatchlistUpdate { action, .. } => {
                assert_eq!(action, "add");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_progress_update_message() {
        let content_id = Uuid::new_v4();
        let msg = SyncMessage::progress_update(content_id, 120, 3600);

        let json = msg.to_json().unwrap();
        assert!(json.contains("\"type\":\"progress_update\""));
        assert!(json.contains("\"position\":120"));
        assert!(json.contains("\"duration\":3600"));
    }

    #[test]
    fn test_device_command_message() {
        let msg = SyncMessage::device_command("pause".to_string(), Some(Uuid::new_v4()));

        let json = msg.to_json().unwrap();
        assert!(json.contains("\"type\":\"device_command\""));
        assert!(json.contains("\"command\":\"pause\""));
        assert!(json.contains("\"target_device\""));
    }
}
