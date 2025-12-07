# BATCH_004 Integration Checklist

## Overview
This document provides a comprehensive checklist for integrating BATCH_004 tasks into the Media Gateway codebase. All tasks have been implemented by specialized agents, and this checklist ensures proper module exports, dependency verification, and testing.

---

## 1. Cargo.toml Dependency Verification

### ✅ Core Dependencies (Already Present)
All required dependencies for BATCH_004 features are already present in workspace Cargo.toml:

- **Spell Correction & Autocomplete (TASK-001, TASK-002)**:
  - ✅ `strsim = "0.11"` - String similarity for spell correction
  - ✅ `regex = "1.10"` - Pattern matching
  - ✅ `tantivy = "0.22"` - Full-text search index (in discovery)

- **Faceted Search (TASK-003)**:
  - ✅ `serde = { version = "1", features = ["derive"] }` - Serialization
  - ✅ `serde_json = "1"` - JSON handling
  - ✅ `sqlx` - Database aggregations

- **A/B Experiments (TASK-004)**:
  - ✅ `uuid` - Experiment ID generation
  - ✅ `chrono` - Timestamp tracking
  - ✅ `redis` - Fast assignment lookups

- **Token Revocation (TASK-005)**:
  - ✅ `jsonwebtoken = "9"` - JWT handling
  - ✅ `redis` - Token blacklist
  - ✅ `sqlx` - Family tracking

- **Pagination (TASK-009)**:
  - ✅ `sqlx` - Database queries
  - ✅ `serde` - Response serialization

- **Shutdown Coordination (TASK-010)**:
  - ✅ `tokio` - Async runtime with broadcast channels
  - ✅ `tracing` - Logging

- **Resume Position (TASK-011)**:
  - ✅ `sqlx` - Session storage
  - ✅ `chrono` - Time tracking
  - ✅ `uuid` - Session IDs

### 🔍 Missing Dependencies
**NONE** - All dependencies are present!

---

## 2. Module Export Updates Required

### 📦 Discovery Crate (`crates/discovery/src/`)

#### `search/mod.rs` - Exports for TASK-001, TASK-002, TASK-003
**Status**: ✅ Already properly structured

**Required additions** (if agents created new modules):
```rust
// In search/mod.rs, add:
pub mod spell_correction;      // TASK-001
pub mod autocomplete;          // TASK-002
pub mod facets;                // TASK-003

// Re-export public types
pub use spell_correction::SpellCorrector;
pub use autocomplete::{AutocompleteService, AutocompleteSuggestion};
pub use facets::{FacetedSearchResponse, Facet, FacetValue};
```

#### `lib.rs` - Top-level exports
**Status**: ⚠️ Needs verification

**Required additions**:
```rust
// In crates/discovery/src/lib.rs
pub use search::{
    SpellCorrector,
    AutocompleteService,
    AutocompleteSuggestion,
    FacetedSearchResponse,
    Facet,
    FacetValue,
};
```

---

### 📦 SONA Crate (`crates/sona/src/`)

#### `lib.rs` - Exports for TASK-004
**Status**: ⚠️ Needs verification

**Required additions**:
```rust
// In crates/sona/src/lib.rs
pub mod experiments;  // TASK-004

// Re-export public types
pub use experiments::{
    ExperimentAssignment,
    ExperimentManager,
    Variant,
    AssignmentConsistency,
};
```

---

### 📦 Auth Crate (`crates/auth/src/`)

#### `lib.rs` - Exports for TASK-005
**Status**: ⚠️ Needs verification

**Current exports** (from existing code):
```rust
pub mod token;
pub use token::{TokenManager, TokenType};
```

**Required additions**:
```rust
// In crates/auth/src/lib.rs
pub use token::{
    TokenManager,
    TokenType,
    TokenFamily,           // TASK-005
    FamilyRevocation,      // TASK-005
};
```

**Or if new module created**:
```rust
// In crates/auth/src/lib.rs
pub mod token_families;  // TASK-005
pub use token_families::{TokenFamilyManager, RevocationStatus};
```

---

### 📦 Core Crate (`crates/core/src/`)

