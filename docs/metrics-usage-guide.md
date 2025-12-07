# Prometheus Metrics Implementation Guide

## Overview

The Media Gateway platform now includes comprehensive Prometheus metrics endpoints across all services. This document provides implementation details and usage examples.

## Implementation Details

### Location
- **Module**: `/workspaces/media-gateway/crates/core/src/metrics.rs`
- **Export**: Available via `media-gateway-core` crate

### Metrics Provided

#### HTTP Request Metrics
```rust
// Counter: Total HTTP requests
http_requests_total {method="GET", path="/api/content", status="200"}

// Histogram: Request duration in seconds
http_request_duration_seconds {method="GET", path="/api/content"}
```

#### Connection Metrics
```rust
// Gauge: Active HTTP/WebSocket connections
active_connections

// Gauge: Active database connections
db_connections_active

// Gauge: Idle database connections
db_connections_idle
```

#### Cache Metrics
```rust
// Counter: Cache hits by type
cache_hits_total {cache_type="redis"}

// Counter: Cache misses by type
cache_misses_total {cache_type="redis"}
```

### Histogram Buckets

Request duration buckets (in seconds):
```
[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
```

## Usage Examples

### 1. Basic Setup with Actix-web

```rust
use actix_web::{web, App, HttpServer};
use media_gateway_core::metrics::{metrics_handler, MetricsMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Add metrics middleware for automatic instrumentation
            .wrap(MetricsMiddleware)
            // Expose /metrics endpoint
            .route("/metrics", web::get().to(metrics_handler))
            // Your other routes...
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### 2. Manual Metric Recording

```rust
use media_gateway_core::metrics::*;
use std::time::Instant;

// Record HTTP request
async fn handle_request() -> actix_web::HttpResponse {
    let start = Instant::now();

    // ... handle request ...

    // Record metrics
    let duration = start.elapsed().as_secs_f64();
    record_http_request("GET", "/api/content", "200");
    observe_http_duration("GET", "/api/content", duration);

    actix_web::HttpResponse::Ok().finish()
}
```

### 3. Connection Tracking

```rust
use media_gateway_core::metrics::*;

// Track WebSocket connections
async fn websocket_handler() {
    increment_active_connections();

    // ... handle WebSocket connection ...

    decrement_active_connections();
}
```

### 4. Database Pool Metrics

```rust
use media_gateway_core::metrics::update_db_pool_metrics;
use sqlx::PgPool;

async fn update_pool_metrics(pool: &PgPool) {
    let active = pool.size() as usize;
    let idle = pool.num_idle();
    update_db_pool_metrics(active, idle);
}

// Call periodically (e.g., every 30 seconds)
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        update_pool_metrics(&pool).await;
    }
});
```

### 5. Cache Metrics

```rust
use media_gateway_core::metrics::*;

async fn get_from_cache(key: &str) -> Option<String> {
    match redis_client.get(key).await {
        Ok(Some(value)) => {
            record_cache_hit("redis");
            Some(value)
        }
        _ => {
            record_cache_miss("redis");
            None
        }
    }
}
```

### 6. Custom Service Integration

```rust
use media_gateway_core::METRICS_REGISTRY;
use prometheus::{Counter, Opts};

// Register custom metrics
lazy_static! {
    static ref CUSTOM_METRIC: Counter = {
        let metric = Counter::new(
            "custom_operations_total",
            "Total custom operations"
        ).unwrap();

        METRICS_REGISTRY
            .registry()
            .register(Box::new(metric.clone()))
            .unwrap();

        metric
    };
}

// Use custom metric
fn perform_custom_operation() {
    CUSTOM_METRIC.inc();
}
```

## Metrics Endpoint Response

Example `/metrics` endpoint output:

```prometheus
# HELP http_requests_total Total number of HTTP requests processed
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/api/content",status="200"} 1523

# HELP http_request_duration_seconds HTTP request latency in seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{method="GET",path="/api/content",le="0.001"} 45
http_request_duration_seconds_bucket{method="GET",path="/api/content",le="0.005"} 234
http_request_duration_seconds_bucket{method="GET",path="/api/content",le="0.01"} 456
http_request_duration_seconds_sum{method="GET",path="/api/content"} 23.45
http_request_duration_seconds_count{method="GET",path="/api/content"} 1523

