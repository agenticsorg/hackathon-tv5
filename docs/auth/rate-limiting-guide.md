# Rate Limiting Middleware Guide

## Overview

The rate limiting middleware protects auth endpoints from abuse using a sliding window algorithm backed by Redis.

## Features

- **Sliding Window Algorithm**: 60-second rolling windows for accurate rate limiting
- **Per-Endpoint Limits**: Different limits for different auth endpoints
- **Client Identification**: Uses X-Client-ID header or falls back to IP address
- **Internal Service Bypass**: Allow unlimited requests from trusted internal services
- **Informative Responses**: Returns 429 with Retry-After headers when limit exceeded
- **Rate Limit Headers**: Adds X-RateLimit-* headers to all responses

## Configuration

### Default Limits

```rust
RateLimitConfig {
    token_endpoint_limit: 10,      // /auth/token - 10 requests/minute
    device_endpoint_limit: 5,      // /auth/device - 5 requests/minute
    authorize_endpoint_limit: 20,  // /auth/authorize - 20 requests/minute
    revoke_endpoint_limit: 10,     // /auth/revoke - 10 requests/minute
    internal_service_secret: None,
}
```

### Custom Configuration

```rust
use media_gateway_auth::middleware::{RateLimitConfig, RateLimitMiddleware};

let config = RateLimitConfig::new(15, 8, 25, 12)
    .with_internal_secret("your-internal-service-secret".to_string());

let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
let rate_limiter = RateLimitMiddleware::new(redis_client, config);
```

## Integration with Actix-web

### Basic Integration

```rust
use actix_web::{App, HttpServer};
use media_gateway_auth::middleware::configure_rate_limiting;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1:6379")
        .expect("Failed to create Redis client");

    let config = RateLimitConfig::default();

    HttpServer::new(move || {
        App::new()
            .wrap(configure_rate_limiting(redis_client.clone(), config.clone()))
            .service(token_exchange)
            .service(device_authorization)
            .service(authorize)
            .service(revoke_token)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### Selective Application

Apply rate limiting only to specific routes:

```rust
use actix_web::web;

App::new()
    .service(
        web::scope("/auth")
            .wrap(configure_rate_limiting(redis_client, config))
            .route("/token", web::post().to(token_handler))
            .route("/device", web::post().to(device_handler))
    )
    .service(
        web::scope("/public")
            // No rate limiting for public endpoints
            .route("/health", web::get().to(health_handler))
    )
```

## Client Identification

### Using X-Client-ID Header (Recommended)

```bash
curl -X POST https://api.mediagateway.io/auth/token \
  -H "X-Client-ID: mobile-app-v1.2.3" \
  -d "grant_type=authorization_code&code=abc123"
```

### IP-based Fallback

If no `X-Client-ID` header is provided, the middleware falls back to the client's IP address:

```rust
// Automatically uses peer IP if X-Client-ID is missing
let client_id = extract_client_id(&req);
// Returns: "192.168.1.100" or "X-Client-ID value"
```

## Internal Service Bypass

For trusted internal services that need unlimited access:

### Server Configuration

```rust
let config = RateLimitConfig::default()
    .with_internal_secret("super-secret-internal-key-12345".to_string());
```

### Client Usage

```bash
curl -X POST https://api.mediagateway.io/auth/token \
  -H "X-Internal-Service: super-secret-internal-key-12345" \
  -d "grant_type=refresh_token&refresh_token=xyz"
```

**Security Note**: Store the internal service secret securely (environment variables, secrets manager, etc.). Never commit it to version control.

## Response Format

### Successful Request (Within Limit)

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
Content-Type: application/json

{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Rate Limited Request

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 42
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 42
Content-Type: application/json

{
  "error": "rate_limit_exceeded",
  "message": "Rate limit exceeded. Maximum 10 requests per minute allowed.",
  "retry_after": 42,
  "current_count": 11,
  "limit": 10
}
```

## Sliding Window Algorithm

The middleware uses a 60-second sliding window:

```
Window Start = (current_timestamp / 60) * 60

Redis Key = rate_limit:{endpoint}:{client_id}:{window_start}
```

