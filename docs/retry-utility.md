# Exponential Backoff Retry Utility

## Overview

The retry utility (`media_gateway_core::retry`) provides production-ready exponential backoff retry logic for handling transient failures in distributed systems. It's designed to integrate seamlessly with the Media Gateway error handling system.

## Features

- **Exponential Backoff**: Automatically increases delay between retries using `delay = base * 2^attempt`
- **Delay Capping**: Prevents excessive wait times with configurable maximum delay
- **Jitter**: Adds randomness to prevent thundering herd problems
- **Error Discrimination**: Only retries errors that are explicitly marked as retryable
- **Type Safety**: Fully generic over error types and async operations
- **Zero-cost Abstractions**: No runtime overhead when not retrying

## Basic Usage

```rust
use media_gateway_core::retry::{retry_with_backoff, RetryPolicy};
use media_gateway_core::error::MediaGatewayError;

// Retry a network operation with default policy
let result = retry_with_backoff(
    || async {
        // Your async operation here
        make_http_request().await
    },
    RetryPolicy::default(),
    |err: &MediaGatewayError| err.is_retryable(),
).await?;
```

## Retry Policies

### Default Policy

Balanced policy suitable for most use cases:

- **max_retries**: 3
- **base_delay_ms**: 100ms
- **max_delay_ms**: 5000ms (5 seconds)
- **jitter**: enabled

```rust
let policy = RetryPolicy::default();
```

**Delay progression**: 100ms → 200ms → 400ms → 800ms (capped at 5s)

### Aggressive Policy

For critical operations that must succeed:

- **max_retries**: 5
- **base_delay_ms**: 50ms
- **max_delay_ms**: 5000ms
- **jitter**: enabled

```rust
let policy = RetryPolicy::aggressive();
```

**Use cases**:
- Database connection establishment
- Authentication token refresh
- Critical API calls
- Payment processing

### Gentle Policy

For non-critical operations:

- **max_retries**: 2
- **base_delay_ms**: 500ms
- **max_delay_ms**: 3000ms
- **jitter**: enabled

```rust
let policy = RetryPolicy::gentle();
```

**Use cases**:
- Analytics events
- Background jobs
- Cache updates
- Logging to external services

### Custom Policy

Create your own policy for specific requirements:

```rust
let policy = RetryPolicy::new(
    10,    // max_retries
    200,   // base_delay_ms
    8000,  // max_delay_ms
    true,  // jitter
);
```

## Error Discrimination

The retry utility respects error semantics. Only retry errors that are transient:

```rust
retry_with_backoff(
    operation,
    policy,
    |err: &MediaGatewayError| match err {
        // Retry these
        MediaGatewayError::NetworkError { .. } => true,
        MediaGatewayError::TimeoutError { .. } => true,
        MediaGatewayError::ServiceUnavailableError { .. } => true,
        MediaGatewayError::RateLimitError { .. } => true,

        // Don't retry these
        MediaGatewayError::ValidationError { .. } => false,
        MediaGatewayError::AuthenticationError { .. } => false,
        MediaGatewayError::NotFoundError { .. } => false,
        _ => false,
    },
).await
```

The `MediaGatewayError` type provides a built-in `is_retryable()` method:

```rust
retry_with_backoff(
    operation,
    policy,
    |err: &MediaGatewayError| err.is_retryable(),
).await
```

## Advanced Examples

### Database Connection with Retry

```rust
use media_gateway_core::retry::{retry_with_backoff, RetryPolicy};
use sqlx::PgPool;

async fn connect_to_database(database_url: &str) -> Result<PgPool, MediaGatewayError> {
    retry_with_backoff(
        || async {
            PgPool::connect(database_url)
                .await
                .map_err(|e| MediaGatewayError::DatabaseError {
                    message: e.to_string(),
                    operation: "connect".to_string(),
                    source: Some(Box::new(e)),
                })
        },
        RetryPolicy::aggressive(),
        |err: &MediaGatewayError| err.is_retryable(),
    )
    .await
}
```

### HTTP Request with Retry

```rust
use media_gateway_core::retry::{retry_with_backoff, RetryPolicy};
use reqwest::Client;

async fn fetch_with_retry(
    client: &Client,
    url: &str,
) -> Result<String, MediaGatewayError> {
    retry_with_backoff(
        || async {
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|e| MediaGatewayError::NetworkError {
                    message: e.to_string(),
                    source: Some(Box::new(e)),
                })?;

            if response.status().is_server_error() {
                return Err(MediaGatewayError::ExternalAPIError {
                    api: "external-service".to_string(),
                    message: "Server error".to_string(),
                    status_code: Some(response.status().as_u16()),
                    source: None,
                });
            }

            response
                .text()
                .await
                .map_err(|e| MediaGatewayError::NetworkError {
                    message: e.to_string(),
                    source: Some(Box::new(e)),
                })
        },
        RetryPolicy::default(),
        |err: &MediaGatewayError| matches!(
            err,
            MediaGatewayError::NetworkError { .. }
                | MediaGatewayError::ExternalAPIError { status_code: Some(500..=599), .. }
        ),
    )
    .await
}
```

### Redis Operation with Retry

```rust
use media_gateway_core::retry::{retry_with_backoff, RetryPolicy};
use redis::AsyncCommands;

async fn get_from_cache(
    redis: &mut redis::aio::ConnectionManager,
    key: &str,
) -> Result<Option<String>, MediaGatewayError> {
    retry_with_backoff(
        || async {
            redis
                .get(key)
                .await
                .map_err(|e| MediaGatewayError::CacheError {
                    operation: "get".to_string(),
                    message: e.to_string(),
                })
        },
        RetryPolicy::gentle(), // Cache is non-critical
        |_| true, // Retry all cache errors
    )
    .await
}
```

