# Prometheus Metrics Endpoint Documentation

## Overview

The Media Gateway API exposes Prometheus-format metrics at the `/metrics` endpoint for monitoring and observability.

## Endpoint

```
GET /metrics
```

**Response Format:** Prometheus text exposition format
**Content-Type:** `text/plain; version=0.0.4`
**Expected Response Time:** <10ms

## Available Metrics

### HTTP Request Metrics

#### `http_requests_total`
**Type:** Counter
**Description:** Total number of HTTP requests processed
**Labels:**
- `method`: HTTP method (GET, POST, PUT, DELETE, etc.)
- `path`: Request path (e.g., `/api/v1/content`)
- `status`: HTTP status code (200, 404, 500, etc.)

**Example:**
```prometheus
http_requests_total{method="GET",path="/api/v1/content",status="200"} 1234
http_requests_total{method="POST",path="/api/v1/search",status="201"} 567
http_requests_total{method="GET",path="/api/v1/content",status="404"} 12
```

#### `http_request_duration_seconds`
**Type:** Histogram
**Description:** HTTP request latency in seconds
**Labels:**
- `method`: HTTP method
- `path`: Request path

**Buckets:** 1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s, 5s

**Example:**
```prometheus
http_request_duration_seconds_bucket{method="GET",path="/api/v1/content",le="0.005"} 100
http_request_duration_seconds_bucket{method="GET",path="/api/v1/content",le="0.01"} 150
http_request_duration_seconds_sum{method="GET",path="/api/v1/content"} 12.34
http_request_duration_seconds_count{method="GET",path="/api/v1/content"} 200
```

### Connection Metrics

#### `active_connections`
**Type:** Gauge
**Description:** Number of active HTTP/WebSocket connections
**Labels:** None

**Example:**
```prometheus
active_connections 42
```

### Database Metrics

#### `db_connections_active`
**Type:** Gauge
**Description:** Number of active database connections in the pool
**Labels:** None

**Example:**
```prometheus
db_connections_active 5
```

#### `db_connections_idle`
**Type:** Gauge
**Description:** Number of idle database connections in the pool
**Labels:** None

**Example:**
```prometheus
db_connections_idle 15
```

### Cache Metrics

#### `cache_hits_total`
**Type:** Counter
**Description:** Total number of cache hits
**Labels:**
- `cache_type`: Type of cache (redis, memory, cdn)

**Example:**
```prometheus
cache_hits_total{cache_type="redis"} 10234
cache_hits_total{cache_type="memory"} 5678
```

#### `cache_misses_total`
**Type:** Counter
**Description:** Total number of cache misses
**Labels:**
- `cache_type`: Type of cache (redis, memory, cdn)

**Example:**
```prometheus
cache_misses_total{cache_type="redis"} 234
cache_misses_total{cache_type="memory"} 123
```

## Prometheus Scrape Configuration

Add this job to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'api-gateway'
    scrape_interval: 10s
    metrics_path: '/metrics'
    static_configs:
      - targets: ['api-gateway:8080']
        labels:
          service: 'api-gateway'
          component: 'gateway'
```

## Useful PromQL Queries

### Request Rate
```promql
# Requests per second
rate(http_requests_total{service="api-gateway"}[5m])

# By endpoint
sum(rate(http_requests_total{service="api-gateway"}[5m])) by (path)
```

### Latency Percentiles
```promql
# p50 latency
histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))

# p95 latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# p99 latency
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
```

### Error Rate
```promql
# 5xx error rate
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# 4xx error rate
rate(http_requests_total{status=~"4.."}[5m]) / rate(http_requests_total[5m])
```

### Cache Hit Rate
```promql
# Overall cache hit rate
sum(rate(cache_hits_total[5m])) / (sum(rate(cache_hits_total[5m])) + sum(rate(cache_misses_total[5m])))

# By cache type
sum(rate(cache_hits_total[5m])) by (cache_type) / (sum(rate(cache_hits_total[5m])) by (cache_type) + sum(rate(cache_misses_total[5m])) by (cache_type))
```

### Database Pool Utilization
```promql
# Connection pool utilization
db_connections_active / (db_connections_active + db_connections_idle)

