# BATCH_006: Media Gateway Action List

**Generated**: 2025-12-06T00:00:00Z
**Analysis Method**: 9-Agent Claude-Flow Swarm Analysis
**Previous Batches**: BATCH_001 through BATCH_005 Completed
**Focus**: Advanced Features, Security Hardening, and Production Readiness

---

## Executive Summary

Following comprehensive analysis of the repository after BATCH_001-005 implementation, this batch focuses on:
1. **Security Hardening** - MFA, additional OAuth providers, API key management
2. **Advanced Recommendation Features** - Collaborative filtering, real-time embeddings
3. **Platform Integration** - Webhook systems, continue watching APIs
4. **Observability** - Distributed tracing, metrics endpoints
5. **Cross-Service Integration** - Health aggregation, circuit breakers

---

## Task List

### TASK-001: Implement Multi-Factor Authentication (MFA) System
**Priority**: P0-Critical
**Complexity**: High
**Estimated LOC**: 400-500
**Crate**: `auth`

**Description**:
Implement TOTP-based multi-factor authentication for enhanced account security. The system should support enrollment, verification, backup codes, and recovery flows.

**Acceptance Criteria**:
- [ ] `MfaManager` struct with TOTP generation using RFC 6238
- [ ] `POST /api/v1/auth/mfa/enroll` - Initiate MFA enrollment with QR code
- [ ] `POST /api/v1/auth/mfa/verify` - Verify TOTP code during enrollment
- [ ] `POST /api/v1/auth/mfa/challenge` - Challenge user during login
- [ ] Backup code generation (10 single-use codes per user)
- [ ] `mfa_enrollments` PostgreSQL table with encrypted secrets
- [ ] Redis-backed rate limiting (5 attempts per minute)
- [ ] Integration tests with time-based mock

**Files to Create/Modify**:
- `crates/auth/src/mfa/mod.rs` (new)
- `crates/auth/src/mfa/totp.rs` (new)
- `crates/auth/src/mfa/backup_codes.rs` (new)
- `crates/auth/src/handlers.rs` (modify)
- `crates/auth/src/lib.rs` (modify)

**Dependencies**: BATCH_005 Token Family implementation

---

### TASK-002: GitHub OAuth Provider Implementation
**Priority**: P0-Critical
**Complexity**: Medium
**Estimated LOC**: 250-300
**Crate**: `auth`

**Description**:
Add GitHub as an OAuth 2.0 identity provider alongside existing Netflix and Google providers, enabling developer-focused user authentication.

**Acceptance Criteria**:
- [ ] `GitHubOAuthProvider` implementing `OAuthProvider` trait
- [ ] Authorization URL generation with `user:email`, `read:user` scopes
- [ ] Token exchange via GitHub's OAuth endpoint
- [ ] User profile fetching from `https://api.github.com/user`
- [ ] Email verification via `https://api.github.com/user/emails`
- [ ] Account linking with existing users by email
- [ ] Unit tests with mocked HTTP responses

**Files to Create/Modify**:
- `crates/auth/src/oauth/providers/github.rs` (new)
- `crates/auth/src/oauth/providers/mod.rs` (modify)
- `crates/auth/src/handlers.rs` (modify)

**Dependencies**: BATCH_005 Google OAuth pattern

---

### TASK-003: Real-Time Collaborative Filtering Pipeline
**Priority**: P1-High
**Complexity**: High
**Estimated LOC**: 450-550
**Crate**: `sona`

**Description**:
Implement true collaborative filtering using user-item interaction matrices and approximate nearest neighbor search, replacing the current placeholder implementation.

**Acceptance Criteria**:
- [ ] `CollaborativeFilteringEngine` with user-item matrix construction
- [ ] Implicit feedback collection (views, completions, ratings)
- [ ] ALS (Alternating Least Squares) factorization for embeddings
- [ ] User similarity computation using cosine similarity
- [ ] Item-item similarity for "users who watched X also watched Y"
- [ ] Qdrant vector storage for user/item embeddings
- [ ] Incremental update support (new interactions → embedding updates)
- [ ] Integration with `GenerateRecommendations` pipeline

**Files to Create/Modify**:
- `crates/sona/src/collaborative.rs` (significant rewrite)
- `crates/sona/src/matrix_factorization.rs` (new)
- `crates/sona/src/recommendation.rs` (modify)

**Dependencies**: Qdrant vector database (deployed in docker-compose)

---

