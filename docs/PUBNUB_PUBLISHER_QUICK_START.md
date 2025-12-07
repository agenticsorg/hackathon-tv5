# PubNub Publisher Quick Start Guide

## Basic Setup

### 1. Create a Publisher

```rust
use media_gateway_sync::sync::{PubNubPublisher, SyncPublisher};
use media_gateway_sync::pubnub::PubNubConfig;
use std::sync::Arc;

// Basic publisher (immediate publishing)
let config = PubNubConfig::default();
let publisher = Arc::new(PubNubPublisher::new(
    config,
    "user-123".to_string(),
    "device-abc".to_string(),
));

// OR with batching (recommended for high-frequency updates)
let publisher = Arc::new(PubNubPublisher::new_with_batching(
    config,
    "user-123".to_string(),
    "device-abc".to_string(),
));
```

### 2. Integrate with Sync Managers

```rust
use media_gateway_sync::sync::{WatchlistSync, ProgressSync};

// Create watchlist sync with publisher
let watchlist = WatchlistSync::new_with_publisher(
    "user-123".to_string(),
    "device-abc".to_string(),
    Arc::clone(&publisher),
);

// Create progress sync with publisher
let progress = ProgressSync::new_with_publisher(
    "user-123".to_string(),
    "device-abc".to_string(),
    Arc::clone(&publisher),
);
```

### 3. Updates Automatically Publish

```rust
// Add to watchlist - automatically publishes
let update = watchlist.add_to_watchlist("movie-456".to_string());

// Remove from watchlist - automatically publishes
let updates = watchlist.remove_from_watchlist("movie-456");

// Update progress - automatically publishes
let update = progress.update_progress(
    "movie-456".to_string(),
    120,  // position_seconds
    1800, // duration_seconds
    PlaybackState::Playing,
);
```

## Advanced Usage

### Manual Publishing

```rust
use media_gateway_sync::sync::{SyncMessage, MessagePayload};

// Create custom message
let message = SyncMessage {
    payload: MessagePayload::ProgressUpdate {
        content_id: "movie-123".to_string(),
        position_seconds: 100,
        duration_seconds: 1000,
        state: "Playing".to_string(),
        timestamp: HLCTimestamp::new(1000, 0, "device-1".to_string()),
    },
    timestamp: chrono::Utc::now().to_rfc3339(),
    operation_type: "progress_update".to_string(),
    device_id: "device-abc".to_string(),
    message_id: uuid::Uuid::new_v4().to_string(),
};

// Publish manually
publisher.publish(message).await?;
```

### Batch Publishing

```rust
// Collect messages
let mut messages = Vec::new();
for i in 0..10 {
    messages.push(create_message(i));
}

// Publish batch
publisher.publish_batch(messages).await?;

// Manually flush pending batches
publisher.flush().await?;
```

## Configuration

### Environment Variables

```bash
# PubNub credentials
export PUBNUB_PUBLISH_KEY="your-publish-key"
export PUBNUB_SUBSCRIBE_KEY="your-subscribe-key"
```

### Custom Configuration

```rust
use media_gateway_sync::pubnub::PubNubConfig;

let config = PubNubConfig {
    publish_key: "your-publish-key".to_string(),
    subscribe_key: "your-subscribe-key".to_string(),
    origin: "ps.pndsn.com".to_string(),
};
```

## Error Handling

```rust
use media_gateway_sync::sync::PublisherError;

match publisher.publish_watchlist_update(update).await {
    Ok(_) => println!("Published successfully"),
    Err(PublisherError::PublishFailed { channel, attempts, source }) => {
        eprintln!("Failed to publish to {} after {} attempts: {}",
                  channel, attempts, source);
    }
    Err(e) => eprintln!("Publish error: {}", e),
}
```

## Channel Structure

All messages publish to: `user.{userId}.sync`

Examples:
- `user.user-123.sync`
- `user.alice@example.com.sync`

## Message Format