## Best Practices

### 1. Choose the Right Policy

- **Aggressive**: Critical path operations (auth, payments, core business logic)
- **Default**: Standard API calls, database queries
- **Gentle**: Analytics, logging, background tasks

### 2. Enable Jitter

Always enable jitter in production to prevent thundering herd:

```rust
let policy = RetryPolicy::new(3, 100, 5000, true); // jitter enabled
```

### 3. Implement Circuit Breaker for Persistent Failures

Combine retry logic with circuit breaker pattern for better resilience:

```rust
// After multiple failed retry attempts, open circuit
if consecutive_failures > threshold {
    circuit_breaker.open();
    return Err(error);
}
```

### 4. Log Retry Attempts

The retry utility automatically logs via `tracing`:

```rust
// Logs are emitted at debug/warn levels
// Configure your tracing subscriber to capture them
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### 5. Set Appropriate Timeouts

Combine retry with operation timeouts:

```rust
use tokio::time::timeout;
use std::time::Duration;

let result = timeout(
    Duration::from_secs(30),
    retry_with_backoff(operation, policy, is_retryable)
).await??;
```

### 6. Monitor Retry Metrics

Track retry attempts for observability:

```rust
use media_gateway_core::metrics::METRICS_REGISTRY;

// Increment retry counter
if let Err(_) = operation().await {
    METRICS_REGISTRY
        .retry_attempts
        .with_label_values(&["service_name"])
        .inc();
}
```

## Mathematical Details

### Exponential Backoff Formula

```
delay = min(base_delay * 2^attempt, max_delay)
```

Example with `base_delay=100ms`, `max_delay=5000ms`:

| Attempt | Calculation | Actual Delay |
|---------|-------------|--------------|
| 0 | 100 * 2^0 = 100 | 100ms |
| 1 | 100 * 2^1 = 200 | 200ms |
| 2 | 100 * 2^2 = 400 | 400ms |
| 3 | 100 * 2^3 = 800 | 800ms |
| 4 | 100 * 2^4 = 1600 | 1600ms |
| 5 | 100 * 2^5 = 3200 | 3200ms |
| 6 | 100 * 2^6 = 6400 | 5000ms (capped) |

### Jitter Calculation

Jitter adds 0-30% random delay to prevent synchronized retries:

```
final_delay = base_delay + random(0, base_delay * 0.3)
```

This prevents thundering herd when multiple clients retry simultaneously.

## Testing

The retry module includes comprehensive tests:

```bash
# Run all retry tests
cargo test -p media-gateway-core retry

# Run specific test
cargo test -p media-gateway-core test_retry_succeeds_after_failures

# Run with output
cargo test -p media-gateway-core retry -- --nocapture
```

## Performance Considerations

### Time Complexity

- **Best case**: O(1) - operation succeeds immediately
- **Worst case**: O(n) where n = max_retries
- **Average case**: Depends on transient failure rate

### Memory Usage

- Minimal overhead: single policy struct + closure state
- No allocations during retry loop
- Async-friendly: yields to executor during delays

### Delay Accumulation

Total time with default policy (worst case):

```
100ms + 200ms + 400ms = 700ms + operation time
```

With aggressive policy:

```
50ms + 100ms + 200ms + 400ms + 800ms = 1550ms + operation time
```

## Integration with Media Gateway

The retry utility integrates seamlessly with existing Media Gateway components:

### With Database Pool

```rust
use media_gateway_core::database::DatabasePool;

let pool = retry_with_backoff(
    || DatabasePool::new(&config),
    RetryPolicy::aggressive(),
    |_| true,
).await?;
```

### With External APIs

```rust
use media_gateway_core::retry::{retry_with_backoff, RetryPolicy};

let tmdb_data = retry_with_backoff(
    || fetch_from_tmdb(&movie_id),
    RetryPolicy::default(),
    |err| err.is_retryable(),
).await?;
```

### With Health Checks

```rust
use media_gateway_core::health::HealthCheck;

impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> HealthStatus {
        match retry_with_backoff(
            || self.pool.acquire(),
            RetryPolicy::gentle(),
            |_| true,
        ).await {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }
}
```

## Error Handling

### Exhausted Retries

When all retries are exhausted, the last error is returned:

```rust
match retry_with_backoff(operation, policy, is_retryable).await {
    Ok(result) => {
        // Success
    }
    Err(err) => {
        // All retries exhausted - handle final error
        log::error!("Operation failed after retries: {}", err);
    }
}
```

### Non-Retryable Errors

Non-retryable errors fail immediately without consuming retries:

```rust
// This will fail on first attempt without retrying
retry_with_backoff(
    || async { Err(MediaGatewayError::ValidationError { .. }) },
    RetryPolicy::default(),
    |err| err.is_retryable(), // Returns false for ValidationError
).await
```

## Future Enhancements

Planned improvements for the retry utility:

1. **Circuit Breaker Integration**: Automatic circuit opening on persistent failures
2. **Adaptive Retry**: Dynamic policy adjustment based on success rates
3. **Retry Budget**: Prevent retry storms with configurable budgets
4. **Metrics Integration**: Built-in Prometheus metrics for retry attempts
5. **Deadline-Based Retries**: Stop retrying when deadline is reached
6. **Custom Jitter Strategies**: Pluggable jitter algorithms

## References

- [Exponential Backoff And Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [Retry Pattern - Microsoft Azure](https://docs.microsoft.com/en-us/azure/architecture/patterns/retry)
- [Google SRE Book - Handling Overload](https://sre.google/sre-book/handling-overload/)

## License

MIT License - See LICENSE file for details