# HELP active_connections Number of active HTTP/WebSocket connections
# TYPE active_connections gauge
active_connections 42

# HELP db_connections_active Number of active database connections in the pool
# TYPE db_connections_active gauge
db_connections_active 15

# HELP db_connections_idle Number of idle database connections in the pool
# TYPE db_connections_idle gauge
db_connections_idle 25

# HELP cache_hits_total Total number of cache hits
# TYPE cache_hits_total counter
cache_hits_total{cache_type="redis"} 8934

# HELP cache_misses_total Total number of cache misses
# TYPE cache_misses_total counter
cache_misses_total{cache_type="redis"} 234
```

## Prometheus Configuration

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'media-gateway-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s

  - job_name: 'media-gateway-discovery'
    static_configs:
      - targets: ['localhost:8081']
    metrics_path: '/metrics'
    scrape_interval: 15s

  - job_name: 'media-gateway-playback'
    static_configs:
      - targets: ['localhost:8082']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Grafana Dashboard Queries

### Request Rate (requests/sec)
```promql
rate(http_requests_total[5m])
```

### Average Request Duration
```promql
rate(http_request_duration_seconds_sum[5m]) /
rate(http_request_duration_seconds_count[5m])
```

### 95th Percentile Latency
```promql
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)
```

### Error Rate (5xx responses)
```promql
sum(rate(http_requests_total{status=~"5.."}[5m])) /
sum(rate(http_requests_total[5m]))
```

### Cache Hit Rate
```promql
sum(rate(cache_hits_total[5m])) /
(sum(rate(cache_hits_total[5m])) + sum(rate(cache_misses_total[5m])))
```

### Database Pool Utilization
```promql
db_connections_active /
(db_connections_active + db_connections_idle)
```

## Best Practices

1. **Cardinality Management**: Avoid high-cardinality labels (e.g., user IDs, session IDs)
2. **Path Normalization**: Normalize dynamic paths before recording:
   ```rust
   let normalized_path = path.replace(
       regex::Regex::new(r"/\d+").unwrap(),
       "/:id"
   );
   ```

3. **Periodic Updates**: Update gauge metrics periodically, not on every request

4. **Error Handling**: Always handle metric gathering errors gracefully

5. **Testing**: Include metrics assertions in integration tests

## Testing

The metrics module includes comprehensive unit tests:

```bash
cargo test -p media-gateway-core metrics
```

## Architecture

```
┌─────────────────────┐
│  Actix-web Service  │
└──────────┬──────────┘
           │
           ├─> MetricsMiddleware (automatic instrumentation)
           │
           ├─> Manual metric calls
           │
           v
    ┌──────────────────┐
    │ MetricsRegistry  │
    │  (Lazy Static)   │
    └──────────────────┘
           │
           v
    ┌──────────────────┐
    │  Prometheus      │
    │  Registry        │
    └──────────────────┘
           │
           v
    GET /metrics
    (Prometheus scraper)
```

## Service-Specific Implementation

### API Service
- HTTP request/response metrics
- Authentication success/failure rates
- Rate limit hits

### Discovery Service
- Search query metrics
- Content recommendation performance
- Vector database query latency

### Playback Service
- Stream initialization time
- Bitrate changes
- Buffer events

### Ingestion Service
- Content processing time
- Metadata extraction duration
- Storage upload metrics

## Migration Checklist

For each service, implement:

- [ ] Add `MetricsMiddleware` to Actix app
- [ ] Expose `/metrics` endpoint
- [ ] Add database pool metric updates
- [ ] Add cache metric tracking
- [ ] Configure Prometheus scraping
- [ ] Create Grafana dashboards
- [ ] Add alerts for critical metrics
- [ ] Document service-specific metrics

## Related Files

- `/workspaces/media-gateway/crates/core/src/metrics.rs` - Core implementation
- `/workspaces/media-gateway/crates/core/src/lib.rs` - Public exports
- `/workspaces/media-gateway/crates/core/Cargo.toml` - Dependencies

## References

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Prometheus Rust Client](https://docs.rs/prometheus/)
- [Actix-web Middleware](https://actix.rs/docs/middleware/)
- [Best Practices for Naming Metrics](https://prometheus.io/docs/practices/naming/)
