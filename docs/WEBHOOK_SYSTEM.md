# Webhook Integration System

## Overview

The webhook integration system enables real-time content updates from streaming platforms. It provides HMAC-verified webhook receiving, Redis-based queue processing, deduplication, and comprehensive metrics tracking.

## Architecture

```
Platform → Webhook Endpoint → HMAC Verification → Deduplication → Redis Queue → Processing → Database
                                      ↓                                              ↓
                                  Rate Limit                                    Dead Letter Queue
                                      ↓
                                  Metrics
```

## Components

### 1. WebhookReceiver

Central orchestrator that coordinates webhook receiving and processing.

**Features:**
- Platform handler registration
- HMAC signature verification
- Rate limiting (configurable per platform)
- Deduplication checking
- Queue integration
- Metrics tracking

**Usage:**
```rust
use media_gateway_ingestion::webhooks::*;

let queue = Arc::new(RedisWebhookQueue::new("redis://localhost:6379", None, None, None)?);
let deduplicator = Arc::new(WebhookDeduplicator::new("redis://localhost:6379", Some(24))?);
let metrics = Arc::new(WebhookMetrics::new());

let receiver = WebhookReceiver::new(queue, deduplicator, metrics);

// Register platform handlers
let handler = Box::new(NetflixWebhookHandler::new());
let config = PlatformWebhookConfig {
    platform: "netflix".to_string(),
    secret: "your-hmac-secret".to_string(),
    rate_limit: 100, // webhooks per minute
    enabled: true,
};

receiver.register_handler(handler, config).await?;
```

### 2. HMAC Verification

SHA-256 HMAC signature verification for webhook authenticity.

**Format:** `sha256=<hex-encoded-signature>`

**Implementation:**
```rust
use media_gateway_ingestion::webhooks::verify_hmac_signature;

let is_valid = verify_hmac_signature(
    payload_bytes,
    "sha256=abcd1234...",
    "platform-secret"
)?;
```

### 3. Redis Queue System

**Stream Keys:**
- Incoming: `webhooks:incoming:{platform}`
- Dead Letter Queue: `webhooks:dlq:{platform}`

**Consumer Group:** `webhook-processors`

**Features:**
- Async processing with Redis Streams
- Consumer groups for distributed processing
- Dead letter queue for failed webhooks
- Queue statistics

### 4. Deduplication

Content-based deduplication using SHA-256 hashing.

**Hash Components:**
- Platform ID
- Event type
- Payload content (excluding signature)

**Storage:**
- Redis SET: `webhook:hash:{sha256-hash}`
- TTL: 24 hours (configurable)

### 5. Metrics

Real-time metrics tracking:
- Webhooks received
- Webhooks processed
- Failed webhooks
- Duplicate webhooks
- Rate limited requests
- Success/failure rates

## API Endpoints

### Receive Webhook

```http
POST /api/v1/webhooks/{platform}
Content-Type: application/json
X-Webhook-Signature: sha256=...

{
  "event_type": "content_added",
  "platform": "netflix",
  "timestamp": "2025-12-06T00:00:00Z",
  "payload": {
    "content_id": "12345",
    "title": "New Movie"
  },
  "signature": "sha256=..."
}
```

**Response:**
```json
{
  "event_id": "a1b2c3d4...",
  "status": "accepted"
}
```

### Register Webhook

```http
POST /api/v1/webhooks/register
Content-Type: application/json

{
  "platform": "netflix",
  "url": "https://platform.example.com/webhook",
  "event_types": ["content_added", "content_updated"],
  "secret": "your-hmac-secret"
}
```

### Get Metrics

```http
GET /api/v1/webhooks/metrics
```

**Response:**
```json
{
  "received": 1000,
  "processed": 950,
  "failed": 30,
  "duplicates": 20,
  "rate_limited": 10,
  "success_rate": 95.0,
  "failure_rate": 3.0
}
```

### Get Queue Statistics

```http
GET /api/v1/webhooks/stats
```

**Response:**
```json
{
  "pending_count": 50,
  "processing_count": 5,
  "dead_letter_count": 10,
  "total_processed": 1000
}
```

## Platform Handlers

### Netflix Handler

Specialized handler for Netflix webhook payloads.

**Required Fields:**
- `content_id` (string)
- `title` (optional string)
- `content_type` (optional string)

### Generic Handler

Fallback handler for platforms without specific implementations.

**Features:**
- Standard payload validation
- Generic processing flow
- Extensible design

## Event Types

- `content_added` - New content added to platform
- `content_updated` - Existing content updated
- `content_removed` - Content removed from platform

## Configuration

### Platform Configuration

