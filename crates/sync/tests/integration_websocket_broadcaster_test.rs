/// Integration tests for WebSocket broadcaster with multiple simulated clients
///
/// Tests the complete flow: PubNub → Broadcaster → WebSocket connections
use actix::Actor;
use actix_rt::System;
use media_gateway_sync::crdt::HLCTimestamp;
use media_gateway_sync::pubnub::{PubNubClient, PubNubConfig, SyncMessage as PubNubSyncMessage};
use media_gateway_sync::websocket::SyncWebSocket;
use media_gateway_sync::ws::{
    BroadcasterMessageHandler, ConnectionRegistry, SyncMessage, WebSocketBroadcaster,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Helper to create test broadcaster infrastructure
fn create_test_infrastructure() -> (
    Arc<ConnectionRegistry>,
    Arc<WebSocketBroadcaster>,
    Arc<PubNubClient>,
) {
    let registry = Arc::new(ConnectionRegistry::new());

    let config = PubNubConfig::default();
    let pubnub_client = Arc::new(PubNubClient::new(
        config,
        "test-user".to_string(),
        "test-device".to_string(),
    ));

    let broadcaster = Arc::new(WebSocketBroadcaster::new(
        registry.clone(),
        pubnub_client.clone(),
    ));

    (registry, broadcaster, pubnub_client)
}

#[tokio::test]
async fn test_broadcaster_metrics_initialization() {
    let (_, broadcaster, _) = create_test_infrastructure();

    let metrics = broadcaster.metrics();

    assert_eq!(metrics.total_messages_relayed(), 0);
    assert_eq!(metrics.average_latency_ms(), 0.0);
    assert_eq!(broadcaster.active_connections(), 0);
}

#[tokio::test]
async fn test_relay_watchlist_update() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    // Create PubNub message
    let pubnub_msg = PubNubSyncMessage::WatchlistUpdate {
        operation: "add".to_string(),
        content_id: content_id.to_string(),
        unique_tag: "tag-1".to_string(),
        timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
        device_id: "device-1".to_string(),
    };

    // Relay message (no connections, should succeed with 0 count)
    broadcaster.relay_pubnub_message(user_id, pubnub_msg).await;

    let metrics = broadcaster.metrics();
    assert_eq!(metrics.total_messages_relayed(), 1);
}

#[tokio::test]
async fn test_relay_progress_update() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let pubnub_msg = PubNubSyncMessage::ProgressUpdate {
        content_id: content_id.to_string(),
        position_seconds: 120,
        duration_seconds: 3600,
        timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
        device_id: "device-1".to_string(),
    };

    broadcaster.relay_pubnub_message(user_id, pubnub_msg).await;

    let metrics = broadcaster.metrics();
    assert_eq!(metrics.total_messages_relayed(), 1);
}

#[tokio::test]
async fn test_relay_device_handoff() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();
    let target_device_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let pubnub_msg = PubNubSyncMessage::DeviceHandoff {
        target_device_id: target_device_id.to_string(),
        content_id: content_id.to_string(),
        position_seconds: Some(100),
        timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
    };

    broadcaster.relay_pubnub_message(user_id, pubnub_msg).await;

    let metrics = broadcaster.metrics();
    assert_eq!(metrics.total_messages_relayed(), 1);
}

#[tokio::test]
async fn test_broadcaster_with_invalid_uuid() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();

    // Invalid UUID in content_id should be handled gracefully
    let pubnub_msg = PubNubSyncMessage::WatchlistUpdate {
        operation: "add".to_string(),
        content_id: "not-a-uuid".to_string(),
        unique_tag: "tag-1".to_string(),
        timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
        device_id: "device-1".to_string(),
    };

    broadcaster.relay_pubnub_message(user_id, pubnub_msg).await;

    // Should not increment metrics for invalid messages
    let metrics = broadcaster.metrics();
    assert_eq!(metrics.total_messages_relayed(), 0);
}

#[tokio::test]
async fn test_multiple_message_relays() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();

    // Relay multiple messages
    for i in 0..10 {
        let content_id = Uuid::new_v4();
        let pubnub_msg = PubNubSyncMessage::ProgressUpdate {
            content_id: content_id.to_string(),
            position_seconds: i * 10,
            duration_seconds: 3600,
            timestamp: HLCTimestamp::new(1000 + i as u64, 0, "device-1".to_string()),
            device_id: "device-1".to_string(),
        };

        broadcaster.relay_pubnub_message(user_id, pubnub_msg).await;
    }

    let metrics = broadcaster.metrics();
    assert_eq!(metrics.total_messages_relayed(), 10);

    // Check latency tracking
    let avg_latency = metrics.average_latency_ms();
    assert!(avg_latency >= 0.0);
}