#### `lib.rs` - Exports for TASK-009, TASK-010
**Status**: ⚠️ Needs verification

**Required additions**:
```rust
// In crates/core/src/lib.rs
pub mod pagination;         // TASK-009
pub mod shutdown;           // TASK-010

// Re-export public types
pub use pagination::{
    PaginatedResponse,
    PaginationParams,
    PageInfo,
    CursorPagination,
};

pub use shutdown::{
    ShutdownCoordinator,
    ShutdownSignal,
    GracefulShutdown,
};
```

---

### 📦 Playback Crate (`crates/playback/src/`)

#### `lib.rs` - Create and export TASK-011
**Status**: ❌ **CRITICAL - playback crate has NO lib.rs!**

**Action required**:
```rust
// CREATE FILE: crates/playback/src/lib.rs

pub mod events;
pub mod session;
pub mod resume;  // TASK-011

// Re-export public types
pub use events::{PlaybackEvent, PlaybackEventType};
pub use session::{PlaybackSession, SessionManager};
pub use resume::{
    ResumePosition,
    ResumeCalculator,
    calculate_resume_position,
};

#[cfg(test)]
mod tests;
```

---

### 📦 Ingestion Crate (`crates/ingestion/src/`)

#### `normalizer/mod.rs` - No changes needed for BATCH_004
**Status**: ✅ Complete

Current structure is sufficient. No BATCH_004 tasks affect this module.

---

## 3. Module Export Checklist

### Priority 1: Critical (Blocks compilation)
- [ ] **Create** `/workspaces/media-gateway/crates/playback/src/lib.rs`
- [ ] Add `pub mod resume;` to playback lib.rs
- [ ] Add `pub use resume::*;` exports

### Priority 2: High (Required for integration tests)
- [ ] Verify `crates/discovery/src/search/mod.rs` exports spell_correction
- [ ] Verify `crates/discovery/src/search/mod.rs` exports autocomplete
- [ ] Verify `crates/discovery/src/search/mod.rs` exports facets
- [ ] Update `crates/discovery/src/lib.rs` to re-export new types

### Priority 3: Medium (Required for service integration)
- [ ] Verify `crates/sona/src/lib.rs` exports experiments module
- [ ] Verify `crates/auth/src/lib.rs` exports token family types
- [ ] Verify `crates/core/src/lib.rs` exports pagination module
- [ ] Verify `crates/core/src/lib.rs` exports shutdown module

### Priority 4: Low (Documentation and cleanup)
- [ ] Update module documentation in lib.rs files
- [ ] Add feature flags if needed for optional functionality
- [ ] Update crate-level docs with BATCH_004 features

---

## 4. Docker Compose Verification

### Current Services Analysis
**File**: `/workspaces/media-gateway/docker-compose.yml`

#### ✅ PostgreSQL (Port 5432)
- **Status**: Sufficient for all BATCH_004 features
- **Usage**:
  - TASK-001: Spell corrections table
  - TASK-003: Faceted search aggregations
  - TASK-004: Experiment assignments
  - TASK-005: Token families
  - TASK-009: Pagination queries
  - TASK-011: Playback sessions

#### ✅ Redis (Port 6379)
- **Status**: Sufficient for all BATCH_004 features
- **Usage**:
  - TASK-002: Autocomplete cache
  - TASK-004: Experiment assignment cache
  - TASK-005: Token revocation blacklist
  - General: Search result caching

#### ✅ Qdrant (Ports 6333/6334)
- **Status**: Sufficient
- **Usage**: Vector search (existing feature, not BATCH_004)

### 🎯 No Changes Required
All BATCH_004 features can be implemented with existing infrastructure:
- PostgreSQL for persistent data
- Redis for caching and blacklists
- Qdrant for vector search (unchanged)

### Optional Enhancements (Future)
```yaml
# Optional: Kafka for event streaming (TASK-010 advanced coordination)
kafka:
  image: confluentinc/cp-kafka:latest
  # ... configuration

# Optional: Elasticsearch for advanced autocomplete (TASK-002)
elasticsearch:
  image: elasticsearch:8.11.0
  # ... configuration
```

**Decision**: Not required for BATCH_004 initial implementation.

