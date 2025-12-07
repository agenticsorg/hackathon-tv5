# Circuit Breaker Redis Persistence

## Overview

The circuit breaker implementation now supports Redis-based state persistence, enabling multiple gateway instances to share circuit breaker state. This ensures consistent behavior across a distributed deployment.

## Features

### 1. State Persistence
- **Automatic Sync**: Circuit state is automatically persisted to Redis on every state transition
- **JSON Format**: State is stored as JSON for easy debugging and inspection
- **TTL Management**: Redis keys expire after 1 hour to prevent stale data accumulation

### 2. State Recovery
- **Startup Recovery**: Circuit breakers load their state from Redis when first accessed
- **Cross-Instance Sharing**: Multiple gateway instances share the same circuit state
- **Graceful Degradation**: Falls back to in-memory operation if Redis is unavailable

### 3. Persisted Data

Each circuit breaker stores:
- `state`: Current state ("closed", "open", "half_open")
- `failure_count`: Number of consecutive failures
- `success_count`: Number of consecutive successes
- `last_failure_time`: Unix timestamp of last failure
- `opened_at`: Unix timestamp when circuit opened

## Usage

### Basic Usage (In-Memory Only)

```rust
use media_gateway_api::circuit_breaker::CircuitBreakerManager;
use media_gateway_api::config::Config;
use std::sync::Arc;

let config = Arc::new(Config::default());
let manager = CircuitBreakerManager::new(config);
```

### With Redis Persistence

```rust
use media_gateway_api::circuit_breaker::CircuitBreakerManager;
use media_gateway_api::config::Config;
use std::sync::Arc;

let mut config = Config::default();
config.redis.url = "redis://localhost:6379".to_string();

let manager = CircuitBreakerManager::with_redis(Arc::new(config))
    .await
    .expect("Failed to create manager with Redis");

// Circuit breaker state is now persisted to Redis
let result = manager
    .call("discovery", || {
        // Your service call here
        Ok::<_, std::io::Error>("success")
    })
    .await;
```

## Redis Key Format

Circuit breaker states are stored with the following key pattern:

```
circuit_breaker:{service}:state
```

Examples:
- `circuit_breaker:discovery:state`
- `circuit_breaker:sona:state`
- `circuit_breaker:auth:state`

## State Format

State is stored as JSON:

```json
{
  "state": "open",
  "failure_count": 15,
  "success_count": 0,
  "last_failure_time": 1733500800,
  "opened_at": 1733500795
}
```

## Configuration

### Environment Variables

```bash
# Redis connection URL
REDIS_URL="redis://localhost:6379"

# Circuit breaker settings (in config file or env)
CIRCUIT_BREAKER_ENABLED=true
```

### Config File

```yaml
circuit_breaker:
  enabled: true
  services:
    discovery:
      failure_threshold: 20
      timeout_seconds: 3
      error_rate_threshold: 0.5
    sona:
      failure_threshold: 10
      timeout_seconds: 2
      error_rate_threshold: 0.4

redis:
  url: "redis://localhost:6379"
  pool_size: 10
```

## Behavior

### State Transitions

All state transitions are automatically persisted:

1. **Closed → Open**: When failure threshold is reached
   - Persists: state="open", failure_count, opened_at timestamp

2. **Open → Half-Open**: After timeout period expires
   - Persists: state="half_open"

3. **Half-Open → Closed**: On successful request
   - Persists: state="closed", resets counters

4. **Half-Open → Open**: On failed request
   - Persists: state="open", updated opened_at timestamp

### Multi-Instance Behavior

When multiple gateway instances use Redis persistence:

1. **Instance A** opens a circuit due to failures
2. Circuit state is persisted to Redis
3. **Instance B** accesses the same service
4. **Instance B** loads the "open" state from Redis
5. **Instance B** immediately rejects requests (no retry needed)

### Failure Handling

If Redis is unavailable:
- Circuit breaker continues to work in-memory
- Warning is logged on startup
- No errors are thrown (graceful degradation)
- State transitions still work, just not shared

## Performance Considerations

### Redis Operations

- **Write Operations**: Async, non-blocking
- **Read Operations**: Only on circuit breaker initialization
- **Network Overhead**: Minimal (small JSON payloads)
- **TTL**: 1 hour prevents indefinite storage

### Optimization Tips

