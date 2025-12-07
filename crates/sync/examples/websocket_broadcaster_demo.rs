use media_gateway_sync::crdt::HLCTimestamp;
/// WebSocket Broadcaster Demo
///
/// Demonstrates the complete WebSocket broadcasting flow:
/// 1. Initialize ConnectionRegistry
/// 2. Create WebSocketBroadcaster with PubNub client
/// 3. Subscribe to user channel
/// 4. Relay messages to connected WebSocket clients
///
/// Run with: cargo run --example websocket_broadcaster_demo
use media_gateway_sync::pubnub::{PubNubClient, PubNubConfig, SyncMessage as PubNubSyncMessage};
use media_gateway_sync::ws::{ConnectionRegistry, WebSocketBroadcaster};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,websocket_broadcaster_demo=debug")
        .init();

    println!("🚀 WebSocket Broadcaster Demo");
    println!("================================\n");

    // Step 1: Create ConnectionRegistry
    println!("1. Creating ConnectionRegistry...");
    let registry = Arc::new(ConnectionRegistry::new());
    println!("   ✓ Registry created");
    println!("   - Active connections: {}", registry.connection_count());
    println!("   - Active users: {}\n", registry.active_users_count());

    // Step 2: Initialize PubNub client
    println!("2. Initializing PubNub client...");
    let config = PubNubConfig {
        publish_key: std::env::var("PUBNUB_PUBLISH_KEY").unwrap_or_else(|_| "demo".to_string()),
        subscribe_key: std::env::var("PUBNUB_SUBSCRIBE_KEY").unwrap_or_else(|_| "demo".to_string()),
        origin: "ps.pndsn.com".to_string(),
    };

    let user_id = "demo-user".to_string();
    let device_id = "demo-device".to_string();

    let pubnub_client = Arc::new(PubNubClient::new(
        config,
        user_id.clone(),
        device_id.clone(),
    ));
    println!("   ✓ PubNub client initialized");
    println!("   - User ID: {}", user_id);
    println!("   - Device ID: {}\n", device_id);

    // Step 3: Create WebSocketBroadcaster
    println!("3. Creating WebSocketBroadcaster...");
    let broadcaster = Arc::new(WebSocketBroadcaster::new(
        registry.clone(),
        pubnub_client.clone(),
    ));
    println!("   ✓ Broadcaster created\n");

    // Step 4: Subscribe to user channel
    println!("4. Subscribing to user channel...");
    let user_uuid = Uuid::new_v4();
    let channel_name = format!("user.{}.sync", user_uuid);

    // Note: This will fail without valid PubNub credentials, but demonstrates the API
    match broadcaster.subscribe_user_channel(user_uuid).await {
        Ok(_) => {
            println!("   ✓ Successfully subscribed to: {}", channel_name);
        }
        Err(e) => {
            println!("   ⚠ Subscription failed (expected in demo): {}", e);
            println!("   💡 Set PUBNUB_PUBLISH_KEY and PUBNUB_SUBSCRIBE_KEY for real connection\n");
        }
    }

    // Step 5: Demonstrate message relay (simulated)
    println!("5. Simulating message relay...\n");

    // Example 1: Watchlist Update
    println!("   Example 1: WATCHLIST_UPDATE");
    let content_id = Uuid::new_v4();
    let watchlist_msg = PubNubSyncMessage::WatchlistUpdate {
        operation: "add".to_string(),
        content_id: content_id.to_string(),
        unique_tag: "movie-123".to_string(),
        timestamp: HLCTimestamp::new(1000, 0, device_id.clone()),
        device_id: device_id.clone(),
    };

    broadcaster
        .relay_pubnub_message(user_uuid, watchlist_msg)
        .await;
    println!("   ✓ Watchlist update relayed");
    println!("     - Content ID: {}", content_id);
    println!("     - Action: add\n");

    // Example 2: Progress Update
    println!("   Example 2: PROGRESS_UPDATE");
    let progress_msg = PubNubSyncMessage::ProgressUpdate {
        content_id: content_id.to_string(),
        position_seconds: 1234,
        duration_seconds: 7200,
        timestamp: HLCTimestamp::new(2000, 0, device_id.clone()),
        device_id: device_id.clone(),
    };

    broadcaster
        .relay_pubnub_message(user_uuid, progress_msg)
        .await;
    println!("   ✓ Progress update relayed");
    println!("     - Position: 1234s / 7200s (17.1%)");
    println!("     - Content ID: {}\n", content_id);

    // Example 3: Device Handoff
    println!("   Example 3: DEVICE_COMMAND (Handoff)");
    let target_device = Uuid::new_v4();
    let handoff_msg = PubNubSyncMessage::DeviceHandoff {
        target_device_id: target_device.to_string(),
        content_id: content_id.to_string(),
        position_seconds: Some(1234),
        timestamp: HLCTimestamp::new(3000, 0, device_id.clone()),
    };

    broadcaster
        .relay_pubnub_message(user_uuid, handoff_msg)
        .await;
    println!("   ✓ Device handoff relayed");
    println!("     - Target device: {}", target_device);
    println!("     - Resume position: 1234s\n");

    // Step 6: Display metrics
    println!("6. Broadcaster Metrics:");
    let metrics = broadcaster.metrics();
    println!(
        "   - Messages relayed: {}",
        metrics.total_messages_relayed()
    );
    println!(
        "   - Average latency: {:.2}ms",
        metrics.average_latency_ms()
    );
    println!("   - P50 latency: {:.2}ms", metrics.p50_latency_ms());
    println!("   - P95 latency: {:.2}ms", metrics.p95_latency_ms());
    println!("   - P99 latency: {:.2}ms", metrics.p99_latency_ms());
    println!(
        "   - Active connections: {}\n",
        broadcaster.active_connections()
    );

    // Step 7: Registry metrics
    println!("7. Registry Metrics:");
    println!("   - Total connections: {}", registry.connection_count());
    println!("   - Active users: {}", registry.active_users_count());
    println!("   - Messages sent: {}\n", registry.messages_sent());

    println!("================================");
    println!("✅ Demo completed successfully!\n");

    println!("📖 Usage Guide:");
    println!("   1. Set PubNub environment variables:");
    println!("      export PUBNUB_PUBLISH_KEY=your-publish-key");
    println!("      export PUBNUB_SUBSCRIBE_KEY=your-subscribe-key");
    println!();
    println!("   2. Start the sync service:");
    println!("      cargo run --bin sync-service");
    println!();
    println!("   3. Connect WebSocket clients:");
    println!("      ws://localhost:8083/ws");
    println!();
    println!("   4. Publish events to PubNub:");
    println!("      Channel: user.<user_id>.sync");
    println!("      Message types: watchlist_update, progress_update, device_handoff");

    Ok(())
}
