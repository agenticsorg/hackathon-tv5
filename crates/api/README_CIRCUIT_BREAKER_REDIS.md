# Circuit Breaker with Redis Persistence - Quick Reference

## TL;DR

Circuit breaker now supports Redis persistence for sharing state across multiple gateway instances.

**Before (in-memory only):**
```rust
let manager = CircuitBreakerManager::new(config);
```

**After (with Redis persistence):**
```rust
let manager = CircuitBreakerManager::with_redis(config).await?;
```

## Why Use Redis Persistence?

### Problem Without Persistence
- Each gateway instance has independent circuit state
- Service failures affect only one instance
- Other instances continue sending requests to failing service
- Wastes resources and increases latency

### Solution With Redis Persistence
- All gateway instances share circuit state
- When one instance opens a circuit, all instances see it
- Immediate failure prevention across entire deployment
- Reduced load on failing services
- Faster recovery

## Quick Setup

### 1. Start Redis

```bash
docker run -d -p 6379:6379 redis:7-alpine
```

### 2. Configure Redis URL

**Environment Variable:**
```bash
export REDIS_URL=redis://localhost:6379
```

**Or in Config:**
```rust
let mut config = Config::default();
config.redis.url = "redis://localhost:6379".to_string();
```

### 3. Use with_redis() Constructor

```rust
use media_gateway_api::circuit_breaker::CircuitBreakerManager;
use std::sync::Arc;

let manager = CircuitBreakerManager::with_redis(Arc::new(config))
    .await
    .expect("Failed to initialize Redis persistence");
```

### 4. Use Normally

```rust
let result = manager
    .call("discovery", || {
        discovery_client.search(query)
    })
    .await?;
```

That's it! State is now persisted automatically.

## How It Works

### State Persistence Flow

```
Request → Circuit Breaker → State Change? → Persist to Redis
                          ↓
                    Load from Redis (on first access)
```

### Redis Keys

```
circuit_breaker:discovery:state
circuit_breaker:sona:state
circuit_breaker:auth:state
```

### State Data (JSON)

```json
{
  "state": "open",
  "failure_count": 15,
  "success_count": 0,
  "last_failure_time": 1733500800,
  "opened_at": 1733500795
}
```

### TTL

Keys automatically expire after **1 hour** (3600 seconds)

## Multi-Instance Behavior

### Scenario: Service Failure

```
Instance A: Failure → Open Circuit → Save to Redis
Instance B: Load from Redis → Circuit Already Open → Reject Immediately
Instance C: Load from Redis → Circuit Already Open → Reject Immediately
```

### Timeline

```
T+0s:  Instance A opens circuit (3 failures)
T+0s:  State saved to Redis
T+1s:  Instance B tries to call service
T+1s:  Instance B loads state from Redis
T+1s:  Instance B sees circuit is open
T+1s:  Instance B rejects request (no retry needed)
```

## Checking Circuit State

### In Code

```rust
// Get state for one service
let state = manager.get_state("discovery").await;
println!("Discovery circuit: {:?}", state);

// Get all circuit states
let all_states = manager.get_all_states().await;
for (service, state) in all_states {
    println!("{}: {}", service, state);
}
```

### In Redis CLI

```bash
# Get circuit state
redis-cli GET circuit_breaker:discovery:state

# See all circuit breaker keys
redis-cli KEYS 'circuit_breaker:*'

# Check TTL
redis-cli TTL circuit_breaker:discovery:state

# Pretty print JSON
redis-cli GET circuit_breaker:discovery:state | jq .
```

## Configuration Options

### Service-Specific Settings

```rust
config.circuit_breaker.services.insert(
    "discovery".to_string(),
    CircuitBreakerServiceConfig {
        failure_threshold: 20,    // Open after 20 failures
        timeout_seconds: 3,       // Half-open after 3 seconds
        error_rate_threshold: 0.5,// Open if >50% error rate
    },
);
```

### Global Settings

```rust
config.circuit_breaker.enabled = true;
config.redis.url = "redis://localhost:6379";
config.redis.pool_size = 10;
```

## Graceful Degradation

If Redis is unavailable:
- ⚠️ Warning is logged
- ✅ Circuit breaker continues in-memory
- ✅ No errors thrown
- ❌ State not shared across instances

```
WARN: Failed to connect to Redis for circuit breaker persistence,
      falling back to in-memory only
```

## Troubleshooting

### Circuit Not Opening

**Check:**
1. Failure threshold configuration
2. Actual failure count
3. Circuit breaker enabled

