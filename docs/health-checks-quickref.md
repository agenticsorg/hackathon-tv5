# Health Checks Quick Reference

## TL;DR

```rust
use media_gateway_core::health::HealthChecker;

let checker = HealthChecker::new()
    .with_postgres(db_pool)
    .with_redis(redis_client)
    .with_qdrant("http://localhost:6333");

let health = checker.check_all().await;  // Full details
let simple = checker.check_simple().await;  // Status only
```

## Endpoints

| Endpoint | Purpose | Response | When to Use |
|----------|---------|----------|-------------|
| `GET /health` | Simple status | `{"status":"healthy","version":"0.1.0"}` | Load balancers |
| `GET /health/ready` | Full details | Component list + latencies | K8s readiness probe |
| `GET /liveness` | Process alive | `{"status":"alive"}` | K8s liveness probe |

## Status Codes

| Status | Meaning | HTTP Code | Still Serving? |
|--------|---------|-----------|----------------|
| `healthy` | All good | 200 | Yes ✅ |
| `degraded` | Non-critical down | 200 | Yes ✅ |
| `unhealthy` | Critical down | 503 | No ❌ |

## Component Types

| Component | Critical? | Impact if Down |
|-----------|-----------|----------------|
| PostgreSQL | Yes | Service fails |
| Qdrant | Yes | Service fails |
| Redis | No | Service slower (degraded) |

## Actix-Web Integration

```rust
use actix_web::{web, App, HttpResponse};
use std::sync::Arc;

struct AppState {
    health_checker: Arc<HealthChecker>,
}

async fn health(state: web::Data<AppState>) -> HttpResponse {
    let health = state.health_checker.check_all().await;
    let status = if health.is_ready() { 200 } else { 503 };
    HttpResponse::build(actix_web::http::StatusCode::from_u16(status).unwrap())
        .json(health)
}

App::new()
    .app_data(web::Data::new(AppState { health_checker }))
    .route("/health", web::get().to(health))
```

## Custom Health Check

```rust
use media_gateway_core::health::{HealthCheck, ComponentHealth};
use async_trait::async_trait;

struct MyCheck;

#[async_trait]
impl HealthCheck for MyCheck {
    async fn check(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        let result = tokio::time::timeout(
            Duration::from_secs(2),
            do_check()
        ).await;

        let ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(_)) => ComponentHealth::healthy("my-check", ms, true),
            _ => ComponentHealth::unhealthy("my-check", ms, true, "Failed"),
        }
    }

    fn name(&self) -> &str { "my-check" }
    fn is_critical(&self) -> bool { true }
}

let checker = HealthChecker::new().add_check(MyCheck);
```

## Environment Variables

```bash
DATABASE_URL="postgresql://localhost/media_gateway"
REDIS_URL="redis://localhost:6379"
QDRANT_URL="http://localhost:6333"
```

## Kubernetes Probes

```yaml
livenessProbe:
  httpGet:
    path: /liveness
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
```

## Testing

```bash
# Unit tests
cargo test --package media-gateway-core --lib health

# Integration tests (requires infrastructure)
export DATABASE_URL="postgresql://localhost/test"
export REDIS_URL="redis://localhost:6379"
export QDRANT_URL="http://localhost:6333"
cargo test --test health_checks_integration_test -- --ignored

# Run example
cargo run --example health_check_integration
```

## Response Examples

### Healthy
```json
{
  "status": "healthy",
  "components": [
    {"name": "postgres", "status": "healthy", "latency_ms": 12, "critical": true},
    {"name": "redis", "status": "healthy", "latency_ms": 5, "critical": false},
    {"name": "qdrant", "status": "healthy", "latency_ms": 45, "critical": true}
  ],
  "version": "0.1.0",
  "timestamp": "2025-12-06T16:15:30Z",
  "total_latency_ms": 48
}
```

### Degraded (Redis down)
```json
{
  "status": "degraded",
  "components": [
    {"name": "postgres", "status": "healthy", "latency_ms": 12, "critical": true},
    {"name": "redis", "status": "unhealthy", "latency_ms": 2000, "critical": false,
     "message": "Health check timed out after 2s"},
    {"name": "qdrant", "status": "healthy", "latency_ms": 45, "critical": true}
  ],
  "version": "0.1.0",
  "timestamp": "2025-12-06T16:15:30Z",
  "total_latency_ms": 2045
}
```

### Unhealthy (Postgres down)
```json
{
  "status": "unhealthy",
  "components": [
    {"name": "postgres", "status": "unhealthy", "latency_ms": 2000, "critical": true,
     "message": "Database query failed: connection refused"},
    {"name": "redis", "status": "healthy", "latency_ms": 5, "critical": false},
    {"name": "qdrant", "status": "healthy", "latency_ms": 45, "critical": true}
  ],
  "version": "0.1.0",
  "timestamp": "2025-12-06T16:15:30Z",
  "total_latency_ms": 2045
}
```

## Key Files

- Core: `/workspaces/media-gateway/crates/core/src/health.rs`
- Example: `/workspaces/media-gateway/examples/health_check_integration.rs`
- Tests: `/workspaces/media-gateway/tests/health_checks_integration_test.rs`
- Docs: `/workspaces/media-gateway/docs/health-checks.md`

## Common Issues

| Issue | Solution |
|-------|----------|
| All checks timeout | Check network connectivity |
| False negatives | Increase timeout beyond 2s |
| False positives | Add more comprehensive checks |
| Slow health checks | Ensure parallel execution |

## Performance

- **Parallel execution**: ~2s max (not 6s sequential)
- **Per-check timeout**: 2 seconds
- **Typical latency**: 10-100ms when healthy
- **Memory**: Minimal
- **CPU**: Negligible