1. **Connection Pooling**: Use Redis connection manager (automatically configured)
2. **Async Operations**: All Redis operations are async
3. **Batching**: State updates are triggered only on state transitions, not every request

## Monitoring

### Key Metrics to Monitor

1. **Redis Connectivity**: Monitor connection failures in logs
2. **State Sync Errors**: Watch for persistence failures
3. **State Load Errors**: Monitor deserialization errors
4. **Circuit State Distribution**: Track which circuits are open across instances

### Log Messages

```
DEBUG: Circuit breaker Redis persistence enabled
DEBUG: Restored circuit breaker state from Redis (service=discovery, state=open, failure_count=15)
DEBUG: Persisted circuit breaker state to Redis (service=sona, state=closed)
ERROR: Failed to persist circuit breaker state to Redis (service=auth, error=...)
WARN: Failed to connect to Redis for circuit breaker persistence, falling back to in-memory only
```

## Testing

### Unit Tests

The implementation includes comprehensive tests:

- State persistence for all states (closed, open, half_open)
- State restoration on circuit breaker initialization
- TTL verification
- Multi-instance state sharing
- Graceful fallback without Redis

### Running Tests

```bash
# Ensure Redis is running
docker run -d -p 6379:6379 redis:7-alpine

# Run tests
cargo test --package media-gateway-api --test circuit_breaker_redis_tests

# Run with logging
RUST_LOG=debug cargo test --package media-gateway-api --test circuit_breaker_redis_tests
```

### Manual Testing

```bash
# Check circuit breaker state in Redis
redis-cli GET circuit_breaker:discovery:state

# View all circuit breaker keys
redis-cli KEYS 'circuit_breaker:*'

# Check TTL
redis-cli TTL circuit_breaker:discovery:state

# Delete all circuit breaker state
redis-cli DEL $(redis-cli KEYS 'circuit_breaker:*')
```

## Migration Guide

### Upgrading from In-Memory to Redis

1. **Update Configuration**: Add Redis URL to config
2. **Use `with_redis()` Constructor**: Replace `new()` with `with_redis().await`
3. **Handle Result**: `with_redis()` returns a Result (for error handling)
4. **Deploy**: Deploy updated gateway instances

### Backward Compatibility

The implementation maintains full backward compatibility:

- `CircuitBreakerManager::new()` still works (in-memory only)
- Existing code continues to work without changes
- Redis persistence is opt-in via `with_redis()`

### Example Migration

**Before:**
```rust
let manager = CircuitBreakerManager::new(config);
```

**After:**
```rust
let manager = CircuitBreakerManager::with_redis(config)
    .await
    .expect("Failed to initialize Redis persistence");
```

## Troubleshooting

### Redis Connection Failures

**Symptom**: Warning logs about Redis connection failures

**Solution**:
1. Verify Redis is running: `redis-cli ping`
2. Check Redis URL in configuration
3. Verify network connectivity
4. Check firewall rules

### State Not Syncing

**Symptom**: Different instances show different circuit states

**Solution**:
1. Verify all instances use same Redis URL
2. Check Redis logs for errors
3. Verify TTL hasn't expired
4. Check network latency between instances and Redis

### High Redis Load

**Symptom**: Redis CPU/network usage high

**Solution**:
1. Verify TTL is set correctly (prevents key accumulation)
2. Check for rapid state transitions (may indicate service issues)
3. Consider increasing failure thresholds to reduce transitions

## Security Considerations

### Redis Security

1. **Authentication**: Use Redis AUTH for production
   ```
   redis://username:password@host:port
   ```

2. **TLS**: Use encrypted connections for production
   ```
   rediss://host:port
   ```

3. **Network Isolation**: Keep Redis in private network

4. **Access Control**: Use Redis ACLs to limit permissions

### Data Sensitivity

Circuit breaker state contains:
- Service names
- Failure counts
- Timestamps

This is generally low-sensitivity data, but should still be protected in production environments.

## Future Enhancements

Potential improvements:

1. **Metrics Export**: Expose circuit state as Prometheus metrics
2. **Pub/Sub**: Real-time state notifications across instances
3. **State Analytics**: Track historical state transitions
4. **Dynamic Configuration**: Update thresholds without restart
5. **Circuit Health Score**: Aggregate failure patterns

## References

- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Redis Best Practices](https://redis.io/docs/manual/patterns/)
- [Distributed Systems Patterns](https://www.microsoft.com/en-us/research/publication/patterns-for-resilient-architecture/)