# Total connections
db_connections_active + db_connections_idle
```

## Grafana Dashboard

A pre-configured Grafana dashboard is available at:
- **File:** `config/grafana/dashboards/api-gateway.json`
- **Access:** http://localhost:3000 (default credentials: admin/admin)

### Dashboard Panels

1. **HTTP Request Rate** - Requests per second by method, path, and status
2. **HTTP Request Latency Percentiles** - p50, p95, p99 latency by endpoint
3. **HTTP Error Rate** - 4xx and 5xx error rates
4. **Active Connections** - Current active HTTP/WebSocket connections
5. **Database Connection Pool** - Active vs idle database connections
6. **Cache Hit Rate** - Cache performance by cache type

## Alerting Rules

Recommended Prometheus alerting rules:

```yaml
groups:
  - name: api-gateway
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High 5xx error rate on API Gateway"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 5%)"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High p95 latency on API Gateway"
          description: "p95 latency is {{ $value }}s (threshold: 500ms)"

      - alert: DatabasePoolExhausted
        expr: db_connections_idle == 0
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Database connection pool exhausted"
          description: "No idle database connections available"

      - alert: LowCacheHitRate
        expr: sum(rate(cache_hits_total[5m])) / (sum(rate(cache_hits_total[5m])) + sum(rate(cache_misses_total[5m]))) < 0.7
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Low cache hit rate"
          description: "Cache hit rate is {{ $value | humanizePercentage }} (threshold: 70%)"
```

## Docker Compose Setup

The metrics stack is configured in `docker-compose.yml`:

```yaml
# Prometheus - scrapes metrics from all services
prometheus:
  image: prom/prometheus:v2.48.0
  ports:
    - "9090:9090"
  volumes:
    - ./config/prometheus.yml:/etc/prometheus/prometheus.yml:ro

# Grafana - visualizes metrics
grafana:
  image: grafana/grafana:10.2.2
  ports:
    - "3000:3000"
  environment:
    GF_SECURITY_ADMIN_PASSWORD: admin
```

### Starting the Metrics Stack

```bash
# Start all services including Prometheus and Grafana
docker-compose up -d

# Verify metrics endpoint
curl http://localhost:8080/metrics

# Access Prometheus UI
open http://localhost:9090

# Access Grafana UI
open http://localhost:3000
```

## Security Considerations

### Current Implementation
- Metrics endpoint is publicly accessible
- No authentication required
- Metrics contain service-level data only (no PII)

### Production Recommendations
1. **Basic Authentication:** Add HTTP basic auth to `/metrics` endpoint
2. **IP Allowlisting:** Restrict access to Prometheus server IPs only
3. **mTLS:** Use mutual TLS for production environments
4. **Service Mesh:** Leverage Istio/Linkerd for automatic mTLS

### Example: Adding Basic Auth

```rust
use actix_web::{HttpRequest, HttpResponse, dev::ServiceRequest};
use actix_web_httpauth::extractors::basic::BasicAuth;

async fn metrics_handler_with_auth(req: HttpRequest, auth: BasicAuth) -> HttpResponse {
    // Validate credentials
    if auth.user_id() != "prometheus" || auth.password().unwrap_or("") != "secret" {
        return HttpResponse::Unauthorized()
            .header("WWW-Authenticate", "Basic realm=\"Metrics\"")
            .finish();
    }

    // Return metrics
    media_gateway_core::metrics_handler().await
}
```

## Performance Characteristics

- **Endpoint Latency:** <10ms (in-memory aggregation)
- **Memory Overhead:** ~1-2MB per service
- **CPU Impact:** <1% (async collection)
- **Scrape Frequency:** 10-15 seconds recommended
- **Data Retention:** 30 days (configurable in Prometheus)

## Troubleshooting

### Metrics Not Appearing
1. Check service is running: `curl http://localhost:8080/health`
2. Verify metrics endpoint: `curl http://localhost:8080/metrics`
3. Check Prometheus targets: http://localhost:9090/targets
4. Review Prometheus logs: `docker logs mg-prometheus`

### Missing Labels
- Ensure `MetricsMiddleware` is registered in Actix-web
- Verify middleware is called before routes
- Check tracing logs for metric recording calls

### High Cardinality Warning
- Avoid high-cardinality labels (user IDs, session tokens)
- Group similar paths (use `/api/v1/content/{id}` not `/api/v1/content/123`)
- Limit status codes to standard HTTP codes

## References

- [Prometheus Exposition Formats](https://prometheus.io/docs/instrumenting/exposition_formats/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Grafana Dashboard Documentation](https://grafana.com/docs/grafana/latest/dashboards/)
