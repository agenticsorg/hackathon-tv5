# Rate Limiting Integration Example

## Complete Server Setup with Rate Limiting

This example shows how to integrate the rate limiting middleware into the auth server.

### Updated server.rs

```rust
use crate::{
    error::{AuthError, Result},
    jwt::JwtManager,
    middleware::{RateLimitConfig, RateLimitMiddleware},
    oauth::{OAuthConfig, OAuthManager},
    rbac::RbacManager,
    scopes::ScopeManager,
    session::SessionManager,
    storage::AuthStorage,
};
use actix_web::{
    get, post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use std::sync::Arc;

pub async fn start_server_with_rate_limiting(
    bind_address: &str,
    jwt_manager: Arc<JwtManager>,
    session_manager: Arc<SessionManager>,
    oauth_config: OAuthConfig,
    storage: Arc<AuthStorage>,
    redis_url: &str,
) -> std::io::Result<()> {
    let app_state = Data::new(AppState {
        jwt_manager,
        session_manager,
        oauth_manager: Arc::new(OAuthManager::new(oauth_config)),
        rbac_manager: Arc::new(RbacManager::new()),
        scope_manager: Arc::new(ScopeManager::new()),
        storage,
    });

    // Configure rate limiting
    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client for rate limiting");

    let rate_limit_config = RateLimitConfig::new(
        10,  // token endpoint: 10/min
        5,   // device endpoint: 5/min
        20,  // authorize endpoint: 20/min
        10,  // revoke endpoint: 10/min
    )
    .with_internal_secret(
        std::env::var("INTERNAL_SERVICE_SECRET")
            .unwrap_or_else(|_| "change-me-in-production".to_string())
    );

    tracing::info!("Starting auth service with rate limiting on {}", bind_address);
    tracing::info!("Rate limits: token=10/min, device=5/min, authorize=20/min, revoke=10/min");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Apply rate limiting to all /auth/* endpoints
            .service(
                web::scope("/auth")
                    .wrap(RateLimitMiddleware::new(
                        redis_client.clone(),
                        rate_limit_config.clone(),
                    ))
                    .service(authorize)
                    .service(token_exchange)
                    .service(revoke_token)
                    .service(device_authorization)
                    .service(device_poll)
            )
            // Health check without rate limiting
            .service(health_check)
    })
    .bind(bind_address)?
    .run()
    .await
}
```

### Updated main.rs

```rust
use media_gateway_auth::{
    jwt::JwtManager,
    oauth::OAuthConfig,
    server::start_server_with_rate_limiting,
    session::SessionManager,
    storage::AuthStorage,
};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    // Load configuration
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    // Initialize components
    let storage = Arc::new(
        AuthStorage::new(&database_url)
            .await
            .expect("Failed to initialize storage")
    );

    let jwt_manager = Arc::new(
        JwtManager::new(&jwt_secret)
            .expect("Failed to initialize JWT manager")
    );

    let session_manager = Arc::new(
        SessionManager::new(&redis_url)
            .await
            .expect("Failed to initialize session manager")
    );

    let oauth_config = OAuthConfig {
        issuer: "https://auth.mediagateway.io".to_string(),
        authorization_endpoint: "https://auth.mediagateway.io/auth/authorize".to_string(),
        token_endpoint: "https://auth.mediagateway.io/auth/token".to_string(),
        device_authorization_endpoint: Some("https://auth.mediagateway.io/auth/device".to_string()),
        registered_clients: vec![],
    };

    // Start server with rate limiting
    let bind_address = "0.0.0.0:8080";
    start_server_with_rate_limiting(
        bind_address,
        jwt_manager,
        session_manager,
        oauth_config,
        storage,
        &redis_url,
    )
    .await
}
```

### Environment Variables

Create a `.env` file:

```bash
# Database
DATABASE_URL=postgres://user:password@localhost/auth_db

# Redis (for sessions and rate limiting)
REDIS_URL=redis://127.0.0.1:6379

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-here

# Rate Limiting
INTERNAL_SERVICE_SECRET=internal-service-bypass-secret-xyz

# Logging
RUST_LOG=info,media_gateway_auth=debug
```

