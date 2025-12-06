# BATCH_002: Service Initialization & API Gateway Integration

**Generated**: 2025-12-06
**Depends On**: BATCH_001 (completed)
**Priority**: HIGH
**Theme**: Service lifecycle, configuration management, API routing, and health monitoring

---

## TASK-013: Implement service configuration loading system

**Description**: Create a centralized configuration system using `config` crate with environment variable overrides, config file support, and validation for all microservices.

**Files**:
- `crates/core/src/config.rs` - Create centralized config module with ServiceConfig trait
- `crates/core/src/config/mod.rs` - Configuration loaders and validators
- `crates/core/src/config/env.rs` - Environment variable parsing and validation
- `crates/auth/src/config.rs` - Auth service configuration (OAuth, JWT, Redis)
- `crates/sona/src/config.rs` - SONA configuration (LoRA, embeddings, thresholds)
- `crates/playback/src/config.rs` - Playback configuration (deep links, platforms)

**Dependencies**: TASK-011 (shared math utilities completed)

**Acceptance Criteria**:
- [ ] ServiceConfig trait with load(), validate(), and to_env_vars() methods
- [ ] Support for config files (TOML) in `/config` directory
- [ ] Environment variable override with MEDIA_GATEWAY_ prefix
- [ ] Secrets loaded from environment only (never from files)
- [ ] Validation with detailed error messages on startup
- [ ] Unit tests with 90%+ coverage for config loading edge cases

---

## TASK-014: Build API Gateway request routing and middleware

**Description**: Implement comprehensive API Gateway with service discovery, rate limiting, auth validation, CORS, request ID propagation, and circuit breaker patterns.

**Files**:
- `crates/api/src/lib.rs` - API Gateway library exports
- `crates/api/src/router.rs` - Route definitions and service routing logic
- `crates/api/src/middleware/mod.rs` - Middleware chain coordinator
- `crates/api/src/middleware/rate_limit.rs` - Redis-backed rate limiting (governor crate)
- `crates/api/src/middleware/auth_validate.rs` - JWT validation middleware
- `crates/api/src/middleware/cors.rs` - CORS configuration per SPARC spec
- `crates/api/src/middleware/request_id.rs` - Request ID generation and propagation
- `crates/api/src/middleware/circuit_breaker.rs` - Circuit breaker for downstream services
- `crates/api/src/service_discovery.rs` - gRPC service discovery and health checking

**Dependencies**: TASK-013 (configuration system)

**Acceptance Criteria**:
- [ ] Routes for /api/v1/search, /api/v1/recommendations, /api/v1/content, /api/v1/auth
- [ ] Rate limiting: 60 req/min for free_user, 300 req/min for premium_user
- [ ] JWT validation using auth service configuration
- [ ] CORS with allowed origins from config
- [ ] X-Request-ID header generation and propagation
- [ ] Circuit breaker: 5 failures → 30s timeout, exponential backoff
- [ ] Integration tests with mock gRPC services
- [ ] OpenTelemetry tracing integration with trace_id propagation

---

## TASK-015: Create health check and readiness probe endpoints

**Description**: Implement comprehensive health checks for all services with dependency validation (database, Redis, Qdrant) and Kubernetes-compatible probes.

**Files**:
- `crates/core/src/health.rs` - Health check framework with HealthCheck trait
- `crates/auth/src/health.rs` - Auth service health (PostgreSQL, Redis connectivity)
- `crates/discovery/src/health.rs` - Discovery health (PostgreSQL, Qdrant, Redis)
- `crates/sona/src/health.rs` - SONA health (PostgreSQL, model loading status)
- `crates/sync/src/health.rs` - Sync health (PubNub, WebSocket connection pool)
- `crates/ingestion/src/health.rs` - Ingestion health (API quotas, queue depth)
- `crates/playback/src/health.rs` - Playback health (session storage, Redis)
- `crates/api/src/health.rs` - API Gateway health (downstream service checks)

**Dependencies**: TASK-013 (configuration), TASK-010 (num_cpus dependency)

**Acceptance Criteria**:
- [ ] GET /health - Liveness probe (returns 200 if service is running)
- [ ] GET /health/ready - Readiness probe (returns 200 if all dependencies healthy)
- [ ] GET /health/details - Detailed JSON with component statuses and versions
- [ ] Each component check has timeout (max 2s) to prevent blocking
- [ ] Health check responses include: status, timestamp, service_name, version, dependencies[]
- [ ] Dependencies checked: database (SELECT 1), Redis (PING), Qdrant (health API)
- [ ] Graceful degradation: mark degraded if non-critical dependency fails
- [ ] Unit tests for health check logic with mock dependencies