### Example Timeline

```
00:00:00 - Window 1 starts
00:00:15 - Request 1 (count: 1)
00:00:30 - Request 2 (count: 2)
00:00:45 - Request 3 (count: 3)
00:01:00 - Window 2 starts (Window 1 count resets)
00:01:15 - Request 4 (count: 1 in new window)
```

### Key Expiration

Redis keys expire after 120 seconds to cover:
- Current window (60s)
- Previous window (60s) for cleanup

## Monitoring and Debugging

### Check Redis Keys

```bash
# View all rate limit keys
redis-cli KEYS "rate_limit:*"

# Check specific client's count
redis-cli GET "rate_limit:/auth/token:mobile-app-v1:1701878400"

# Monitor rate limit activity
redis-cli MONITOR | grep rate_limit
```

### Logging

Enable debug logging to see rate limit decisions:

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

Example log output:
```
DEBUG Rate limit bypassed for internal service
WARN Rate limit exceeded for client mobile-app-v1 on endpoint /auth/token: 11/10
```

## Performance Considerations

### Redis Connection Pooling

The middleware uses Redis multiplexed connections for efficient connection reuse:

```rust
// Automatically managed by redis crate
let mut conn = redis_client.get_multiplexed_async_connection().await?;
```

### Overhead

- **Latency**: ~1-2ms per request (Redis INCR + EXPIRE)
- **Memory**: ~100 bytes per unique (endpoint, client, window) combination
- **Network**: 2 Redis commands per request (INCR, EXPIRE on first request)

## Testing

### Unit Tests

```bash
cargo test --package media-gateway-auth --lib middleware::rate_limit::tests
```

### Integration Tests (Requires Redis)

```bash
# Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# Run integration tests
cargo test --package media-gateway-auth --test rate_limit_integration_tests -- --test-threads=1

# Set custom Redis URL
REDIS_URL=redis://localhost:6379 cargo test --package media-gateway-auth
```

### Load Testing

```bash
# Test rate limiting under load
ab -n 100 -c 10 -H "X-Client-ID: load-test" \
  http://localhost:8080/auth/token
```

## Best Practices

1. **Use X-Client-ID**: Always send unique client identifiers for accurate rate limiting
2. **Secure Internal Secret**: Store bypass secret in environment variables
3. **Monitor Redis**: Set up Redis monitoring and alerting
4. **Adjust Limits**: Tune limits based on actual usage patterns
5. **Cache Connections**: Reuse Redis client instances across requests
6. **Handle Errors**: Implement retry logic with exponential backoff in clients
7. **Test Failover**: Ensure graceful degradation if Redis is unavailable

## Troubleshooting

### Rate Limits Not Working

1. Verify Redis connection:
   ```bash
   redis-cli PING
   ```

2. Check Redis keys are being created:
   ```bash
   redis-cli KEYS "rate_limit:*"
   ```

3. Enable debug logging to see rate limit checks

### Too Many False Positives

- Ensure clients send unique X-Client-ID headers
- Check if NAT/proxy is causing IP address collisions
- Consider increasing limits for high-traffic endpoints

### Redis Connection Errors

- Verify Redis is running and accessible
- Check firewall rules
- Increase Redis connection pool size if needed

## Security Considerations

1. **DDoS Protection**: Rate limiting helps prevent abuse but isn't a complete DDoS solution
2. **Distributed Attacks**: IP-based limiting may not catch distributed attacks
3. **Secret Rotation**: Regularly rotate internal service secrets
4. **Redis Security**: Use Redis AUTH and TLS in production
5. **Monitoring**: Set up alerts for unusual rate limit patterns

## Future Enhancements

Potential improvements for future versions:

- [ ] Dynamic rate limit adjustment based on load
- [ ] Per-user rate limits in addition to per-client
- [ ] Redis Cluster support for high availability
- [ ] Leaky bucket algorithm option
- [ ] Custom rate limit rules via configuration file
- [ ] Metrics export (Prometheus, StatsD)
- [ ] Geographic-based rate limiting