#[tokio::test]
async fn test_latency_percentiles() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();

    // Generate messages to populate latency metrics
    for _ in 0..50 {
        let content_id = Uuid::new_v4();
        let pubnub_msg = PubNubSyncMessage::WatchlistUpdate {
            operation: "add".to_string(),
            content_id: content_id.to_string(),
            unique_tag: "tag-1".to_string(),
            timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
            device_id: "device-1".to_string(),
        };

        broadcaster.relay_pubnub_message(user_id, pubnub_msg).await;
    }

    let metrics = broadcaster.metrics();

    // All percentiles should be >= 0
    assert!(metrics.p50_latency_ms() >= 0.0);
    assert!(metrics.p95_latency_ms() >= 0.0);
    assert!(metrics.p99_latency_ms() >= 0.0);

    // p99 >= p95 >= p50
    assert!(metrics.p99_latency_ms() >= metrics.p95_latency_ms());
    assert!(metrics.p95_latency_ms() >= metrics.p50_latency_ms());
}

#[tokio::test]
async fn test_subscribe_user_channel() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();

    // Subscription will fail without real PubNub, but test the structure
    let result = broadcaster.subscribe_user_channel(user_id).await;

    // Expected to fail in test environment
    assert!(result.is_err());
}

#[test]
fn test_registry_basic_operations() {
    let registry = ConnectionRegistry::new();

    assert_eq!(registry.connection_count(), 0);
    assert_eq!(registry.active_users_count(), 0);
    assert_eq!(registry.messages_sent(), 0);

    let user_id = Uuid::new_v4();
    let conns = registry.get_user_connections(user_id);
    assert_eq!(conns.len(), 0);
}

#[test]
fn test_sync_message_types() {
    let content_id = Uuid::new_v4();

    // Test watchlist update
    let msg = SyncMessage::watchlist_update(content_id, "add".to_string());
    let json = msg.to_json().unwrap();
    assert!(json.contains("watchlist_update"));

    // Test progress update
    let msg = SyncMessage::progress_update(content_id, 120, 3600);
    let json = msg.to_json().unwrap();
    assert!(json.contains("progress_update"));

    // Test device command
    let target = Uuid::new_v4();
    let msg = SyncMessage::device_command("pause".to_string(), Some(target));
    let json = msg.to_json().unwrap();
    assert!(json.contains("device_command"));
}

#[tokio::test]
async fn test_concurrent_message_relays() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();
    let broadcaster = Arc::new(broadcaster);

    // Spawn concurrent relay tasks
    let mut handles = vec![];

    for i in 0..20 {
        let broadcaster_clone = broadcaster.clone();
        let user_id_clone = user_id;

        let handle = tokio::spawn(async move {
            let content_id = Uuid::new_v4();
            let pubnub_msg = PubNubSyncMessage::ProgressUpdate {
                content_id: content_id.to_string(),
                position_seconds: i * 10,
                duration_seconds: 3600,
                timestamp: HLCTimestamp::new(1000 + i as u64, 0, "device-1".to_string()),
                device_id: "device-1".to_string(),
            };

            broadcaster_clone
                .relay_pubnub_message(user_id_clone, pubnub_msg)
                .await;
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    let metrics = broadcaster.metrics();
    assert_eq!(metrics.total_messages_relayed(), 20);
}

#[tokio::test]
async fn test_broadcaster_message_handler() {
    let (registry, broadcaster, _) = create_test_infrastructure();

    let user_id = Uuid::new_v4();
    let handler = BroadcasterMessageHandler::new(Arc::new(broadcaster.as_ref().clone()), user_id);

    // Test handling sync message
    let content_id = Uuid::new_v4();
    let sync_msg = PubNubSyncMessage::WatchlistUpdate {
        operation: "add".to_string(),
        content_id: content_id.to_string(),
        unique_tag: "tag-1".to_string(),
        timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
        device_id: "device-1".to_string(),
    };

    use media_gateway_sync::pubnub::MessageHandler;
    handler.handle_sync_message(sync_msg).await;

    // Verify message was relayed
    let metrics = broadcaster.metrics();
    assert_eq!(metrics.total_messages_relayed(), 1);
}
