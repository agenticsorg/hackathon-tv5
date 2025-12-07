# Production Health Check System

## Overview

The Media Gateway platform includes a comprehensive production-ready health check system for monitoring all service dependencies. Health checks run in parallel with configurable timeouts and support both simple and detailed health endpoints.

## Features

- **Parallel Execution**: All health checks run concurrently for minimal latency
- **Timeout Protection**: Each check has a 2-second timeout to prevent hanging
- **Critical vs Non-Critical**: Components can be marked as critical or non-critical
- **Degraded State**: Service can be degraded (non-critical failures) while still serving traffic
- **Latency Tracking**: All checks report latency in milliseconds
- **Version Information**: Health responses include service version
- **HTTP Status Codes**: Returns 200 (OK) or 503 (Service Unavailable)

## Architecture

### Health Status Levels

```rust
pub enum HealthStatus {
    Healthy,    // All systems operational (HTTP 200)
    Degraded,   // Some non-critical components failing (HTTP 200)
    Unhealthy,  // Critical components failing (HTTP 503)
}
```

### Component Classification

**Critical Components** (must be healthy for service to function):
- PostgreSQL database
- Qdrant vector database

**Non-Critical Components** (degraded if unavailable):
- Redis cache (service slower but functional)

### Endpoints

1. **GET /health** - Simple health status
   - Returns minimal status for load balancer health checks
   - Fast response, minimal information
   - 200 OK if healthy/degraded, 503 if unhealthy

2. **GET /health/ready** - Detailed readiness check
   - Returns component-level health status
   - Includes latency, error messages, timestamps
   - Used for Kubernetes readiness probes

3. **GET /liveness** - Liveness probe
   - Minimal check that process is running
   - No dependency checks
   - Used for Kubernetes liveness probes

## Usage

### 1. Core Health Module

The core health module (`media-gateway-core::health`) provides all health check infrastructure:

```rust
use media_gateway_core::{
    health::{HealthChecker, AggregatedHealth, SimpleHealth},
    DatabasePool,
};
use redis::Client as RedisClient;

// Initialize components
let db_pool = DatabasePool::from_env().await?;
let redis_client = RedisClient::open("redis://localhost:6379")?;

// Build health checker
let health_checker = HealthChecker::new()
    .with_postgres(db_pool.pool().clone())
    .with_redis(redis_client)
    .with_qdrant("http://localhost:6333");

// Perform health checks
let health: AggregatedHealth = health_checker.check_all().await;
let simple: SimpleHealth = health_checker.check_simple().await;
```

### 2. Actix-Web Integration

```rust
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

struct AppState {
    health_checker: Arc<HealthChecker>,
}

async fn health(state: web::Data<AppState>) -> impl Responder {
    let simple_health = state.health_checker.check_simple().await;
    let full_health = state.health_checker.check_all().await;

    let status_code = if full_health.is_ready() {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(simple_health)
}

async fn ready(state: web::Data<AppState>) -> impl Responder {
    let health = state.health_checker.check_ready().await;

    let status_code = if health.is_ready() {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(health)
}

async fn liveness() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Register routes
App::new()
    .app_data(web::Data::new(AppState { health_checker }))
    .route("/health", web::get().to(health))
    .route("/health/ready", web::get().to(ready))
    .route("/liveness", web::get().to(liveness))
```

### 3. Custom Health Checks

Implement the `HealthCheck` trait for custom components:

```rust
use media_gateway_core::health::{HealthCheck, ComponentHealth, HealthStatus};
use async_trait::async_trait;

pub struct CustomServiceCheck {
    service_url: String,
}

#[async_trait]
impl HealthCheck for CustomServiceCheck {
    async fn check(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        // Perform check with timeout
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            check_service(&self.service_url)
        ).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(_)) => ComponentHealth::healthy("custom-service", latency_ms, true),
            Ok(Err(e)) => ComponentHealth::unhealthy(
                "custom-service",
                latency_ms,
                true,
                format!("Service check failed: {}", e)
            ),
            Err(_) => ComponentHealth::unhealthy(
                "custom-service",
                2000,
                true,
                "Health check timed out after 2s"
            ),
        }
    }

    fn name(&self) -> &str {
        "custom-service"
    }

    fn is_critical(&self) -> bool {
        true
    }
}

// Add to health checker
let checker = HealthChecker::new()
    .add_check(CustomServiceCheck { service_url: "http://api.example.com" });
```

## Response Formats

### Simple Health Response

```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### Detailed Health Response

```json
{
  "status": "degraded",
  "components": [
    {
      "name": "postgres",
      "status": "healthy",
      "latency_ms": 12,
      "critical": true
    },
    {
      "name": "redis",
      "status": "unhealthy",
      "latency_ms": 2000,
      "critical": false,
      "message": "Health check timed out after 2s"
    },
    {
      "name": "qdrant",
      "status": "healthy",
      "latency_ms": 45,
      "critical": true
    }
  ],
  "version": "0.1.0",
  "timestamp": "2025-12-06T16:15:30.123Z",
  "total_latency_ms": 48
}
```

## Performance Characteristics

### Parallel Execution

All health checks run in parallel using `futures::future::join_all()`:

- **Sequential**: 3 checks × 2s timeout = 6s maximum
- **Parallel**: max(check1, check2, check3) ≈ 2s maximum

### Timeout Behavior

Each component check has a 2-second timeout:
- PostgreSQL: `SELECT 1` query with 2s timeout
- Redis: `PING` command with 2s timeout
- Qdrant: GET /health endpoint with 2s timeout

If a check exceeds 2 seconds, it returns unhealthy with `latency_ms: 2000`.

### Typical Latencies

Under normal conditions:
- PostgreSQL: 5-50ms
- Redis: 1-10ms
- Qdrant: 10-100ms
- Total (parallel): 10-100ms

## Kubernetes Integration

### Liveness Probe

```yaml
livenessProbe:
  httpGet:
    path: /liveness
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 1
  failureThreshold: 3
