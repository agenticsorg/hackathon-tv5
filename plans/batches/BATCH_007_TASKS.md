# BATCH_007: Media Gateway Action List

**Generated**: 2025-12-06
**Analysis Method**: 9-Agent Claude-Flow Swarm Analysis
**Previous Batches**: BATCH_001 through BATCH_006 Completed (70 tasks total)
**Focus**: User Management, Testing Infrastructure, Admin APIs, Advanced Integrations

---

## Executive Summary

Following comprehensive analysis of the repository after BATCH_001-006 implementation, this batch focuses on:
1. **User Management** - Registration, login, password reset, email verification
2. **Testing Infrastructure** - Integration test suites, E2E testing framework
3. **Admin APIs** - User administration, system configuration, audit logging
4. **Advanced Sync** - WebSocket broadcasting, conflict visualization
5. **Content Management** - Catalog CRUD, content moderation workflows

---

## Task List

### TASK-001: Implement User Registration and Password Authentication
**Priority**: P0-Critical
**Complexity**: High
**Estimated LOC**: 450-500
**Crate**: `auth`

**Description**:
The auth system supports OAuth, MFA, and API keys but has NO native email/password registration or login. All user creation currently relies on OAuth flows. Implement complete user registration with email/password authentication including password hashing, validation, and secure storage.

**Acceptance Criteria**:
- [ ] `UserRepository` trait with `create_user()`, `find_by_email()`, `find_by_id()` methods
- [ ] `POST /api/v1/auth/register` - Create user with email, password, display_name
- [ ] `POST /api/v1/auth/login` - Authenticate with email/password, return JWT tokens
- [ ] Password hashing using argon2id with configurable parameters
- [ ] Password strength validation (min 8 chars, uppercase, lowercase, number)
- [ ] `users` PostgreSQL table with migration
- [ ] Rate limiting: 5 registration attempts per IP per hour
- [ ] Integration tests for registration and login flows

**Files to Create/Modify**:
- `crates/auth/src/user/mod.rs` (new)
- `crates/auth/src/user/repository.rs` (new)
- `crates/auth/src/user/password.rs` (new)
- `crates/auth/src/handlers.rs` (modify)
- `migrations/010_create_users.sql` (new)

**Dependencies**: None

---

### TASK-002: Implement Email Verification Flow
**Priority**: P0-Critical
**Complexity**: Medium
**Estimated LOC**: 300-350
**Crate**: `auth`

**Description**:
Users can register but there's no email verification. Unverified accounts pose security and spam risks. Implement email verification token generation, sending, and validation with configurable expiration.

**Acceptance Criteria**:
- [ ] `EmailService` trait with `send_verification()`, `send_password_reset()` methods
- [ ] `POST /api/v1/auth/verify-email` - Verify token and activate account
- [ ] `POST /api/v1/auth/resend-verification` - Resend verification email
- [ ] Verification tokens stored in Redis with 24-hour TTL
- [ ] `email_verified` boolean column in users table
- [ ] Block login for unverified accounts (configurable)
- [ ] SendGrid/AWS SES abstraction for email delivery
- [ ] Email templates for verification (HTML + plaintext)

**Files to Create/Modify**:
- `crates/auth/src/email/mod.rs` (new)
- `crates/auth/src/email/service.rs` (new)
- `crates/auth/src/email/templates.rs` (new)
- `crates/auth/src/handlers.rs` (modify)

**Dependencies**: TASK-001 (User repository)

---

### TASK-003: Implement Password Reset Flow
**Priority**: P0-Critical
**Complexity**: Medium
**Estimated LOC**: 250-300
**Crate**: `auth`

**Description**:
No password reset mechanism exists. Users who forget passwords have no recovery path. Implement secure password reset with email-based token verification and password change functionality.

**Acceptance Criteria**:
- [ ] `POST /api/v1/auth/password/forgot` - Request password reset email
- [ ] `POST /api/v1/auth/password/reset` - Reset password with token
- [ ] Reset tokens stored in Redis with 1-hour TTL
- [ ] Single-use tokens (invalidate after successful reset)
- [ ] Invalidate all existing sessions on password change
- [ ] Rate limiting: 3 reset requests per email per hour
- [ ] Email notification when password is changed
- [ ] Integration tests for reset flow

**Files to Create/Modify**:
- `crates/auth/src/password_reset.rs` (new)
- `crates/auth/src/handlers.rs` (modify)
- `crates/auth/src/email/templates.rs` (modify)

**Dependencies**: TASK-001 (User repository), TASK-002 (Email service)

---

### TASK-004: Implement User Profile Management API
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 300-350
**Crate**: `auth`