---

## TASK-016: Implement graceful shutdown handlers for all services

**Description**: Add graceful shutdown logic with connection draining, in-flight request completion, and resource cleanup for all microservices.

**Files**:
- `crates/core/src/shutdown.rs` - Shared shutdown coordinator with signal handling
- `crates/auth/src/server.rs` - Update with graceful shutdown (drain Redis connections)
- `crates/discovery/src/server.rs` - Update with graceful shutdown (close Qdrant client)
- `crates/sona/src/server.rs` - Update with graceful shutdown (persist user models)
- `crates/sync/src/server.rs` - Update with graceful shutdown (close WebSocket connections)
- `crates/ingestion/src/server.rs` - Create server with graceful shutdown (finish in-flight jobs)
- `crates/playback/src/server.rs` - Create server with graceful shutdown (close sessions)
- `crates/api/src/main.rs` - Update with graceful shutdown (complete pending requests)

**Dependencies**: TASK-013 (configuration for shutdown timeouts)

**Acceptance Criteria**:
- [ ] Handle SIGTERM and SIGINT signals
- [ ] Stop accepting new requests immediately on signal
- [ ] Wait up to 30 seconds for in-flight requests to complete
- [ ] Close database connection pools cleanly
- [ ] Close Redis connections and flush pending writes
- [ ] Close WebSocket connections with graceful disconnect messages
- [ ] Log shutdown progress at INFO level
- [ ] Exit with code 0 on clean shutdown, code 1 on forced shutdown
- [ ] Integration tests using tokio::signal::ctrl_c simulation

---

## TASK-017: Add OpenTelemetry tracing instrumentation

**Description**: Instrument all services with OpenTelemetry distributed tracing including span creation, trace propagation, and Cloud Trace export.

**Files**:
- `crates/core/src/tracing.rs` - OpenTelemetry setup and configuration
- `crates/core/src/tracing/propagation.rs` - W3C Trace Context propagation
- `crates/core/src/tracing/attributes.rs` - Common span attributes and conventions
- `crates/auth/src/server.rs` - Add tracing_actix_web middleware
- `crates/discovery/src/server.rs` - Add tracing_actix_web middleware
- `crates/sona/src/server.rs` - Add tracing_actix_web middleware
- `crates/sync/src/server.rs` - Add tracing instrumentation
- `crates/api/src/middleware/tracing.rs` - Request tracing middleware with sampling

**Dependencies**: TASK-014 (API Gateway middleware system)

**Acceptance Criteria**:
- [ ] OpenTelemetry SDK with Cloud Trace exporter configured
- [ ] W3C Trace Context propagation via traceparent header
- [ ] Automatic span creation for HTTP requests with status, method, path attributes
- [ ] Manual span creation for critical operations (database queries, external API calls)
- [ ] Sampling: 10% for successful requests, 100% for errors
- [ ] Trace IDs logged in structured logs for correlation
- [ ] gRPC trace propagation for inter-service calls
- [ ] Integration test validating end-to-end trace creation

---

## TASK-018: Create playback service with session management

**Description**: Implement the playback service with session CRUD, deep link generation, platform routing, and DRM token handling.

**Files**:
- `crates/playback/src/lib.rs` - Playback service library exports
- `crates/playback/src/types.rs` - PlaybackSession, Platform, DRMInfo types
- `crates/playback/src/deep_link.rs` - Platform-specific deep link generation
- `crates/playback/src/server.rs` - Actix-web server with session endpoints
- `crates/playback/src/routes.rs` - REST endpoints for playback operations
- `crates/playback/src/redis_storage.rs` - Redis-backed session storage
- `crates/playback/src/drm.rs` - DRM token generation and validation stubs
- `crates/playback/src/platform_router.rs` - Platform capability matching
- `crates/playback/tests/integration_test.rs` - Integration tests with Redis

**Dependencies**: TASK-009 (playback session management), TASK-012 (Docker Compose)