---

## 5. Database Migration Requirements

### Required Migrations

#### TASK-001: Spell Corrections
```sql
-- migrations/YYYYMMDD_001_spell_corrections.sql
CREATE TABLE IF NOT EXISTS spell_corrections (
    misspelled TEXT PRIMARY KEY,
    corrected TEXT NOT NULL,
    frequency INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_spell_corrections_corrected ON spell_corrections(corrected);
```

#### TASK-004: Experiment Assignments
```sql
-- migrations/YYYYMMDD_002_experiments.sql
CREATE TABLE IF NOT EXISTS experiments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    variants TEXT[] NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    active BOOLEAN DEFAULT true
);

CREATE TABLE IF NOT EXISTS experiment_assignments (
    user_id UUID NOT NULL,
    experiment_id TEXT NOT NULL,
    variant TEXT NOT NULL,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, experiment_id),
    FOREIGN KEY (experiment_id) REFERENCES experiments(id)
);

CREATE INDEX idx_experiment_assignments_experiment ON experiment_assignments(experiment_id);
```

#### TASK-005: Token Families
```sql
-- migrations/YYYYMMDD_003_token_families.sql
CREATE TABLE IF NOT EXISTS token_families (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS token_instances (
    id UUID PRIMARY KEY,
    family_id UUID NOT NULL,
    token_hash TEXT NOT NULL,
    issued_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (family_id) REFERENCES token_families(id) ON DELETE CASCADE
);

CREATE INDEX idx_token_families_user ON token_families(user_id);
CREATE INDEX idx_token_families_revoked ON token_families(revoked_at) WHERE revoked_at IS NULL;
CREATE INDEX idx_token_instances_family ON token_instances(family_id);
CREATE INDEX idx_token_instances_hash ON token_instances(token_hash);
```

#### TASK-011: Playback Sessions
```sql
-- migrations/YYYYMMDD_004_playback_sessions.sql
CREATE TABLE IF NOT EXISTS playback_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    content_id UUID NOT NULL,
    position_ms INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (content_id) REFERENCES content(id)
);

CREATE INDEX idx_playback_sessions_user_content ON playback_sessions(user_id, content_id);
CREATE INDEX idx_playback_sessions_updated ON playback_sessions(updated_at DESC);
```

### Migration Checklist
- [ ] Create migration files in `/workspaces/media-gateway/migrations/`
- [ ] Test migrations locally
- [ ] Update `scripts/init-db.sql` with tables
- [ ] Document rollback procedures

---

## 6. Integration Test Execution Plan

### Test File
**Location**: `/workspaces/media-gateway/tests/batch_004_integration.rs`

### Running Tests

#### Run All Integration Tests
```bash
# Set up environment
export DATABASE_URL="postgresql://mediagateway:localdev123@localhost/media_gateway"
export REDIS_URL="redis://localhost:6379"
export QDRANT_URL="http://localhost:6333"

# Start infrastructure
docker-compose up -d

# Wait for health checks
sleep 10

# Run integration tests
cargo test --test batch_004_integration -- --ignored --test-threads=1
```

#### Run Individual Task Tests
```bash
# TASK-001: Spell correction
cargo test --test batch_004_integration test_search_spell_correction_integration -- --ignored

# TASK-002: Autocomplete
cargo test --test batch_004_integration test_autocomplete_integration -- --ignored

# TASK-003: Faceted search
cargo test --test batch_004_integration test_faceted_search_integration -- --ignored

# TASK-004: A/B experiments
cargo test --test batch_004_integration test_ab_experiment_consistency -- --ignored

# TASK-005: Token revocation
cargo test --test batch_004_integration test_token_family_revocation -- --ignored

# TASK-009: Pagination
cargo test --test batch_004_integration test_pagination_utilities -- --ignored

# TASK-010: Shutdown coordination (no infrastructure needed)
cargo test --test batch_004_integration test_shutdown_coordination

# TASK-011: Resume position
cargo test --test batch_004_integration test_resume_position_calculation -- --ignored
```

#### Performance Benchmarks
```bash
# Run pagination performance benchmark
cargo test --test batch_004_integration benchmark_pagination_performance -- --ignored --nocapture
```

