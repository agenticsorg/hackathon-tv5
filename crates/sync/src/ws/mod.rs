/// WebSocket module for real-time synchronization
///
/// Provides WebSocket connection management, broadcasting, and PubNub integration
pub mod broadcaster;
pub mod registry;

pub use broadcaster::{
    BroadcastError, BroadcastMetrics, BroadcasterMessageHandler, WebSocketBroadcaster,
};
pub use registry::{
    BroadcastMessage, ConnectionId, ConnectionRegistry, SyncMessage, SyncMessageType,
};

// Re-export main WebSocket actor from parent
pub use crate::websocket as handler;
