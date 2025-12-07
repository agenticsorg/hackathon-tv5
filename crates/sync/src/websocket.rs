/// WebSocket handler for real-time synchronization
///
/// Manages WebSocket connections with clients for bidirectional sync
use crate::command_router::{Command, CommandRouter};
use crate::device::CommandType;
use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// WebSocket connection heartbeat interval (30 seconds)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Client timeout (60 seconds - 2 missed heartbeats)
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

/// WebSocket session actor
pub struct SyncWebSocket {
    /// User identifier
    user_id: String,

    /// Device identifier
    device_id: String,

    /// Last heartbeat timestamp
    hb: Instant,

    /// Command router for routing device commands
    command_router: Option<Arc<CommandRouter>>,
}

impl SyncWebSocket {
    /// Create new WebSocket session
    pub fn new(user_id: String, device_id: String) -> Self {
        Self {
            user_id,
            device_id,
            hb: Instant::now(),
            command_router: None,
        }
    }

    /// Create new WebSocket session with command router
    pub fn with_command_router(
        user_id: String,
        device_id: String,
        command_router: Arc<CommandRouter>,
    ) -> Self {
        Self {
            user_id,
            device_id,
            hb: Instant::now(),
            command_router: Some(command_router),
        }
    }

    /// Start heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(
            HEARTBEAT_INTERVAL,
            |act, ctx: &mut ws::WebsocketContext<Self>| {
                // Check if client has sent heartbeat recently
                if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                    tracing::warn!(
                        "WebSocket client {} heartbeat timeout, disconnecting",
                        act.device_id
                    );
                    ctx.stop();
                    return;
                }

                // Send ping to client
                ctx.ping(b"");
            },
        );
    }

    /// Handle incoming sync message
    fn handle_sync_message(&mut self, msg: WebSocketMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            WebSocketMessage::WatchlistUpdate { .. } => {
                tracing::debug!("Received watchlist update from {}", self.device_id);
                // In production: broadcast to other user devices via PubNub
            }
            WebSocketMessage::ProgressUpdate { .. } => {
                tracing::debug!("Received progress update from {}", self.device_id);
                // In production: broadcast to other user devices via PubNub
            }
            WebSocketMessage::DeviceHeartbeat => {
                tracing::trace!("Received heartbeat from {}", self.device_id);
                self.hb = Instant::now();
            }
            WebSocketMessage::DeviceCommand {
                target_device_id,
                command_type,
                payload,
            } => {
                tracing::debug!(
                    "Received device command from {} to {}: {:?}",
                    self.device_id,
                    target_device_id,
                    command_type
                );

                // Route command through CommandRouter if available
                if let Some(router) = &self.command_router {
                    let command = Command::new(
                        command_type.clone(),
                        self.device_id.clone(),
                        target_device_id.clone(),
                    )
                    .with_payload(payload.unwrap_or_else(|| serde_json::json!({})));

                    // Spawn async task to route command
                    let router = router.clone();
                    let device_id = self.device_id.clone();
                    actix::spawn(async move {
                        match router.route_command(command).await {
                            Ok(command_id) => {
                                tracing::info!(
                                    "Successfully routed command {} from {}",
                                    command_id,
                                    device_id
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to route command from {}: {}",
                                    device_id,
                                    e
                                );
                            }
                        }
                    });
                } else {
                    tracing::warn!(
                        "Command router not configured for device {}",
                        self.device_id
                    );
                }
            }
            WebSocketMessage::Ping => {
                ctx.pong(b"");
            }
            WebSocketMessage::Pong => {
                self.hb = Instant::now();
            }
        }
    }
}

impl Actor for SyncWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!(
            "WebSocket connection established for user {} device {}",
            self.user_id,
            self.device_id
        );
        self.start_heartbeat(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(
            "WebSocket connection closed for user {} device {}",
            self.user_id,
            self.device_id
        );
    }
}

/// Handler for BroadcastMessage from registry
impl Handler<crate::ws::BroadcastMessage> for SyncWebSocket {
    type Result = ();

    fn handle(
        &mut self,
        msg: crate::ws::BroadcastMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        // Send JSON message to WebSocket client
        use actix_web_actors::ws::WebsocketContext;
        WebsocketContext::text(ctx, msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SyncWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        use actix_web_actors::ws::WebsocketContext;
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                WebsocketContext::pong(ctx, &msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Parse JSON message
                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(msg) => self.handle_sync_message(msg, ctx),
                    Err(e) => {
                        tracing::error!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                tracing::warn!("Binary WebSocket messages not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                tracing::info!("WebSocket close received: {:?}", reason);
                WebsocketContext::close(ctx, reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                tracing::warn!("WebSocket continuation frames not supported");
            }
            Ok(ws::Message::Nop) => {}
            Err(e) => {
                tracing::error!("WebSocket protocol error: {}", e);
                ctx.stop();
            }
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "watchlist_update")]
    WatchlistUpdate {
        operation: String,
        content_id: String,
        timestamp: String,
    },

    #[serde(rename = "progress_update")]
    ProgressUpdate {
        content_id: String,
        position_seconds: u32,
        timestamp: String,
    },

    #[serde(rename = "device_heartbeat")]
    DeviceHeartbeat,

    #[serde(rename = "device_command")]
    DeviceCommand {
        target_device_id: String,
        command_type: CommandType,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },

    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "pong")]
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::ProgressUpdate {
            content_id: "content-1".to_string(),
            position_seconds: 100,
            timestamp: "1000".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("progress_update"));

        let deserialized: WebSocketMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            WebSocketMessage::ProgressUpdate {
                position_seconds, ..
            } => {
                assert_eq!(position_seconds, 100);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