```rust
PlatformWebhookConfig {
    platform: String,        // Platform identifier
    secret: String,          // HMAC secret key
    rate_limit: u32,        // Webhooks per minute
    enabled: bool,          // Enable/disable webhooks
}
```

### Rate Limiting

- **Default:** 100 webhooks per minute per platform
- **Algorithm:** Token bucket (via `governor` crate)
- **Response:** HTTP 429 Too Many Requests

### Deduplication TTL

- **Default:** 24 hours
- **Configurable:** Pass TTL hours to `WebhookDeduplicator::new()`

## Error Handling

### Error Types

- `InvalidSignature` - HMAC verification failed
- `InvalidPayload` - Malformed payload
- `UnsupportedPlatform` - Platform not registered
- `RateLimitExceeded` - Rate limit hit
- `QueueError` - Queue operation failed
- `DeduplicationError` - Deduplication check failed
- `ProcessingError` - Webhook processing failed

### Dead Letter Queue

Failed webhooks are moved to platform-specific DLQ:
- Stream: `webhooks:dlq:{platform}`
- Includes error message
- Requires manual intervention

## Testing

### Unit Tests

```bash
cargo test --package media-gateway-ingestion --lib webhooks
```

### Integration Tests

Requires Redis running:

```bash
export REDIS_URL=redis://localhost:6379
cargo test --package media-gateway-ingestion --test webhook_integration_test
```

### Test Coverage

- HMAC signature verification (valid/invalid/malformed)
- Payload validation (valid/invalid/missing fields)
- Deduplication (first/duplicate webhooks)
- Queue operations (enqueue/dequeue/ack/dlq)
- Rate limiting (within/exceeding limits)
- Metrics tracking (all counters)
- API endpoints (success/error paths)

## Performance Characteristics

### Throughput

- **Target:** 100 webhooks/minute per platform
- **Queue:** Asynchronous processing with Redis Streams
- **Deduplication:** O(1) Redis SET lookup

### Latency

- **Verification:** < 1ms (HMAC computation)
- **Deduplication:** < 5ms (Redis lookup)
- **Enqueue:** < 10ms (Redis XADD)

### Scalability

- **Horizontal:** Multiple consumers via Redis consumer groups
- **Vertical:** Async processing with tokio runtime
- **Storage:** Redis TTL-based automatic cleanup

## Security

### HMAC Verification

- **Algorithm:** HMAC-SHA256
- **Secret Management:** Platform-specific secrets
- **Timing Attack Protection:** Constant-time comparison

### Rate Limiting

- **Per-Platform:** Independent rate limits
- **Token Bucket:** Prevents burst attacks
- **Configurable:** Adjust per platform needs

### Input Validation

- JSON schema validation
- Platform verification
- Event type validation
- Required field checks

## Monitoring

### Metrics Collection

- Prometheus-compatible metrics
- Real-time counters (atomic operations)
- Success/failure rates
- Queue depth monitoring

### Alerting Recommendations

- Dead letter queue growth
- High failure rate (> 5%)
- Rate limit hits
- Queue backlog (> 1000 pending)

## Future Enhancements

1. **Retry Logic:** Exponential backoff for failed webhooks
2. **Batch Processing:** Process multiple webhooks in batches
3. **Webhook Replay:** Replay webhooks from DLQ
4. **Multi-Region:** Geographic distribution
5. **Advanced Rate Limiting:** Per-content-type limits
6. **Webhook Filtering:** Event type subscriptions
7. **Payload Compression:** Support compressed payloads
8. **Webhook Forwarding:** Forward to downstream services

## Dependencies

- `redis` - Redis client with streams support
- `hmac` - HMAC implementation
- `sha2` - SHA-256 hashing
- `hex` - Hex encoding/decoding
- `governor` - Rate limiting
- `actix-web` - HTTP endpoints
- `tokio` - Async runtime

## Files

### Implementation

- `/workspaces/media-gateway/crates/ingestion/src/webhooks/mod.rs` - Module definitions
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/receiver.rs` - Main receiver
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/verification.rs` - HMAC verification
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/queue.rs` - Redis queue
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/deduplication.rs` - Deduplication
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/metrics.rs` - Metrics tracking
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/api.rs` - API endpoints
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/handlers/netflix.rs` - Netflix handler
- `/workspaces/media-gateway/crates/ingestion/src/webhooks/handlers/generic.rs` - Generic handler

### Tests

- `/workspaces/media-gateway/crates/ingestion/tests/webhook_integration_test.rs` - Integration tests

## References

- [SPARC Specification](../SPARC.md)
- [BATCH_006_TASKS.md](../BATCH_006_TASKS.md)
- [Media Gateway Architecture](../ARCHITECTURE.md)
