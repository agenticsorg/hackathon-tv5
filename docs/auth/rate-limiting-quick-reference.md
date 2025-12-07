# Rate Limiting Quick Reference

## Quick Start

```rust
use media_gateway_auth::middleware::{RateLimitConfig, configure_rate_limiting};

// 1. Create Redis client
let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;

// 2. Configure rate limits
let config = RateLimitConfig::default()
    .with_internal_secret("my-secret".to_string());

// 3. Apply to routes
App::new()
    .wrap(configure_rate_limiting(redis_client, config))
    .service(token_exchange)
```

## Default Limits

| Endpoint | Limit (per minute) |
|----------|-------------------|
| `/auth/token` | 10 |
| `/auth/device` | 5 |
| `/auth/authorize` | 20 |
| `/auth/revoke` | 10 |

## Configuration

```rust
// Custom limits
RateLimitConfig::new(15, 8, 25, 12)
    .with_internal_secret("secret");

// Environment-based
RateLimitConfig::new(
    env::var("TOKEN_LIMIT").unwrap_or("10".to_string()).parse().unwrap(),
    env::var("DEVICE_LIMIT").unwrap_or("5".to_string()).parse().unwrap(),
    env::var("AUTHORIZE_LIMIT").unwrap_or("20".to_string()).parse().unwrap(),
    env::var("REVOKE_LIMIT").unwrap_or("10".to_string()).parse().unwrap(),
)
```

## Client Requests

### With Client ID
```bash
curl -X POST https://api.example.com/auth/token \
  -H "X-Client-ID: my-app-v1.0" \
  -d "grant_type=authorization_code&code=abc"
```

### Internal Service Bypass
```bash
curl -X POST https://api.example.com/auth/token \
  -H "X-Internal-Service: my-secret" \
  -d "grant_type=refresh_token&refresh_token=xyz"
```

## Response Headers

### Success (200 OK)
```http
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
```

### Rate Limited (429)
```http
Retry-After: 42
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 42
```

## Algorithm Details

```
Window Size: 60 seconds
Key Format: rate_limit:{endpoint}:{client_id}:{window_start}
Window Start: (current_timestamp / 60) * 60
Key TTL: 120 seconds
```

## Redis Commands

```bash
# View all rate limit keys
redis-cli KEYS "rate_limit:*"

# Check specific client
redis-cli GET "rate_limit:/auth/token:client123:1701878400"

# Monitor activity
redis-cli MONITOR | grep rate_limit

# Delete all rate limit keys (testing only)
redis-cli KEYS "rate_limit:*" | xargs redis-cli DEL
```

## Testing

```bash
# Unit tests
cargo test --package media-gateway-auth --lib middleware::rate_limit::tests

# Integration tests (requires Redis)
docker run -d -p 6379:6379 redis:7-alpine
cargo test --package media-gateway-auth rate_limit_integration_tests

# Load test
ab -n 100 -c 10 -H "X-Client-ID: test" http://localhost:8080/auth/token
```

## Common Issues

| Problem | Solution |
|---------|----------|
| Rate limits not working | Verify Redis connection |
| Too many 429s | Check client ID uniqueness |
| Bypass not working | Verify secret matches config |
| Keys accumulating | Keys auto-expire after 120s |

## Files Reference

| File | Purpose |
|------|---------|
| `src/middleware/rate_limit.rs` | Implementation |
| `src/middleware/rate_limit_integration_tests.rs` | Integration tests |
| `docs/auth/rate-limiting-guide.md` | Full documentation |
| `docs/auth/rate-limiting-integration-example.md` | Integration examples |

## Environment Variables

```bash
REDIS_URL=redis://127.0.0.1:6379
INTERNAL_SERVICE_SECRET=change-me-in-production
RATE_LIMIT_TOKEN=10
RATE_LIMIT_DEVICE=5
RATE_LIMIT_AUTHORIZE=20
RATE_LIMIT_REVOKE=10
```