---

## 7. Manual Testing Steps

### TASK-001: Spell Correction
```bash
# 1. Populate spell corrections table
psql $DATABASE_URL << EOF
INSERT INTO spell_corrections (misspelled, corrected) VALUES
    ('teh', 'the'),
    ('moive', 'movie'),
    ('reccomend', 'recommend');
EOF

# 2. Test search with misspelling
curl -X POST http://localhost:8080/api/v1/search \
  -H "Content-Type: application/json" \
  -d '{"query": "teh matrix", "page": 1, "page_size": 10}'

# 3. Verify response includes corrected query suggestion
```

### TASK-002: Autocomplete
```bash
# 1. Test autocomplete endpoint
curl "http://localhost:8080/api/v1/autocomplete?q=the&limit=10"

# 2. Verify suggestions are sorted by popularity
# 3. Test with empty query (should return popular items)
curl "http://localhost:8080/api/v1/autocomplete?q=&limit=5"
```

### TASK-003: Faceted Search
```bash
# 1. Test faceted search response
curl -X POST http://localhost:8080/api/v1/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "action",
    "include_facets": true,
    "page": 1,
    "page_size": 20
  }'

# 2. Verify response includes facets object with:
#    - genres (with counts)
#    - platforms (with counts)
#    - year_range (min/max)
#    - rating_range (min/max)
```

### TASK-004: A/B Experiments
```bash
# 1. Create test experiment
curl -X POST http://localhost:8080/api/v1/experiments \
  -H "Content-Type: application/json" \
  -d '{
    "id": "search_algo_test",
    "name": "Search Algorithm Test",
    "variants": ["control", "new_algo"],
    "active": true
  }'

# 2. Request assignment for same user multiple times
USER_ID="550e8400-e29b-41d4-a716-446655440000"
for i in {1..5}; do
  curl "http://localhost:8080/api/v1/experiments/search_algo_test/assign?user_id=$USER_ID"
done

# 3. Verify user gets same variant every time
```

### TASK-005: Token Family Revocation
```bash
# 1. Login to get token family
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'

# Save family_id from response

# 2. Revoke token family
FAMILY_ID="<family_id_from_login>"
curl -X POST "http://localhost:8080/api/v1/auth/revoke-family/$FAMILY_ID"

# 3. Try to use token - should fail with 401
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/user/profile
```

### TASK-009: Pagination
```bash
# 1. Test offset-based pagination
curl "http://localhost:8080/api/v1/content?page=1&page_size=20"
curl "http://localhost:8080/api/v1/content?page=2&page_size=20"

# 2. Test cursor-based pagination
curl "http://localhost:8080/api/v1/content?limit=20"
# Use next_cursor from response
curl "http://localhost:8080/api/v1/content?cursor=<next_cursor>&limit=20"

# 3. Verify no duplicate items across pages
```

### TASK-010: Shutdown Coordination
```bash
# 1. Start a service
cargo run --bin discovery-service &
SERVICE_PID=$!

# 2. Monitor logs
tail -f logs/discovery.log &

# 3. Send graceful shutdown signal
kill -SIGTERM $SERVICE_PID

# 4. Verify in logs:
#    - "Received shutdown signal"
#    - "Closing database connections"
#    - "Draining in-flight requests"
#    - "Service stopped gracefully"
```

### TASK-011: Resume Position
```bash
# 1. Start playback and update position
curl -X POST http://localhost:8080/api/v1/playback/start \
  -H "Authorization: Bearer <token>" \
  -d '{
    "content_id": "<content_id>",
    "position_ms": 0
  }'

# 2. Update position multiple times
curl -X POST http://localhost:8080/api/v1/playback/position \
  -H "Authorization: Bearer <token>" \
  -d '{
    "session_id": "<session_id>",
    "position_ms": 300000
  }'

# 3. Request resume position
curl -H "Authorization: Bearer <token>" \
  "http://localhost:8080/api/v1/playback/resume/<content_id>"

# 4. Verify response includes correct position (300000ms)
```

---

## 8. API Documentation Updates Needed

### New Endpoints to Document

