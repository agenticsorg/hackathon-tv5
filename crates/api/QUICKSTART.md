# API Gateway Quick Start Guide

## Setup

### Prerequisites
- Rust 1.75+
- Redis server
- Internal services running (Discovery, SONA, Sync, Auth)

### Installation

```bash
cd /workspaces/media-gateway/crates/api

# Copy environment template
cp .env.example .env

# Edit configuration
nano .env

# Build
cargo build --release

# Run
cargo run --release
```

### Environment Variables

```bash
# Minimal required configuration
API_GATEWAY_PORT=8080
REDIS_URL=redis://localhost:6379
DISCOVERY_SERVICE_URL=http://localhost:8081
SONA_SERVICE_URL=http://localhost:8082
SYNC_SERVICE_URL=http://localhost:8083
AUTH_SERVICE_URL=http://localhost:8084
JWT_SECRET=your-secret-key-here
```

## Docker Quick Start

```bash
# Build
docker build -t media-gateway-api .

# Run
docker run -d \
  --name api-gateway \
  -p 8080:8080 \
  -e REDIS_URL=redis://redis:6379 \
  -e DISCOVERY_SERVICE_URL=http://discovery:8081 \
  -e SONA_SERVICE_URL=http://sona:8082 \
  -e SYNC_SERVICE_URL=http://sync:8083 \
  -e AUTH_SERVICE_URL=http://auth:8084 \
  -e JWT_SECRET=your-secret \
  media-gateway-api

# Check logs
docker logs -f api-gateway

# Check health
curl http://localhost:8080/health
```

## Testing the API

### Health Check
```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 123,
  "checks": {
    "discovery": {
      "status": "healthy",
      "circuit_breaker": "closed"
    },
    "sona": {
      "status": "healthy",
      "circuit_breaker": "closed"
    }
  }
}
```

### Anonymous Request (Rate Limited: 5 req/s)
```bash
curl -i http://localhost:8080/api/v1/content/trending
```

Response headers:
```
X-Request-ID: 550e8400-e29b-41d4-a716-446655440000
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1733475600
```

### Authenticated Request
```bash
# Get JWT token from auth service first
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# Make authenticated request
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/v1/user/profile
```

### Search Content
```bash
curl -X POST http://localhost:8080/api/v1/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "inception",
    "filters": {
      "type": "movie",
      "year": 2010
    }
  }'
```

## Common Operations

### Check Rate Limits
```bash
# Make multiple requests to see rate limiting
for i in {1..10}; do
  curl -i http://localhost:8080/api/v1/content/trending
  sleep 0.1
done
```

### Monitor Health
```bash
# Continuous health monitoring
watch -n 5 'curl -s http://localhost:8080/health | jq'
```

### Test Circuit Breaker
```bash
# Stop a backend service to trigger circuit breaker
# The gateway will return 503 with circuit breaker message

curl http://localhost:8080/api/v1/content/123
# Expected: 503 Service Unavailable if service is down
```

## API Endpoints Reference

### Public Endpoints (No Auth Required)

```bash
# Content
GET  /api/v1/content/{id}
GET  /api/v1/content/{id}/availability
GET  /api/v1/content/trending
GET  /api/v1/movies/popular
GET  /api/v1/tv/popular

# Search
POST /api/v1/search
POST /api/v1/search/semantic
GET  /api/v1/search/autocomplete?q=term

# Discovery
GET  /api/v1/discover/movies?genre=action&year=2024
GET  /api/v1/discover/tv?genre=drama
GET  /api/v1/genres

# Health
GET  /health
GET  /health/ready
GET  /health/live
```

### Protected Endpoints (Auth Required)

```bash
# User Profile
GET  /api/v1/user/profile
PUT  /api/v1/user/preferences

# Watchlist
GET    /api/v1/user/watchlist
POST   /api/v1/user/watchlist
DELETE /api/v1/user/watchlist/{id}

# History
GET  /api/v1/user/history
```

## Rate Limit Tiers

| Tier       | Auth Required | Requests/Second | Requests/Minute |
|------------|---------------|-----------------|-----------------|
| Anonymous  | No            | 5               | 100             |
| Free       | Yes           | 10              | 200             |
| Pro        | Yes           | 50              | 1,000           |
| Enterprise | Yes           | 200             | 5,000           |