**Description**:
Users can authenticate but cannot view or update their profile. No endpoints exist for profile retrieval, display name updates, avatar management, or preference settings.

**Acceptance Criteria**:
- [ ] `GET /api/v1/users/me` - Get current user profile
- [ ] `PATCH /api/v1/users/me` - Update profile (display_name, avatar_url, preferences)
- [ ] `DELETE /api/v1/users/me` - Delete account (soft delete with 30-day grace period)
- [ ] `POST /api/v1/users/me/avatar` - Upload avatar (S3/GCS storage)
- [ ] User preferences JSON column for settings
- [ ] Profile response includes linked OAuth providers
- [ ] Audit logging for profile changes
- [ ] Integration tests for all profile operations

**Files to Create/Modify**:
- `crates/auth/src/profile/mod.rs` (new)
- `crates/auth/src/profile/handlers.rs` (new)
- `crates/auth/src/handlers.rs` (modify)
- `migrations/011_add_user_preferences.sql` (new)

**Dependencies**: TASK-001 (User repository)

---

### TASK-005: Implement Admin User Management API
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 350-400
**Crate**: `auth`

**Description**:
No administrative endpoints exist for user management. Admins cannot list users, suspend accounts, assign roles, or view audit logs. Essential for platform operations and compliance.

**Acceptance Criteria**:
- [ ] `GET /api/v1/admin/users` - List users with pagination, filtering, sorting
- [ ] `GET /api/v1/admin/users/{id}` - Get user details with activity history
- [ ] `PATCH /api/v1/admin/users/{id}` - Update user (suspend, change role)
- [ ] `DELETE /api/v1/admin/users/{id}` - Hard delete user (GDPR compliance)
- [ ] `POST /api/v1/admin/users/{id}/impersonate` - Generate impersonation token
- [ ] Admin-only middleware with role verification
- [ ] Audit logging for all admin actions
- [ ] Integration tests with admin authentication

**Files to Create/Modify**:
- `crates/auth/src/admin/mod.rs` (new)
- `crates/auth/src/admin/handlers.rs` (new)
- `crates/auth/src/admin/middleware.rs` (new)
- `crates/auth/src/handlers.rs` (modify)

**Dependencies**: TASK-001 (User repository)

---

### TASK-006: Implement Audit Logging System
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 300-350
**Crate**: `core`

**Description**:
No centralized audit logging exists. Security events, admin actions, and data changes are not tracked. Required for compliance (SOC2, GDPR) and security incident investigation.

**Acceptance Criteria**:
- [ ] `AuditLogger` trait with `log_event()` method
- [ ] `PostgresAuditLogger` implementation with async writes
- [ ] `audit_logs` table with (timestamp, user_id, action, resource, details, ip_address)
- [ ] Event types: AUTH_LOGIN, AUTH_LOGOUT, USER_CREATED, USER_UPDATED, ADMIN_ACTION, etc.
- [ ] Query API: `GET /api/v1/admin/audit-logs` with date range and filtering
- [ ] Retention policy: 90 days default, configurable
- [ ] Batch insert for high-volume events
- [ ] Integration with existing auth middleware

**Files to Create/Modify**:
- `crates/core/src/audit/mod.rs` (new)
- `crates/core/src/audit/logger.rs` (new)
- `crates/core/src/audit/types.rs` (new)
- `crates/core/src/lib.rs` (modify)
- `migrations/012_create_audit_logs.sql` (new)

**Dependencies**: None

---

### TASK-007: Implement Catalog Content CRUD API
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 400-450
**Crate**: `discovery`

**Description**:
Discovery service has search but NO content management API. Content can only be ingested via pipeline, not created/updated/deleted through API. Admins need direct catalog management capabilities.

**Acceptance Criteria**:
- [ ] `POST /api/v1/admin/catalog/content` - Create content entry manually
- [ ] `GET /api/v1/admin/catalog/content/{id}` - Get content with all metadata
- [ ] `PATCH /api/v1/admin/catalog/content/{id}` - Update content metadata
- [ ] `DELETE /api/v1/admin/catalog/content/{id}` - Remove content (soft delete)
- [ ] `POST /api/v1/admin/catalog/content/{id}/availability` - Update availability
- [ ] Validation for required fields (title, content_type, platform)
- [ ] Automatic Qdrant vector update on content changes
- [ ] Emit Kafka events for content changes
- [ ] Admin authentication required

**Files to Create/Modify**:
- `crates/discovery/src/catalog/mod.rs` (new)
- `crates/discovery/src/catalog/handlers.rs` (new)
- `crates/discovery/src/catalog/service.rs` (new)
- `crates/discovery/src/server.rs` (modify)

**Dependencies**: None

---