```

### Readiness Probe

```yaml
readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 2
```

## Load Balancer Configuration

### Simple Health Check

```nginx
# Nginx upstream health check
upstream media_gateway {
    server api:8080 max_fails=3 fail_timeout=30s;

    # Health check configuration
    check interval=5000 rise=2 fall=3 timeout=2000 type=http;
    check_http_send "GET /health HTTP/1.0\r\n\r\n";
    check_http_expect_alive http_2xx;
}
```

### AWS ALB Target Group

```terraform
resource "aws_lb_target_group" "media_gateway" {
  health_check {
    enabled             = true
    path                = "/health"
    port                = "traffic-port"
    protocol            = "HTTP"
    healthy_threshold   = 2
    unhealthy_threshold = 3
    timeout             = 3
    interval            = 10
    matcher             = "200"
  }
}
```

## Service-Specific Configurations

### API Gateway (Port 8080)

Critical components:
- None (stateless gateway)

Non-critical components:
- Downstream services (checked separately)

### Ingestion Service (Port 8085)

Critical components:
- PostgreSQL (metadata storage)
- Qdrant (vector storage)

Non-critical components:
- Redis (job queue, can use DB fallback)

### Discovery/Search Service (Port 8081)

Critical components:
- PostgreSQL (content metadata)
- Qdrant (vector search)

Non-critical components:
- Redis (result cache, slower without it)

## Testing

### Unit Tests

```bash
cargo test --package media-gateway-core --lib health
```

### Integration Tests

```bash
# Requires PostgreSQL, Redis, Qdrant running
export DATABASE_URL="postgresql://localhost/media_gateway_test"
export REDIS_URL="redis://localhost:6379"
export QDRANT_URL="http://localhost:6333"

cargo test --test health_checks_integration_test -- --ignored
```

### Example Application

```bash
# Run the health check integration example
export DATABASE_URL="postgresql://localhost/media_gateway"
export REDIS_URL="redis://localhost:6379"
export QDRANT_URL="http://localhost:6333"

cargo run --example health_check_integration

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:8080/health/ready
curl http://localhost:8080/liveness
```

## Monitoring and Alerting

### Prometheus Metrics

Health check results should be exposed as metrics:

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref HEALTH_CHECK_DURATION: Histogram = Histogram::new(
        "health_check_duration_seconds",
        "Health check duration"
    ).unwrap();

    static ref HEALTH_CHECK_FAILURES: Counter = Counter::new(
        "health_check_failures_total",
        "Health check failures"
    ).unwrap();
}

// Record metrics during health checks
let start = Instant::now();
let health = checker.check_all().await;
HEALTH_CHECK_DURATION.observe(start.elapsed().as_secs_f64());

if health.status == HealthStatus::Unhealthy {
    HEALTH_CHECK_FAILURES.inc();
}
```

### Alert Rules

```yaml
groups:
  - name: health_checks
    rules:
      - alert: ServiceUnhealthy
        expr: health_check_status{status="unhealthy"} > 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.service }} is unhealthy"

      - alert: ServiceDegraded
        expr: health_check_status{status="degraded"} > 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Service {{ $labels.service }} is degraded"
```

## Best Practices

1. **Keep checks lightweight**: Each check should complete in <100ms under normal conditions
2. **Use appropriate timeouts**: 2 seconds balances responsiveness vs false positives
3. **Classify components correctly**: Mark as critical only if service cannot function without it
4. **Monitor check latency**: Increasing latency may indicate performance issues
5. **Test failure scenarios**: Verify correct behavior when dependencies are unavailable
6. **Document degraded behavior**: Make clear what functionality is lost in degraded state
7. **Use liveness sparingly**: Only check if process is alive, not dependencies

## Troubleshooting

### Health checks timing out

**Symptom**: All checks report 2000ms latency and unhealthy status

**Possible causes**:
- Network connectivity issues
- Firewalls blocking connections
- Services not running
- Database connection pool exhausted

**Resolution**:
1. Check service logs for connection errors
2. Verify services are running: `docker ps` or `systemctl status`
3. Test connectivity: `telnet localhost 5432`, `redis-cli ping`
4. Check database pool metrics

### False negatives (unhealthy when services are fine)

**Symptom**: Health checks fail intermittently but services work

**Possible causes**:
- Timeout too aggressive (2s may be too short)
- Network latency spikes
- Database connection pool contention

**Resolution**:
1. Review health check latency metrics
2. Increase timeout if consistently near limit
3. Scale database connection pool
4. Add retry logic for transient failures

### False positives (healthy when services are failing)

**Symptom**: Health checks pass but service requests fail

**Possible causes**:
- Health check too simple (not testing actual functionality)
- Component marked as non-critical when it should be critical
- Health check cached/stale

**Resolution**:
1. Review component criticality settings
2. Add more comprehensive health checks
3. Ensure health checks execute on every request
4. Test end-to-end functionality, not just connectivity

## References

- [Core Health Module](/workspaces/media-gateway/crates/core/src/health.rs)
- [Integration Example](/workspaces/media-gateway/examples/health_check_integration.rs)
- [Integration Tests](/workspaces/media-gateway/tests/health_checks_integration_test.rs)
- [Service Health Endpoints](/workspaces/media-gateway/crates/api/src/health.rs)