**Acceptance Criteria**:
- [ ] POST /api/v1/playback - Initiate playback session with content_id, platform, device_id
- [ ] GET /api/v1/playback/{session_id} - Retrieve session details
- [ ] PATCH /api/v1/playback/{session_id}/progress - Update playback position
- [ ] DELETE /api/v1/playback/{session_id} - End session
- [ ] Deep link generation for Netflix, Spotify, Apple Music, Hulu, Disney+, Prime Video
- [ ] Redis storage with TTL (24 hours for active, 7 days for paused sessions)
- [ ] Platform capability matching (4K, HDR, Dolby Atmos support)
- [ ] DRM token generation stubs (return mock tokens for now)
- [ ] Unit tests with 90%+ coverage, integration tests with real Redis

---

## TASK-019: Build ingestion service scheduler and job management

**Description**: Create the ingestion service with job scheduler, retry logic, and monitoring for periodic platform catalog refreshes.

**Files**:
- `crates/ingestion/src/lib.rs` - Update with server exports
- `crates/ingestion/src/scheduler.rs` - Job scheduler using tokio::time intervals
- `crates/ingestion/src/job.rs` - Job definition, status, retry logic
- `crates/ingestion/src/worker.rs` - Worker pool using num_cpus for parallelism
- `crates/ingestion/src/server.rs` - Create HTTP server for job management
- `crates/ingestion/src/routes.rs` - REST endpoints for job control and status
- `crates/ingestion/src/queue.rs` - In-memory job queue with priority ordering
- `crates/ingestion/src/metrics.rs` - Job metrics (success rate, latency, queue depth)
- `crates/ingestion/tests/scheduler_test.rs` - Scheduler unit tests

**Dependencies**: TASK-004 (ingestion pipeline persistence), TASK-010 (num_cpus), TASK-013 (config)

**Acceptance Criteria**:
- [ ] Scheduler with configurable intervals: catalog refresh (6h), availability sync (1h)
- [ ] Worker pool sized by num_cpus (default: num_cpus * 2)
- [ ] Retry logic: exponential backoff (1s, 2s, 4s, 8s, 16s), max 5 attempts
- [ ] Job queue with priority levels: CRITICAL, HIGH, NORMAL, LOW
- [ ] POST /api/v1/ingestion/jobs - Manually trigger job
- [ ] GET /api/v1/ingestion/jobs - List jobs with status, progress
- [ ] GET /api/v1/ingestion/metrics - Job metrics and queue depth
- [ ] Integration with TASK-004 pipeline for actual ingestion execution
- [ ] Unit tests for scheduler, retry logic, queue management

---

## TASK-020: Implement SONA service initialization with model loading

**Description**: Create the SONA recommendation service with base model loading, LoRA adapter management, and inference endpoints.

**Files**:
- `crates/sona/src/server.rs` - Create Actix-web server for SONA service
- `crates/sona/src/routes.rs` - REST endpoints for recommendations
- `crates/sona/src/model_loader.rs` - Base model and LoRA loading from filesystem
- `crates/sona/src/inference.rs` - SONA inference logic using pre-computed embeddings
- `crates/sona/src/cache.rs` - Redis caching for user profiles and recommendations
- `crates/sona/src/types.rs` - Update with RecommendationRequest, RecommendationResponse
- `crates/sona/tests/integration_test.rs` - Integration tests with mock models

**Dependencies**: TASK-002 (SONA collaborative filtering), TASK-003 (content-based filtering), TASK-013 (config)

**Acceptance Criteria**:
- [ ] POST /api/v1/recommendations - Get personalized recommendations
- [ ] GET /api/v1/recommendations/{user_id}/profile - Get user embedding profile
- [ ] Base model stub loading (log "Model loaded" without actual PyTorch for now)
- [ ] LoRA adapter stub loading per user from /models/lora/{user_id}.safetensors
- [ ] Inference using collaborative + content-based filtering (TASK-002, TASK-003 logic)
- [ ] Redis caching: user profiles (1h TTL), recommendations (15min TTL)
- [ ] Graceful degradation: return content-based only if LoRA fails to load
- [ ] Latency target: <100ms p95 for recommendation generation
- [ ] Unit tests for model loader, cache, inference pipeline

---

## TASK-021: Add metrics collection and Prometheus export

**Description**: Implement Prometheus metrics export for all services with RED metrics (Rate, Errors, Duration) and custom business metrics.