### TASK-008: Implement WebSocket Broadcasting for Sync Service
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 350-400
**Crate**: `sync`

**Description**:
PubNub handles cross-device sync but WebSocket connections only receive messages, never broadcast to other connected devices of the same user. Local WebSocket clients should receive real-time updates from PubNub events.

**Acceptance Criteria**:
- [ ] `WebSocketBroadcaster` struct managing per-user connection pools
- [ ] Subscribe to PubNub user channels and relay to WebSocket clients
- [ ] Connection registry: track active WebSocket connections per user
- [ ] Broadcast message types: WATCHLIST_UPDATE, PROGRESS_UPDATE, DEVICE_COMMAND
- [ ] Graceful handling of WebSocket disconnections
- [ ] Metrics: active connections, messages relayed, broadcast latency
- [ ] Integration with existing `websocket.rs` handler
- [ ] Integration tests with multiple simulated clients

**Files to Create/Modify**:
- `crates/sync/src/websocket/broadcaster.rs` (new)
- `crates/sync/src/websocket/registry.rs` (new)
- `crates/sync/src/websocket/mod.rs` (modify)
- `crates/sync/src/main.rs` (modify)

**Dependencies**: None

---

### TASK-009: Implement Integration Test Framework
**Priority**: P1-High
**Complexity**: High
**Estimated LOC**: 500-600
**Crate**: `tests` (new workspace member)

**Description**:
No integration test framework exists. Each crate has unit tests but no cross-service integration tests. Critical user flows (registration -> login -> search -> playback) are untested end-to-end.

**Acceptance Criteria**:
- [ ] Create `tests/` workspace member with shared test utilities
- [ ] `TestContext` struct that spins up test database, Redis, and services
- [ ] Database migration runner for test setup
- [ ] Test fixtures for users, content, sessions
- [ ] HTTP client wrapper for authenticated requests
- [ ] Integration tests for auth flow: register -> verify -> login -> refresh
- [ ] Integration tests for search flow: search -> get details -> track view
- [ ] Integration tests for playback flow: create session -> update position -> resume
- [ ] CI/CD integration with `cargo test --workspace`

**Files to Create/Modify**:
- `tests/Cargo.toml` (new)
- `tests/src/lib.rs` (new)
- `tests/src/context.rs` (new)
- `tests/src/fixtures.rs` (new)
- `tests/src/auth_tests.rs` (new)
- `tests/src/search_tests.rs` (new)
- `tests/src/playback_tests.rs` (new)
- `Cargo.toml` (modify - add workspace member)

**Dependencies**: TASK-001 (User authentication for test flows)

---

### TASK-010: Implement Content Quality Scoring System
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 300-350
**Crate**: `ingestion`

**Description**:
Content has no quality score. All content is treated equally regardless of metadata completeness, image quality, or data freshness. Quality scoring enables better search ranking and identifies content needing enrichment.

**Acceptance Criteria**:
- [ ] `QualityScorer` struct with configurable scoring rules
- [ ] Scoring dimensions: metadata_completeness, image_quality, freshness, external_ratings
- [ ] `quality_score` float column in content table (0.0 - 1.0)
- [ ] Scoring rules: +0.1 for description, +0.1 for poster, +0.1 for IMDB rating, etc.
- [ ] Batch scoring job in metadata enrichment pipeline
- [ ] Quality score decay over time (freshness factor)
- [ ] `GET /api/v1/admin/content/quality-report` - Low-quality content report
- [ ] Integration with search ranking (quality boost factor)

**Files to Create/Modify**:
- `crates/ingestion/src/quality/mod.rs` (new)
- `crates/ingestion/src/quality/scorer.rs` (new)
- `crates/ingestion/src/pipeline.rs` (modify)
- `migrations/013_add_quality_score.sql` (new)

**Dependencies**: None

---

### TASK-011: Implement Apple OAuth Provider
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 280-320
**Crate**: `auth`

**Description**:
Google and GitHub OAuth providers exist but Apple Sign-In is missing. Apple Sign-In is required for iOS apps per Apple App Store guidelines. Different from other OAuth providers due to JWT-based client secret.

**Acceptance Criteria**:
- [ ] `AppleOAuthProvider` implementing `OAuthProvider` trait
- [ ] JWT client secret generation using Apple private key
- [ ] Authorization URL with `openid`, `email`, `name` scopes
- [ ] Token exchange at `https://appleid.apple.com/auth/token`
- [ ] ID token validation and user info extraction
- [ ] Handle Apple's privacy relay email addresses
- [ ] `GET /auth/oauth/apple/authorize` and `GET /auth/oauth/apple/callback` endpoints
- [ ] Configuration via `APPLE_CLIENT_ID`, `APPLE_TEAM_ID`, `APPLE_KEY_ID`, `APPLE_PRIVATE_KEY`
- [ ] Unit tests with mocked Apple responses