### TASK-004: Platform Webhook Integration System
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 350-400
**Crate**: `ingestion`

**Description**:
Implement a webhook receiver system that allows streaming platforms to push catalog updates in real-time, reducing polling overhead and improving data freshness.

**Acceptance Criteria**:
- [ ] `WebhookReceiver` trait for platform-specific handlers
- [ ] `POST /api/v1/webhooks/{platform}` endpoint with HMAC verification
- [ ] Webhook payload validation and normalization
- [ ] Async queue processing with Redis Streams
- [ ] Dead letter queue for failed webhook processing
- [ ] Webhook registration API for platforms
- [ ] Event deduplication using content hash
- [ ] Metrics: webhooks received, processed, failed

**Files to Create/Modify**:
- `crates/ingestion/src/webhooks/mod.rs` (new)
- `crates/ingestion/src/webhooks/receiver.rs` (new)
- `crates/ingestion/src/webhooks/handlers/` (new directory)
- `crates/ingestion/src/lib.rs` (modify)

**Dependencies**: Redis Streams configuration

---

### TASK-005: Continue Watching API Implementation
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 300-350
**Crate**: `playback`

**Description**:
Implement a comprehensive "Continue Watching" API that tracks user progress, aggregates across devices, and provides personalized resume points.

**Acceptance Criteria**:
- [ ] `ContinueWatchingService` with progress tracking
- [ ] `GET /api/v1/playback/continue-watching` - List in-progress content
- [ ] `POST /api/v1/playback/progress` - Update playback progress
- [ ] Progress persistence to PostgreSQL with conflict resolution
- [ ] Cross-device sync via Sync service integration
- [ ] TTL-based cleanup for stale progress (>30 days)
- [ ] Completion threshold detection (95% = completed)
- [ ] Response includes: content metadata, progress %, last watched time

**Files to Create/Modify**:
- `crates/playback/src/continue_watching.rs` (new)
- `crates/playback/src/progress.rs` (new)
- `crates/playback/src/server.rs` (modify)
- `crates/playback/src/lib.rs` (modify)

**Dependencies**: Sync service CRDT implementation

---

### TASK-006: Distributed Tracing with OpenTelemetry
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 300-350
**Crate**: `core` (shared), all services

**Description**:
Implement distributed tracing across all services using OpenTelemetry, enabling end-to-end request visibility and performance analysis.

**Acceptance Criteria**:
- [ ] `TracingConfig` in core crate with OTLP exporter
- [ ] Trace context propagation via HTTP headers (`traceparent`)
- [ ] Span creation for all HTTP handlers (automatic via middleware)
- [ ] Database query spans with `sqlx` instrumentation
- [ ] Redis operation spans
- [ ] External API call spans (platform APIs, PubNub)
- [ ] Service-to-service trace correlation
- [ ] Jaeger/Zipkin exporter configuration

**Files to Create/Modify**:
- `crates/core/src/telemetry/mod.rs` (new)
- `crates/core/src/telemetry/tracing.rs` (new)
- `crates/core/src/telemetry/middleware.rs` (new)
- All service `main.rs` files (modify)
- `docker-compose.yml` (add Jaeger service)

**Dependencies**: OpenTelemetry Rust SDK

---

### TASK-007: API Key Management System
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 350-400
**Crate**: `auth`

**Description**:
Implement API key authentication for machine-to-machine communication and third-party integrations, complementing OAuth-based user authentication.

**Acceptance Criteria**:
- [ ] `ApiKeyManager` with secure key generation (256-bit random)
- [ ] `POST /api/v1/auth/api-keys` - Create API key with scopes
- [ ] `GET /api/v1/auth/api-keys` - List user's API keys (masked)
- [ ] `DELETE /api/v1/auth/api-keys/{key_id}` - Revoke key
- [ ] Key hash storage (SHA-256, never store plaintext)
- [ ] Scope-based authorization (read, write, admin)
- [ ] Rate limiting per API key
- [ ] Last used timestamp tracking
- [ ] Key expiration support (optional)

**Files to Create/Modify**:
- `crates/auth/src/api_keys/mod.rs` (new)
- `crates/auth/src/api_keys/manager.rs` (new)
- `crates/auth/src/middleware.rs` (modify)
- `crates/auth/src/handlers.rs` (modify)

**Dependencies**: None (builds on existing auth infrastructure)

---

### TASK-008: Search Analytics and Query Insights
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 300-350
**Crate**: `discovery`

**Description**:
Implement search analytics to track query patterns, popular searches, zero-result queries, and search performance metrics.