**Files**:
- `crates/core/src/metrics.rs` - Metrics framework with Prometheus registry
- `crates/core/src/metrics/http.rs` - HTTP metrics (request count, latency, errors)
- `crates/core/src/metrics/database.rs` - Database metrics (connection pool, query latency)
- `crates/auth/src/metrics.rs` - Auth-specific metrics (login rate, token generation)
- `crates/discovery/src/metrics.rs` - Search metrics (query latency, cache hit rate)
- `crates/sona/src/metrics.rs` - Recommendation metrics (inference latency, cache hits)
- `crates/sync/src/metrics.rs` - Sync metrics (message rate, WebSocket connections)
- `crates/api/src/metrics.rs` - Gateway metrics (request rate, downstream latency)

**Dependencies**: TASK-014 (API Gateway), TASK-015 (health checks)

**Acceptance Criteria**:
- [ ] GET /metrics - Prometheus text format export on all services
- [ ] RED metrics: http_requests_total{method, status, path}, http_request_duration_seconds
- [ ] Database metrics: db_connections_active, db_connections_idle, db_query_duration_seconds
- [ ] Custom metrics: auth_logins_total, search_queries_total, recommendations_generated_total
- [ ] Cache metrics: cache_hits_total, cache_misses_total by cache_type label
- [ ] Histogram buckets: [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
- [ ] Labels follow Prometheus naming conventions (snake_case, service, environment)
- [ ] Integration test validating metric values after requests

---

## TASK-022: Create Docker Compose orchestration for full stack

**Description**: Expand the existing Docker Compose configuration to include all microservices with proper networking, dependencies, and health checks.

**Files**:
- `docker-compose.yml` - Update with all 8 microservices
- `docker-compose.dev.yml` - Development override with hot reload
- `docker-compose.prod.yml` - Production override with resource limits
- `Dockerfile` - Multi-stage build for Rust services
- `Dockerfile.api` - Dockerfile for API Gateway
- `.dockerignore` - Exclude target/, .git/, *.md
- `scripts/docker-build.sh` - Build script for all services
- `scripts/docker-health-check.sh` - Health check wrapper for Docker

**Dependencies**: All service implementation tasks (TASK-014 through TASK-020)

**Acceptance Criteria**:
- [ ] Services: api-gateway, auth-service, discovery-service, sona-service, sync-service, ingestion-service, playback-service
- [ ] Infrastructure: PostgreSQL 16, Redis 7, Qdrant latest
- [ ] Networks: frontend (API gateway), backend (inter-service), data (databases)
- [ ] Health checks: HTTP GET /health for all services, retry 3 times, interval 10s
- [ ] Dependency ordering: databases → services → API gateway
- [ ] Environment variable injection from .env file
- [ ] Volume mounts: ./data/postgres, ./data/redis, ./data/qdrant, ./config
- [ ] Port mapping: 8080 (API), 8081 (discovery), 8082 (SONA), 8083 (sync), 8084 (auth), 8085 (ingestion), 8086 (playback)
- [ ] Resource limits (prod): cpu: 1.0, memory: 1Gi per service
- [ ] Multi-stage Dockerfile: builder (cargo build --release) → runtime (debian:bookworm-slim)
- [ ] docker-compose up starts full stack, docker-compose down removes all resources

---

## TASK-023: Build authentication flow integration tests

**Description**: Create comprehensive integration tests for OAuth 2.0 + PKCE, device authorization grant, JWT lifecycle, and token refresh flows.

**Files**:
- `crates/auth/tests/integration/mod.rs` - Integration test suite coordinator
- `crates/auth/tests/integration/oauth_pkce_test.rs` - OAuth PKCE flow end-to-end tests
- `crates/auth/tests/integration/device_flow_test.rs` - Device authorization grant tests
- `crates/auth/tests/integration/jwt_lifecycle_test.rs` - Token generation, validation, refresh
- `crates/auth/tests/integration/token_revocation_test.rs` - Token revocation and blacklist
- `crates/auth/tests/integration/rbac_test.rs` - Role-based access control validation
- `crates/auth/tests/fixtures/mod.rs` - Test fixtures (mock OAuth provider, test users)
- `crates/auth/tests/helpers.rs` - Test helpers for HTTP requests and assertions

**Dependencies**: TASK-006 (auth storage Redis migration), TASK-013 (configuration), TASK-012 (Docker Compose)

**Acceptance Criteria**:
- [ ] OAuth PKCE flow: authorization code generation, code challenge validation, token exchange
- [ ] Device flow: device code generation, user approval, polling, token grant
- [ ] JWT lifecycle: access token generation, validation, expiry, refresh token exchange
- [ ] Token revocation: logout revokes access + refresh, blacklist checked on validation
- [ ] RBAC: verify permission checks for free_user, premium_user, admin roles
- [ ] Integration tests use real Redis (via Docker Compose testcontainers)
- [ ] Tests clean up state after execution (delete test users, revoke tokens)
- [ ] Minimum 20 integration tests covering happy path + error cases
- [ ] Tests run in <30 seconds total with parallel execution

---

## TASK-024: Implement service-to-service authentication with mTLS stubs

**Description**: Create mTLS authentication stubs for inter-service communication with certificate validation and identity propagation.

**Files**:
- `crates/core/src/mtls.rs` - mTLS client and server configuration
- `crates/core/src/mtls/identity.rs` - Service identity extraction from certificates
- `crates/core/src/mtls/validator.rs` - Certificate validation logic (stub for now)
- `crates/api/src/grpc_client.rs` - gRPC client with mTLS configuration
- `crates/auth/src/grpc_server.rs` - gRPC server with mTLS listener
- `crates/discovery/src/grpc_server.rs` - gRPC server with mTLS listener
- `crates/sona/src/grpc_server.rs` - gRPC server with mTLS listener
- `scripts/generate-test-certs.sh` - Generate self-signed certificates for testing

**Dependencies**: TASK-014 (API Gateway service discovery)

**Acceptance Criteria**:
- [ ] TLS 1.3 with client certificate authentication required
- [ ] Certificate validation: check CN matches service name (e.g., CN=discovery-service)
- [ ] Service identity extracted from certificate and logged
- [ ] gRPC client configuration with client certificate and CA bundle
- [ ] gRPC server configuration with server certificate and client CA verification
- [ ] Test certificates generated with generate-test-certs.sh (valid for 365 days)
- [ ] Stub certificate validation (always succeeds for now, real CA validation in BATCH_003)
- [ ] Integration test with two services communicating via mTLS gRPC
- [ ] Graceful error handling: log certificate errors, return Unauthenticated gRPC status

---

## TASK-025: Add structured logging with correlation IDs

**Description**: Implement structured JSON logging with correlation ID propagation, log levels, and contextual metadata for all services.

**Files**:
- `crates/core/src/logging.rs` - Logging configuration and structured log macros
- `crates/core/src/logging/correlation.rs` - Correlation ID generation and propagation
- `crates/core/src/logging/context.rs` - Log context builder with service metadata
- `crates/auth/src/server.rs` - Update with structured logging
- `crates/discovery/src/server.rs` - Update with structured logging
- `crates/sona/src/server.rs` - Update with structured logging
- `crates/sync/src/server.rs` - Update with structured logging
- `crates/api/src/middleware/logging.rs` - Request logging middleware

**Dependencies**: TASK-014 (API Gateway middleware), TASK-017 (tracing)

**Acceptance Criteria**:
- [ ] JSON log format with fields: timestamp, level, service, message, correlation_id, trace_id
- [ ] Correlation ID generated per request in API Gateway, propagated via X-Correlation-ID header
- [ ] Log levels: ERROR (always), WARN (always), INFO (default), DEBUG (dev only), TRACE (never in prod)
- [ ] Contextual metadata: user_id, request_method, request_path, response_status, latency_ms
- [ ] Sensitive data masking: redact passwords, tokens, API keys in logs
- [ ] Log aggregation-friendly format (single-line JSON, no multi-line stack traces inline)
- [ ] Request start/end logs with timing information
- [ ] Error logs include error type, error message, stack trace (truncated to 1000 chars)
- [ ] Integration test validates JSON structure and correlation ID propagation

---

## Summary

BATCH_002 establishes the foundational service infrastructure:

| Category | Tasks | Key Deliverables |
|----------|-------|------------------|
| **Configuration & Lifecycle** | 013, 015, 016 | Config management, health checks, graceful shutdown |
| **API Gateway** | 014, 021, 025 | Request routing, metrics, logging |
| **Services** | 018, 019, 020 | Playback, Ingestion, SONA services |
| **Observability** | 017, 021, 025 | Tracing, metrics, structured logging |
| **Security** | 023, 024 | Auth integration tests, mTLS stubs |
| **DevOps** | 022 | Docker Compose orchestration |

**Total Tasks**: 13
**Estimated Total Time**: 40-60 hours (3-4 per task)
**Risk**: LOW (no external API dependencies, incremental build on BATCH_001)

**Next Batch Focus**: External platform integration, MCP server implementation, real-time sync endpoints