**Files to Create/Modify**:
- `crates/auth/src/oauth/providers/apple.rs` (new)
- `crates/auth/src/oauth/providers/mod.rs` (modify)
- `crates/auth/src/handlers.rs` (modify)

**Dependencies**: None

---

### TASK-012: Implement Parental Controls System
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 350-400
**Crate**: `auth` + `discovery`

**Description**:
No parental controls exist. All content is accessible to all users regardless of age ratings. Family accounts need PIN-protected access to mature content and viewing time restrictions.

**Acceptance Criteria**:
- [ ] `ParentalControls` struct with content_rating_limit, viewing_time_limits, PIN
- [ ] `PATCH /api/v1/users/me/parental-controls` - Configure controls
- [ ] `POST /api/v1/users/me/parental-controls/verify-pin` - Verify PIN for access
- [ ] Content rating hierarchy: G < PG < PG-13 < R < NC-17
- [ ] Search/recommendations filter content above rating limit
- [ ] Time-based restrictions: viewing windows (e.g., 6am-9pm)
- [ ] PIN verification caching (5 minutes)
- [ ] `parental_controls` JSON column in users table
- [ ] Integration with discovery search filtering

**Files to Create/Modify**:
- `crates/auth/src/parental/mod.rs` (new)
- `crates/auth/src/parental/controls.rs` (new)
- `crates/discovery/src/search/filters.rs` (modify)
- `migrations/014_add_parental_controls.sql` (new)

**Dependencies**: TASK-001 (User repository)

---

## Implementation Order

The recommended implementation sequence based on dependencies and priority:

1. **TASK-001**: User Registration/Login (foundational)
2. **TASK-002**: Email Verification (depends on TASK-001)
3. **TASK-003**: Password Reset (depends on TASK-001, TASK-002)
4. **TASK-006**: Audit Logging (independent, enables compliance)
5. **TASK-004**: User Profile Management (depends on TASK-001)
6. **TASK-005**: Admin User Management (depends on TASK-001)
7. **TASK-007**: Catalog Content CRUD (independent)
8. **TASK-008**: WebSocket Broadcasting (independent)
9. **TASK-009**: Integration Test Framework (depends on TASK-001)
10. **TASK-010**: Content Quality Scoring (independent)
11. **TASK-011**: Apple OAuth Provider (independent)
12. **TASK-012**: Parental Controls (depends on TASK-001)

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
- [ ] Database migrations tested

---

## Notes

- **No duplication**: All tasks are new work not covered in BATCH_001-006
- **SPARC aligned**: Each task follows Specification -> Pseudocode -> Architecture -> Refinement -> Completion
- **Priority justified**: P0 tasks address critical missing user management, P1 tasks enable operations, P2 tasks enhance features
- **Incremental**: Tasks can be parallelized by different teams/agents

---

## Completed Tasks Inventory (BATCH_001-006)

To ensure no duplication, the following tasks have been completed:

**Auth Crate**: OAuth infrastructure, PKCE, Device Authorization, Token Families, MFA (TOTP), Backup Codes, API Keys, Rate Limiting, GitHub OAuth, Google OAuth, Session Management, RBAC, Scopes

**Discovery Crate**: Hybrid Search, Vector Search, Keyword Search, Intent Parsing, Autocomplete, Faceted Search, Spell Correction, Redis Caching, Search Analytics, Personalization

**SONA Crate**: LoRA Adapters, Collaborative Filtering (ALS), Content-Based Filtering, Graph Recommendations, A/B Testing, Diversity Filter, Cold Start, Context-Aware Filtering, ONNX Inference

**Ingestion Crate**: Platform Normalizers (Netflix, Prime, Disney+, HBO Max, Hulu, Apple TV+, Paramount+, Peacock), Entity Resolution, Embedding Generation, Qdrant Indexing, Webhooks, Pipeline, Repository

**Sync Crate**: CRDT (HLC, LWW, OR-Set), PubNub Integration, Offline Queue, PostgreSQL Persistence, Device Management, Watchlist Sync, Progress Sync

**Playback Crate**: Session Management, Continue Watching, Progress Tracking, Resume Position, Kafka Events

**Core Crate**: Database Pool, Config Loader, Observability, Metrics, Health Checks, Retry Utility, Pagination, Graceful Shutdown, Circuit Breaker, OpenTelemetry Tracing

**Infrastructure**: Docker Compose (all services), Jaeger, Kafka

---

*Generated by BATCH_007 Analysis Swarm*