```rust
// Enable debug logging
RUST_LOG=debug cargo run
```

### State Not Syncing

**Check:**
1. Redis connectivity
2. Redis logs
3. Network latency
4. All instances use same Redis URL

```bash
# Test Redis connection
redis-cli ping

# Monitor Redis commands
redis-cli monitor
```

### High Redis Load

**Cause:** Too many state transitions

**Solutions:**
1. Increase failure threshold
2. Increase timeout seconds
3. Check for flapping services

## Testing

### Unit Tests

```bash
cargo test --package media-gateway-api --test circuit_breaker_redis_tests
```

### Manual Testing

```bash
# Run example
cargo run --package media-gateway-api --example circuit_breaker_with_redis

# Watch Redis
redis-cli monitor
```

## Security Best Practices

### Production Redis

```rust
// Use authentication
config.redis.url = "redis://username:password@host:port";

// Use TLS
config.redis.url = "rediss://host:port";  // Note: rediss:// not redis://
```

### Network Security

1. Keep Redis in private network
2. Use firewall rules
3. Enable Redis AUTH
4. Use Redis ACLs
5. Enable TLS encryption

## Performance

### Overhead

- **Write**: ~1-2ms per state transition
- **Read**: Only on circuit initialization
- **Payload**: ~200 bytes JSON
- **Network**: Async, non-blocking

### Scalability

- ✅ Unlimited gateway instances
- ✅ No locking overhead
- ✅ Eventually consistent
- ✅ Auto-cleanup via TTL

## Migration Guide

### Step 1: Add Redis to Config

```rust
config.redis.url = "redis://localhost:6379".to_string();
```

### Step 2: Update Constructor

```diff
- let manager = CircuitBreakerManager::new(config);
+ let manager = CircuitBreakerManager::with_redis(config).await?;
```

### Step 3: Deploy

Rolling deployment works fine. Old instances use in-memory, new instances use Redis.

### Rollback

Remove Redis URL from config and redeploy. No data loss (circuit state is ephemeral).

## Common Patterns

### With Error Handling

```rust
let manager = match CircuitBreakerManager::with_redis(config.clone()).await {
    Ok(m) => m,
    Err(e) => {
        warn!("Redis unavailable, using in-memory: {}", e);
        CircuitBreakerManager::new(config)
    }
};
```

### In Application State

```rust
pub struct AppState {
    circuit_breaker: Arc<CircuitBreakerManager>,
    // ... other fields
}

async fn init_app() -> AppState {
    let config = Arc::new(Config::from_env().unwrap());
    let circuit_breaker = Arc::new(
        CircuitBreakerManager::with_redis(config.clone())
            .await
            .expect("Failed to initialize circuit breaker")
    );

    AppState { circuit_breaker }
}
```

### With Service Calls

```rust
async fn call_discovery(
    manager: &CircuitBreakerManager,
    query: String,
) -> Result<SearchResults, ApiError> {
    manager.call("discovery", || {
        // Your service call here
        discovery_client.search(query)
    }).await
}
```

## Monitoring Checklist

Monitor these in production:

- [ ] Redis connection status
- [ ] Circuit state distribution
- [ ] State persistence errors
- [ ] State load errors
- [ ] Circuit open/close events
- [ ] Redis memory usage
- [ ] Redis CPU usage
- [ ] Network latency to Redis

## Quick Reference Commands

```bash
# Check circuit state
redis-cli GET circuit_breaker:{service}:state

# List all circuits
redis-cli KEYS 'circuit_breaker:*'

# Check TTL
redis-cli TTL circuit_breaker:{service}:state

# Clear all circuit state
redis-cli DEL $(redis-cli KEYS 'circuit_breaker:*')

# Monitor in real-time
redis-cli monitor | grep circuit_breaker

# Get state with formatting
redis-cli --raw GET circuit_breaker:discovery:state | jq .
```

## Resources

- **Full Documentation**: `/workspaces/media-gateway/docs/circuit-breaker-redis-persistence.md`
- **Example Code**: `/workspaces/media-gateway/crates/api/examples/circuit_breaker_with_redis.rs`
- **Tests**: `/workspaces/media-gateway/crates/api/tests/circuit_breaker_redis_tests.rs`
- **Implementation**: `/workspaces/media-gateway/crates/api/src/circuit_breaker.rs`

## Support

For issues or questions:
1. Check logs with `RUST_LOG=debug`
2. Verify Redis connectivity
3. Review test examples
4. Check documentation

---

**Remember**: Redis persistence is opt-in. Use `with_redis()` to enable, or `new()` for in-memory only.