## Error Responses

### Rate Limited (429)
```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many requests",
    "retry_after": 60
  }
}
```

### Unauthorized (401)
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

### Service Unavailable (503)
```json
{
  "error": {
    "code": "CIRCUIT_BREAKER_OPEN",
    "message": "Circuit breaker open for service: discovery"
  }
}
```

## Monitoring

### View Logs
```bash
# JSON formatted logs
tail -f /var/log/api-gateway.log | jq

# Filter by level
tail -f /var/log/api-gateway.log | jq 'select(.level == "ERROR")'

# Filter by user
tail -f /var/log/api-gateway.log | jq 'select(.user_id == "user123")'
```

### Check Circuit Breaker Status
```bash
curl http://localhost:8080/health | jq '.checks[].circuit_breaker'
```

### Monitor Rate Limits
```bash
# Watch rate limit headers
curl -i http://localhost:8080/api/v1/content/trending | grep X-RateLimit
```

## Troubleshooting

### Gateway Won't Start

```bash
# Check Redis connection
redis-cli ping

# Check port availability
lsof -i :8080

# Check environment variables
env | grep -E "API_GATEWAY|REDIS|SERVICE"
```

### High Latency

```bash
# Check service health
curl http://localhost:8080/health

# Check circuit breaker state
curl http://localhost:8080/health | jq '.checks'

# Check logs for slow requests
tail -f /var/log/api-gateway.log | jq 'select(.duration_ms > 100)'
```

### Rate Limiting Issues

```bash
# Check Redis
redis-cli keys "ratelimit:*"

# Check rate limit configuration
curl http://localhost:8080/health -H "Authorization: Bearer $TOKEN" -I | grep X-RateLimit
```

### Circuit Breaker Stuck Open

```bash
# Check downstream service health
curl http://localhost:8081/health  # Discovery
curl http://localhost:8082/health  # SONA
curl http://localhost:8083/health  # Sync
curl http://localhost:8084/health  # Auth

# Wait for half-open timeout (2-3 seconds)
# Circuit breaker will automatically test recovery
```

## Development

### Run Tests
```bash
cargo test
```

### Run with Debug Logging
```bash
RUST_LOG=debug cargo run
```

### Format Code
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

### Build Release Binary
```bash
cargo build --release
./target/release/media-gateway-api
```

## Performance Testing

### Load Test with Apache Bench
```bash
# 10,000 requests, 100 concurrent
ab -n 10000 -c 100 http://localhost:8080/api/v1/content/trending
```

### Load Test with wrk
```bash
# 30 second test, 10 threads, 100 connections
wrk -t10 -c100 -d30s http://localhost:8080/api/v1/content/trending
```

### Stress Test Rate Limiting
```bash
# Rapid requests to trigger rate limit
seq 1 1000 | xargs -P 100 -I {} curl -s http://localhost:8080/api/v1/content/trending > /dev/null
```

## Configuration Tuning

### High Throughput Setup
```bash
# Increase workers and connections
API_GATEWAY_WORKERS=16
API_GATEWAY_MAX_CONNECTIONS=50000
```

### Strict Rate Limiting
```bash
# Adjust in config.rs or via environment
# Lower limits for tighter control
```

### Circuit Breaker Sensitivity
```bash
# Adjust thresholds in config.rs
# Lower failure_threshold for faster detection
# Higher timeout for slower recovery
```

## Next Steps

1. Set up monitoring (Prometheus + Grafana)
2. Configure log aggregation (ELK/Loki)
3. Set up alerts for circuit breaker events
4. Tune rate limits based on usage patterns
5. Add custom middleware as needed
6. Implement caching layer
7. Set up load balancer in front of gateway

## Support Files

- `README.md` - Comprehensive documentation
- `IMPLEMENTATION.md` - Implementation details
- `FILES.txt` - Complete file listing
- `.env.example` - Configuration template
- `verify.sh` - Automated verification
- `Dockerfile` - Production Docker image

## Resources

- Actix-Web: https://actix.rs
- Redis: https://redis.io
- Circuit Breaker Pattern: https://martinfowler.com/bliki/CircuitBreaker.html
- Rate Limiting: https://en.wikipedia.org/wiki/Rate_limiting

---

API Gateway v0.1.0 - Media Gateway Platform
