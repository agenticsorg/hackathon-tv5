# Auth Middleware Module

This module provides authentication and rate limiting middleware for the Media Gateway auth service.

## Modules

### `auth.rs`
Authentication middleware using JWT tokens.

**Features**:
- JWT token verification
- User context extraction
- RBAC permission checking
- Session revocation checking

**Usage**:
```rust
use media_gateway_auth::middleware::AuthMiddleware;

App::new()
    .wrap(AuthMiddleware::new(jwt_manager, session_manager, rbac_manager))
    .service(protected_handler)
```

### `rate_limit.rs`
Rate limiting middleware using Redis-backed sliding window algorithm.

**Features**:
- Per-endpoint rate limits
- Sliding window algorithm (60s windows)
- Client identification (X-Client-ID header or IP)
- Internal service bypass
- Comprehensive error responses

**Usage**:
```rust
use media_gateway_auth::middleware::{RateLimitConfig, configure_rate_limiting};

let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
let config = RateLimitConfig::default();

App::new()
    .wrap(configure_rate_limiting(redis_client, config))
    .service(token_exchange)
```

**Default Limits**:
- `/auth/token`: 10 requests/minute
- `/auth/device`: 5 requests/minute
- `/auth/authorize`: 20 requests/minute
- `/auth/revoke`: 10 requests/minute

## Testing

### Unit Tests
```bash
cargo test --package media-gateway-auth --lib middleware
```

### Integration Tests (Requires Redis)
```bash
# Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# Run tests
cargo test --package media-gateway-auth rate_limit_integration_tests -- --test-threads=1
```

## Documentation

- **Complete Guide**: `/workspaces/media-gateway/docs/auth/rate-limiting-guide.md`
- **Integration Examples**: `/workspaces/media-gateway/docs/auth/rate-limiting-integration-example.md`
- **Quick Reference**: `/workspaces/media-gateway/docs/auth/rate-limiting-quick-reference.md`

## Exports

```rust
pub use auth::{AuthMiddleware, UserContext, extract_user_context};
pub use rate_limit::{RateLimitConfig, RateLimitMiddleware, configure_rate_limiting};
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 HTTP Request                     │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│        RateLimitMiddleware (Optional)            │
│  - Check Redis for rate limit                   │
│  - Increment counter                             │
│  - Return 429 if exceeded                        │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│         AuthMiddleware (Optional)                │
│  - Verify JWT token                              │
│  - Check session revocation                      │
│  - Extract user context                          │
│  - Check RBAC permissions                        │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│              Route Handler                       │
│  - Process authenticated request                 │
└─────────────────────────────────────────────────┘
```

## Performance

### Rate Limiting
- **Latency**: 1-2ms per request (Redis operations)
- **Memory**: ~100 bytes per (endpoint, client, window)
- **Expiration**: Keys auto-expire after 120 seconds

### Authentication
- **Latency**: <1ms (JWT verification)
- **Memory**: Minimal (only user context)
- **Caching**: Session revocation cached in Redis

## Security Considerations

1. **Rate Limiting**:
   - Protects against brute force attacks
   - Prevents API abuse
   - Internal service bypass requires secret

2. **Authentication**:
   - JWT signature verification
   - Session revocation support
   - Permission-based access control

## Example: Combined Middleware

```rust
use media_gateway_auth::middleware::{
    AuthMiddleware, RateLimitConfig, configure_rate_limiting
};

let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
let rate_config = RateLimitConfig::default();

App::new()
    .wrap(configure_rate_limiting(redis_client, rate_config))
    .service(
        web::scope("/api")
            .wrap(AuthMiddleware::new(jwt_mgr, session_mgr, rbac_mgr))
            .service(protected_endpoint)
    )
    .service(
        web::scope("/auth")
            // Rate limiting only (no auth required)
            .service(token_exchange)
            .service(authorize)
    )
```

## Development

### Adding New Rate Limited Endpoints

1. Update `RateLimitConfig` struct:
```rust
pub struct RateLimitConfig {
    // Add new field
    pub new_endpoint_limit: u32,
}
```

2. Update `get_limit_for_path()`:
```rust
pub fn get_limit_for_path(&self, path: &str) -> Option<u32> {
    if path.contains("/auth/new-endpoint") {
        Some(self.new_endpoint_limit)
    }
    // ...
}
```

3. Add tests for the new endpoint

### Custom Rate Limiting Logic

Extend `RateLimitMiddleware` for custom behavior:
```rust
impl RateLimitMiddleware {
    pub fn with_custom_identifier<F>(mut self, extractor: F) -> Self
    where
        F: Fn(&ServiceRequest) -> String + 'static,
    {
        // Custom implementation
        self
    }
}
```
