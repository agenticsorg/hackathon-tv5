# Media Gateway API

The API Gateway for the Media Gateway platform, providing unified access to all backend services with rate limiting, circuit breakers, and intelligent routing.

## Features

- **Service Routing**: Intelligent routing to Discovery, SONA, Sync, and Auth services
- **Rate Limiting**: Redis-backed sliding window rate limiting with multiple tiers
- **Circuit Breakers**: Per-service circuit breakers with configurable thresholds
- **Authentication**: JWT-based authentication middleware
- **Request Tracking**: Request ID and correlation ID injection
- **Health Checks**: Comprehensive health, readiness, and liveness probes
- **CORS Support**: Configurable CORS for cross-origin requests
- **Structured Logging**: JSON-formatted structured logging with tracing

## Architecture

```
┌─────────────┐
│   Clients   │
└──────┬──────┘
       │
       v
┌─────────────────────────────────────┐
│        API Gateway (Port 8080)      │
│  ┌─────────────────────────────┐   │
│  │  Middleware Stack           │   │
│  │  - Request ID               │   │
│  │  - Logging                  │   │
│  │  - CORS                     │   │
│  │  - Auth (optional/required) │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │  Rate Limiter (Redis)       │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │  Circuit Breaker Manager    │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │  Service Proxy              │   │
│  └─────────────────────────────┘   │
└────┬────┬────┬────┬───────────────┘
     │    │    │    │
     v    v    v    v
  ┌────┐┌────┐┌────┐┌────┐
  │Disc││SONA││Sync││Auth│
  │8081││8082││8083││8084│
  └────┘└────┘└────┘└────┘
```

## API Routes

### Content Routes (Discovery Service)
- `GET /api/v1/content/{id}` - Get content details
- `GET /api/v1/content/{id}/availability` - Get platform availability
- `GET /api/v1/content/trending` - Get trending content
- `GET /api/v1/movies/popular` - Get popular movies
- `GET /api/v1/tv/popular` - Get popular TV shows

### Search Routes (Discovery Service)
- `POST /api/v1/search` - Hybrid search
- `POST /api/v1/search/semantic` - Vector/semantic search
- `GET /api/v1/search/autocomplete` - Search autocomplete

### Discovery Routes (Discovery Service)
- `GET /api/v1/discover/movies` - Browse movies
- `GET /api/v1/discover/tv` - Browse TV shows
- `GET /api/v1/genres` - List genres

### User Routes (Auth + Sync Services)
- `GET /api/v1/user/profile` - User profile (Auth)
- `PUT /api/v1/user/preferences` - Update preferences (Auth)
- `GET /api/v1/user/watchlist` - Get watchlist (Sync)
- `POST /api/v1/user/watchlist` - Add to watchlist (Sync)
- `DELETE /api/v1/user/watchlist/{id}` - Remove from watchlist (Sync)
- `GET /api/v1/user/history` - Watch history (Sync)

### Health Routes
- `GET /health` - Overall health status
- `GET /health/ready` - Readiness probe
- `GET /health/live` - Liveness probe

## Rate Limiting

### Tiers

| Tier       | Requests/Second | Requests/Minute | Burst |
|------------|-----------------|-----------------|-------|
| Anonymous  | 5               | 100             | 10    |
| Free       | 10              | 200             | 20    |
| Pro        | 50              | 1000            | 100   |
| Enterprise | 200             | 5000            | 400   |

### Headers

All responses include rate limit headers:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1733475600
```

## Circuit Breakers

### Configuration

| Service   | Failure Threshold | Timeout | Error Rate |
|-----------|-------------------|---------|------------|
| Discovery | 20 requests       | 3s      | 50%        |
| SONA      | 10 requests       | 2s      | 40%        |
| Sync      | 10 requests       | 2s      | 50%        |
| Auth      | 10 requests       | 2s      | 50%        |

### States

- **Closed**: Normal operation, all requests pass through
- **Open**: Circuit is open, requests fail immediately
- **Half-Open**: Testing if service has recovered

## Performance Targets

From SPARC specifications:

- **p50 Latency**: 20ms
- **p95 Latency**: 80ms
- **p99 Latency**: 150ms
- **Throughput**: 5,000 RPS
- **Error Rate**: <0.1%

## Configuration

Configuration is loaded from environment variables. See `.env.example` for all options.

### Key Variables

```bash
API_GATEWAY_HOST=0.0.0.0
API_GATEWAY_PORT=8080
DISCOVERY_SERVICE_URL=http://localhost:8081
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
RUST_LOG=info,media_gateway_api=debug
```

## Building

```bash
# Build
cargo build --release

# Run
cargo run --release

# Test
cargo test

# With environment file
cp .env.example .env
# Edit .env with your configuration
cargo run --release
```

## Docker

```bash
# Build image
docker build -t media-gateway-api .

# Run with Docker
docker run -p 8080:8080 \
  -e REDIS_URL=redis://redis:6379 \
  -e DISCOVERY_SERVICE_URL=http://discovery:8081 \
  media-gateway-api
```

## Dependencies

- **actix-web**: HTTP server framework
- **tokio**: Async runtime
- **redis**: Rate limiting state
- **reqwest**: Service proxying
- **tower**: Middleware utilities
- **failsafe**: Circuit breaker implementation
- **jsonwebtoken**: JWT validation
- **tracing**: Structured logging

## Error Responses

All errors follow a consistent format:

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many requests",
    "retry_after": 60
  }
}
```

### Error Codes

- `RATE_LIMITED`: Rate limit exceeded
- `SERVICE_UNAVAILABLE`: Downstream service unavailable
- `CIRCUIT_BREAKER_OPEN`: Circuit breaker is open
- `AUTH_FAILED`: Authentication failed
- `UNAUTHORIZED`: Missing or invalid authorization
- `BAD_REQUEST`: Invalid request
- `NOT_FOUND`: Resource not found
- `INTERNAL_ERROR`: Internal server error
- `PROXY_ERROR`: Error proxying to service
- `TIMEOUT`: Request timeout

## Monitoring

The API Gateway emits structured JSON logs suitable for aggregation:

```json
{
  "timestamp": "2025-12-06T10:00:00Z",
  "level": "INFO",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "user123",
  "method": "GET",
  "path": "/api/v1/content/123",
  "status": 200,
  "duration_ms": 45
}
```

## License

Proprietary - Media Gateway Platform