```json
{
  "type": "watchlist_update",
  "operation": "Add",
  "content_id": "movie-456",
  "unique_tag": "tag-abc-123",
  "timestamp": {
    "counter": 1000,
    "node_id": "device-abc"
  },
  "timestamp": "2024-01-01T12:00:00Z",
  "operation_type": "watchlist_add",
  "device_id": "device-abc",
  "message_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Performance Tips

1. **Use batching** for high-frequency updates (>10/second)
2. **Keep publishers alive** - don't recreate per request
3. **Share publishers** via Arc across threads
4. **Monitor logs** for retry patterns
5. **Handle errors gracefully** - local ops should succeed

## Troubleshooting

### Problem: Messages not publishing

**Check:**
- PubNub credentials configured correctly
- Network connectivity
- Channel name format: `user.{userId}.sync`
- Logs for error messages

### Problem: High latency

**Solutions:**
- Enable batching for bulk operations
- Reduce batch flush interval
- Check network conditions
- Monitor retry rates

### Problem: Memory usage growing

**Causes:**
- Large batch sizes accumulating
- Messages not being flushed
- Network failures preventing publishes

**Solutions:**
- Reduce MAX_BATCH_SIZE
- Lower BATCH_FLUSH_INTERVAL_MS
- Implement circuit breaker pattern

## Testing

### Unit Testing with Mock Publisher

```rust
use async_trait::async_trait;
use media_gateway_sync::sync::{SyncPublisher, PublisherError};

struct MockPublisher {
    published: Arc<Mutex<Vec<SyncMessage>>>,
}

#[async_trait]
impl SyncPublisher for MockPublisher {
    async fn publish(&self, message: SyncMessage) -> Result<(), PublisherError> {
        self.published.lock().await.push(message);
        Ok(())
    }

    // ... implement other methods
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_end_to_end_publishing() {
    let config = PubNubConfig::default();
    let publisher = Arc::new(PubNubPublisher::new(
        config,
        "test-user".to_string(),
        "test-device".to_string(),
    ));

    let watchlist = WatchlistSync::new_with_publisher(
        "test-user".to_string(),
        "test-device".to_string(),
        publisher,
    );

    let update = watchlist.add_to_watchlist("test-content".to_string());

    // Wait for async publish
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify via PubNub history API
}
```

## Monitoring

### Key Metrics to Track

- **Publish success rate**: Should be >99%
- **Retry rate**: Should be <5%
- **Batch size**: Monitor for optimization
- **Publish latency**: p50, p95, p99
- **Error rate by type**: Network vs serialization

### Log Patterns to Watch

```
WARN  Publish attempt 1 failed, retrying in 100ms
ERROR Failed to publish to channel user.user-123.sync after 3 attempts
INFO  Successfully flushed batch of 50 messages
```

## Best Practices

1. ✅ **Create one publisher per user/device pair**
2. ✅ **Use batching for progress updates** (frequent)
3. ✅ **Use direct publish for watchlist** (infrequent, user-initiated)
4. ✅ **Share publisher instances via Arc**
5. ✅ **Handle errors without failing operations**
6. ✅ **Monitor logs for retry patterns**
7. ✅ **Test with mock publishers in unit tests**
8. ✅ **Use integration tests for PubNub connectivity**

## Common Patterns

### Pattern 1: Application-Wide Publisher

```rust
pub struct AppState {
    publisher: Arc<dyn SyncPublisher>,
    // ... other state
}

impl AppState {
    pub fn new() -> Self {
        let config = PubNubConfig::default();
        let publisher = Arc::new(PubNubPublisher::new_with_batching(
            config,
            "user-id".to_string(),
            "device-id".to_string(),
        ));

        Self { publisher }
    }
}
```

### Pattern 2: Per-Request Sync Managers

```rust
async fn handle_request(
    state: Arc<AppState>,
    user_id: String,
    device_id: String,
) -> Result<Response> {
    let watchlist = WatchlistSync::new_with_publisher(
        user_id,
        device_id,
        Arc::clone(&state.publisher),
    );

    // Use watchlist...
}
```

### Pattern 3: Graceful Degradation

```rust
let update = watchlist.add_to_watchlist("content-123".to_string());

// Local operation succeeded regardless of publish result
// Real-time sync may be delayed but state is consistent
```

## Support

For issues or questions:
1. Check logs for error messages
2. Verify PubNub configuration
3. Review integration tests
4. Consult main documentation: BATCH_002_TASK-003_IMPLEMENTATION.md

---

**Last Updated**: 2025-12-06
**Version**: 1.0.0