### Docker Compose Setup

```yaml
version: '3.8'

services:
  auth-service:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://auth_user:auth_pass@postgres:5432/auth_db
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=${JWT_SECRET}
      - INTERNAL_SERVICE_SECRET=${INTERNAL_SERVICE_SECRET}
      - RUST_LOG=info
    depends_on:
      - postgres
      - redis

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=auth_user
      - POSTGRES_PASSWORD=auth_pass
      - POSTGRES_DB=auth_db
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes

volumes:
  postgres_data:
  redis_data:
```

### Testing the Rate Limiter

```bash
# Start services
docker-compose up -d

# Test normal requests (should succeed)
for i in {1..10}; do
  curl -X POST http://localhost:8080/auth/token \
    -H "X-Client-ID: test-client" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=authorization_code&code=test123"
  echo ""
done

# 11th request should return 429
curl -X POST http://localhost:8080/auth/token \
  -H "X-Client-ID: test-client" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code&code=test123" \
  -v

# Test internal bypass (should always succeed)
for i in {1..20}; do
  curl -X POST http://localhost:8080/auth/token \
    -H "X-Client-ID: internal-service" \
    -H "X-Internal-Service: internal-service-bypass-secret-xyz" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=refresh_token&refresh_token=xyz"
  echo ""
done
```

### Monitoring Rate Limits

```bash
# Watch Redis keys
redis-cli --scan --pattern "rate_limit:*"

# Monitor in real-time
redis-cli MONITOR | grep rate_limit

# Check specific client
redis-cli GET "rate_limit:/auth/token:test-client:1701878400"

# View all rate limit counters
redis-cli KEYS "rate_limit:*" | while read key; do
  echo "$key: $(redis-cli GET $key)"
done
```

### Production Configuration

```rust
// For production, use environment-based configuration
let rate_limit_config = RateLimitConfig::new(
    std::env::var("RATE_LIMIT_TOKEN")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10),
    std::env::var("RATE_LIMIT_DEVICE")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5),
    std::env::var("RATE_LIMIT_AUTHORIZE")
        .unwrap_or_else(|_| "20".to_string())
        .parse()
        .unwrap_or(20),
    std::env::var("RATE_LIMIT_REVOKE")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10),
)
.with_internal_secret(
    std::env::var("INTERNAL_SERVICE_SECRET")
        .expect("INTERNAL_SERVICE_SECRET must be set in production")
);
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: auth-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: auth-service
  template:
    metadata:
      labels:
        app: auth-service
    spec:
      containers:
      - name: auth-service
        image: mediagateway/auth-service:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: database-url
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: jwt-secret
        - name: INTERNAL_SERVICE_SECRET
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: internal-service-secret
        - name: RATE_LIMIT_TOKEN
          value: "100"  # Higher limits for production
        - name: RATE_LIMIT_DEVICE
          value: "50"
        - name: RATE_LIMIT_AUTHORIZE
          value: "200"
        - name: RATE_LIMIT_REVOKE
          value: "100"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 3
```

## Advanced Configuration

### Per-User Rate Limiting

For more granular control, you can extend the middleware to support per-user limits:

```rust
// Future enhancement: per-user rate limiting
let client_id = if let Some(user_ctx) = req.extensions().get::<UserContext>() {
    format!("user:{}", user_ctx.user_id)
} else {
    extract_client_id(&req)
};
```

### Dynamic Rate Limit Adjustment

```rust
// Adjust limits based on subscription tier
impl RateLimitConfig {
    pub fn for_tier(tier: &str) -> Self {
        match tier {
            "free" => Self::new(10, 5, 20, 10),
            "pro" => Self::new(100, 50, 200, 100),
            "enterprise" => Self::new(1000, 500, 2000, 1000),
            _ => Self::default(),
        }
    }
}
```