**Acceptance Criteria**:
- [ ] `SearchAnalytics` service with event tracking
- [ ] Query logging with anonymized user context
- [ ] Popular searches aggregation (hourly, daily, weekly)
- [ ] Zero-result query tracking for content gap analysis
- [ ] Search latency percentiles (p50, p95, p99)
- [ ] Click-through rate tracking
- [ ] `GET /api/v1/admin/search/analytics` - Dashboard data
- [ ] PostgreSQL storage with time-series optimization

**Files to Create/Modify**:
- `crates/discovery/src/analytics/mod.rs` (new)
- `crates/discovery/src/analytics/search_analytics.rs` (new)
- `crates/discovery/src/analytics/query_log.rs` (new)
- `crates/discovery/src/search/mod.rs` (modify)

**Dependencies**: None

---

### TASK-009: Circuit Breaker for External Services
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 250-300
**Crate**: `core` (shared)

**Description**:
Implement a circuit breaker pattern for all external service calls (streaming platform APIs, PubNub, external embedding services) to improve resilience.

**Acceptance Criteria**:
- [ ] `CircuitBreaker` struct with states: Closed, Open, Half-Open
- [ ] Configurable thresholds: failure count, timeout, recovery time
- [ ] Automatic state transitions based on success/failure
- [ ] Fallback mechanism support (cached data, default responses)
- [ ] Per-service circuit breaker instances
- [ ] Metrics: circuit state, failure rate, recovery attempts
- [ ] Integration with platform normalizers
- [ ] Redis-backed state for distributed circuit breakers

**Files to Create/Modify**:
- `crates/core/src/resilience/mod.rs` (new)
- `crates/core/src/resilience/circuit_breaker.rs` (new)
- `crates/ingestion/src/normalizer/mod.rs` (modify)
- `crates/sync/src/pubnub.rs` (modify)

**Dependencies**: Redis

---

### TASK-010: Health Aggregation Gateway Endpoint
**Priority**: P2-Medium
**Complexity**: Low
**Estimated LOC**: 200-250
**Crate**: `api-gateway` or main entry service

**Description**:
Implement a unified health check endpoint that aggregates health status from all microservices, providing a single point for monitoring and orchestration.

**Acceptance Criteria**:
- [ ] `HealthAggregator` service with concurrent health checks
- [ ] `GET /health/aggregate` - Combined health status
- [ ] Per-service health with response times
- [ ] Dependency health (PostgreSQL, Redis, Qdrant)
- [ ] Graceful degradation reporting (partial healthy)
- [ ] Configurable timeouts per service
- [ ] Cache health results (5-second TTL)
- [ ] JSON response with detailed breakdown

**Files to Create/Modify**:
- `crates/api/src/health/mod.rs` (new or expand)
- `crates/api/src/health/aggregator.rs` (new)
- `crates/api/src/routes.rs` (modify)

**Dependencies**: All service health endpoints (already exist)

---

## Implementation Order

The recommended implementation sequence based on dependencies and priority:

1. **TASK-002**: GitHub OAuth (builds on existing OAuth patterns)
2. **TASK-001**: MFA System (security critical)
3. **TASK-007**: API Key Management (completes auth story)
4. **TASK-006**: Distributed Tracing (enables debugging for remaining tasks)
5. **TASK-009**: Circuit Breaker (resilience foundation)
6. **TASK-010**: Health Aggregation (operational visibility)
7. **TASK-005**: Continue Watching API (user feature)
8. **TASK-004**: Webhook Integration (platform connectivity)
9. **TASK-003**: Collaborative Filtering (recommendation enhancement)
10. **TASK-008**: Search Analytics (insight generation)

---

## Verification Checklist

For each completed task, verify:

- [ ] All acceptance criteria met
- [ ] Unit tests with >80% coverage
- [ ] Integration tests where applicable
- [ ] No compilation warnings
- [ ] Documentation updated
- [ ] SPARC Refinement patterns followed (TDD)
- [ ] Security review for auth-related tasks

---

## Notes

- **No duplication**: All tasks are new work not covered in BATCH_001-005
- **SPARC aligned**: Each task follows Specification → Pseudocode → Architecture → Refinement → Completion
- **Priority justified**: P0 tasks address security gaps, P1 tasks enable core features, P2 tasks enhance operations
- **Incremental**: Tasks can be parallelized by different teams/agents

---

*Generated by BATCH_006 Analysis Swarm*