#### TASK-002: Autocomplete
```
GET /api/v1/autocomplete
Query params: q (string), limit (int, default 10)
Response: { suggestions: [{ text, score, type }] }
```

#### TASK-003: Faceted Search (enhancement to existing)
```
POST /api/v1/search
Body: { ..., include_facets: boolean }
Response: { ..., facets: { genres, platforms, year_range, rating_range } }
```

#### TASK-004: A/B Experiments
```
POST /api/v1/experiments (admin)
GET /api/v1/experiments/:id/assign
```

#### TASK-005: Token Revocation
```
POST /api/v1/auth/revoke-family/:family_id
```

#### TASK-011: Resume Position
```
GET /api/v1/playback/resume/:content_id
POST /api/v1/playback/position
```

### Documentation Checklist
- [ ] Update OpenAPI spec in `/docs/api/openapi.yaml`
- [ ] Generate updated API documentation
- [ ] Update README.md with new features
- [ ] Create BATCH_004 feature guide

---

## 9. Final Integration Checklist

### Code Integration
- [ ] All lib.rs files updated with module exports
- [ ] playback/src/lib.rs created
- [ ] All public APIs exported at crate level
- [ ] No compilation errors: `cargo build --all`
- [ ] No clippy warnings: `cargo clippy --all`
- [ ] Code formatted: `cargo fmt --all`

### Testing
- [ ] All unit tests pass: `cargo test --lib --all`
- [ ] Integration tests pass: `cargo test --test batch_004_integration -- --ignored`
- [ ] Manual testing completed for all tasks
- [ ] Performance benchmarks executed

### Database
- [ ] Migrations created and tested
- [ ] init-db.sql updated
- [ ] Database indexes verified

### Documentation
- [ ] API documentation updated
- [ ] README.md updated
- [ ] BATCH_004 features documented
- [ ] Code comments reviewed

### Infrastructure
- [ ] Docker Compose verified (no changes needed)
- [ ] Environment variables documented
- [ ] Health checks working

### Deployment Readiness
- [ ] Feature flags configured (if applicable)
- [ ] Monitoring dashboards updated
- [ ] Alerts configured for new features
- [ ] Rollback plan documented

---

## 10. Success Criteria

### TASK-001: Spell Correction ✓
- [ ] Misspelled queries are corrected
- [ ] Suggestions returned in search response
- [ ] Performance impact < 10ms per query

### TASK-002: Autocomplete ✓
- [ ] Suggestions returned in < 50ms
- [ ] Results sorted by popularity
- [ ] Cache hit rate > 80%

### TASK-003: Faceted Search ✓
- [ ] All facet types returned correctly
- [ ] Counts accurate
- [ ] Performance impact < 20ms

### TASK-004: A/B Experiments ✓
- [ ] User assignment consistent
- [ ] Variant distribution correct
- [ ] Assignment latency < 5ms

### TASK-005: Token Revocation ✓
- [ ] Revoked tokens rejected immediately
- [ ] All family tokens invalidated
- [ ] Revocation persisted across restarts

### TASK-009: Pagination ✓
- [ ] No duplicate results
- [ ] Cursor pagination working
- [ ] Performance acceptable for deep pages

### TASK-010: Shutdown Coordination ✓
- [ ] Services shutdown gracefully
- [ ] No dropped requests
- [ ] Resources cleaned up properly

### TASK-011: Resume Position ✓
- [ ] Position calculated correctly
- [ ] Resume offered appropriately
- [ ] Position updates persisted

---

## Next Steps

1. **Immediate** (Priority 1):
   - Create `crates/playback/src/lib.rs`
   - Verify module exports in all crates

2. **Short-term** (Priority 2):
   - Run integration tests
   - Create database migrations
   - Update API documentation

3. **Medium-term** (Priority 3):
   - Performance testing
   - Load testing for new features
   - Monitoring dashboard updates

4. **Long-term** (Priority 4):
   - Feature flags for gradual rollout
   - A/B testing of BATCH_004 features
   - Analytics integration

---

**Document Version**: 1.0
**Last Updated**: 2025-12-06
**Status**: Ready for implementation verification
