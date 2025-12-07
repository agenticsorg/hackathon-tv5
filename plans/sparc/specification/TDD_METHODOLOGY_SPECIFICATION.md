# TDD Methodology & Testing Standards Specification

## Media Gateway: Test-Driven Development Framework

**Document Version:** 1.0.0
**SPARC Phase:** Refinement - TDD Planning
**Date:** 2025-12-06
**Status:** Planning Document (Not Implementation)

---

## Table of Contents

1. [TDD Methodology Overview](#1-tdd-methodology-overview)
2. [TDD Approach Selection](#2-tdd-approach-selection)
3. [Test Pyramid Strategy](#3-test-pyramid-strategy)
4. [Testing Standards per Service](#4-testing-standards-per-service)
5. [Test Categories & Requirements](#5-test-categories--requirements)
6. [Red-Green-Refactor Cycle](#6-red-green-refactor-cycle)
7. [Test Tooling Stack](#7-test-tooling-stack)
8. [Continuous Testing Strategy](#8-continuous-testing-strategy)
9. [Test Data Management](#9-test-data-management)
10. [Quality Gates & Metrics](#10-quality-gates--metrics)

---

## 1. TDD Methodology Overview

### 1.1 Purpose & Scope

This document defines the **Test-Driven Development (TDD) methodology** for the Media Gateway platform. It establishes testing standards, tooling, and processes for all 8 core services across the system.

**Key Principles:**
1. **Tests First**: Write tests before implementation code
2. **Incremental Development**: Small, testable increments
3. **Fast Feedback**: Test suites complete in <5 minutes
4. **High Coverage**: Minimum 80% line coverage, 90% branch coverage
5. **Real Dependencies**: Integration tests use actual databases, not mocks

### 1.2 System Architecture Context

```
┌────────────────────────────────────────────────────────────────────┐
│                    MEDIA GATEWAY SERVICE MAP                        │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TIER 1: CRITICAL PATH (99.9% availability)                        │
│  ──────────────────────────────────────────                        │
│  1. Discovery Engine (Rust)      - Content search & aggregation    │
│  2. SONA Intelligence (Rust)     - ML-powered personalization      │
│  3. MCP Server (TypeScript)      - AI agent protocol bridge        │
│  4. API Gateway (TypeScript)     - REST/GraphQL interface          │
│                                                                     │
│  TIER 2: HIGH PRIORITY (99.5% availability)                        │
│  ──────────────────────────────────────                            │
│  5. Ruvector Storage (Rust)      - Vector search & knowledge graph │
│  6. Auth Service (Rust)          - OAuth 2.0 + token management    │
│                                                                     │
│  TIER 3: STANDARD (99.0% availability)                             │
│  ───────────────────────────────────────                           │
│  7. Ingestion Pipeline (Rust)    - Platform data normalization     │
│  8. CLI Tool (TypeScript)        - Developer interface             │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘
```

### 1.3 Testing Philosophy

**Core Beliefs:**
- **Quality is non-negotiable**: We value the quality we deliver to our users
- **No shortcuts**: Real tests with real data, not mocks disguising incomplete work
- **Fail fast**: Tests catch issues in seconds, not production
- **Document via tests**: Tests serve as executable specifications

---

## 2. TDD Approach Selection

### 2.1 Comparison: London vs Chicago School

| Aspect | London School (Outside-In) | Chicago School (Inside-Out) | Media Gateway Choice |
|--------|---------------------------|----------------------------|---------------------|
| **Starting Point** | User-facing features | Domain models/core logic | **Hybrid (feature-based)** |
| **Mocking Strategy** | Heavy mocking (collaborators) | Minimal mocking (I/O only) | **Selective mocking** |
| **Test Focus** | Behavior verification | State verification | **Both (context-dependent)** |
| **Refactoring** | More brittle (coupled to structure) | More flexible (coupled to behavior) | **Balanced approach** |
| **Discovery** | Design emerges from top-down | Design emerges from bottom-up | **Layered discovery** |

### 2.2 Media Gateway Hybrid Approach

**Rationale**: The Media Gateway combines high-level AI agent interactions with low-level data processing. A pure approach would be suboptimal.

#### 2.2.1 When to Use London School (Outside-In)

**Use Cases:**
- MCP Server tool implementations
- API Gateway endpoint handlers
- CLI command implementations
- User-facing feature workflows

**Example: MCP Tool for Semantic Search**

```typescript
// ✅ LONDON SCHOOL: Start from MCP tool interface (user-facing)
describe('MCP Tool: semantic_search', () => {
  it('should return personalized results for natural language query', async () => {
    // GIVEN: User asks "Something scary like Stranger Things"
    const request = {
      method: 'tools/call',
      params: {
        name: 'semantic_search',
        arguments: {
          query: 'Something scary like Stranger Things',
          userId: 'user123'
        }
      }
    };

    // Mock collaborators (SONA, Discovery Engine)
    const mockSONA = createMockSONA({
      personalizeResults: jest.fn().mockResolvedValue([/* results */])
    });

    const mockDiscovery = createMockDiscovery({
      semanticSearch: jest.fn().mockResolvedValue([/* matches */])
    });

    // WHEN: Tool executes
    const result = await mcpServer.handleToolCall(request);

    // THEN: Verify behavior (not implementation)
    expect(mockDiscovery.semanticSearch).toHaveBeenCalledWith({
      query: 'Something scary like Stranger Things',
      filters: expect.any(Object)
    });
    expect(mockSONA.personalizeResults).toHaveBeenCalled();
    expect(result.content).toHaveLength(10);
    expect(result.content[0].title).toBe('The Haunting of Hill House');
  });
});
```

**Why London Here:**
- Test drives interface design from user perspective
- Collaborators (SONA, Discovery) are complex subsystems
- Focus on integration behavior, not internal algorithms

#### 2.2.2 When to Use Chicago School (Inside-Out)

**Use Cases:**
- SONA recommendation algorithms
- Vector similarity calculations
- Entity resolution logic
- CRDT merge functions
- Rate limiting algorithms

**Example: SONA Personalization Algorithm**

```rust
// ✅ CHICAGO SCHOOL: Start from domain logic (state-based)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sona_scoring_combines_base_similarity_with_user_preferences() {
        // GIVEN: Content item and user preference vector
        let content = ContentEmbedding {
            entity_id: "eidr:10.5240/1234",
            vector: vec![0.8, 0.3, 0.5, 0.2], // Horror, Sci-fi, Drama, Comedy
        };

        let user_prefs = UserPreferences {
            user_id: "user123",
            vector: vec![0.9, 0.1, 0.4, 0.3], // Strong horror preference
            timestamp: Utc::now(),
        };

        let base_similarity = 0.75; // From vector search

        // WHEN: Calculate personalized score
        let sona = SONAEngine::new();
        let score = sona.calculate_personalized_score(&content, &user_prefs, base_similarity);

        // THEN: Score should boost horror content (no mocks, pure calculation)
        assert!(score > base_similarity, "Score should be boosted by user preference");
        assert!(score <= 1.0, "Score should not exceed maximum");
        assert_approx_eq!(score, 0.83, 0.01); // Expected based on formula
    }

    #[test]
    fn test_sona_handles_cold_start_with_default_preferences() {
        // GIVEN: New user with no viewing history
        let user_prefs = UserPreferences::default();
        let content = ContentEmbedding { /* ... */ };
        let base_similarity = 0.60;

        // WHEN: Calculate score
        let sona = SONAEngine::new();
        let score = sona.calculate_personalized_score(&content, &user_prefs, base_similarity);

        // THEN: Should fall back to popularity-based scoring
        assert_eq!(score, base_similarity); // No personalization boost yet
    }
}
```

**Why Chicago Here:**
- Algorithm correctness depends on precise state transformations
- No external dependencies (pure function)
- Mocking would hide calculation bugs
- Tests document the scoring formula

### 2.3 Decision Matrix: When to Mock vs Real Implementation

| Dependency Type | Mock? | Rationale |
|-----------------|-------|-----------|
| **In-Memory Data Structures** | ❌ Never | Use real data (CRDT, vectors, etc.) |
| **Database (PostgreSQL)** | ❌ Integration tests only | Use Testcontainers for isolated DB |
| **Cache (Valkey/Redis)** | ✅ Unit tests | Mock for speed; real in integration |
| **External APIs (TMDb, YouTube)** | ✅ Always | Rate limits, cost, reliability |
| **PubNub (Real-time sync)** | ✅ Unit; ❌ E2E | Mock for unit; real in E2E tests |
| **SONA Engine (ML model)** | ✅ API layer; ❌ Engine tests | Mock at service boundary only |
| **Embedding Models** | ✅ Most tests | Use fixture embeddings; real in perf tests |
| **File System** | ❌ Use temp dirs | Real FS with cleanup |
| **Clock/Time** | ✅ Time-sensitive tests | Mock for determinism |

### 2.4 Layered Testing Strategy

```
┌─────────────────────────────────────────────────────────────────────┐
│               LAYERED TDD APPROACH BY SERVICE LAYER                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  LAYER 4: User-Facing Interfaces (LONDON SCHOOL)                    │
│  ────────────────────────────────────────────────                   │
│  • MCP Server Tools         → Mock collaborators, test behavior     │
│  • API Gateway Endpoints    → Mock services, verify contracts       │
│  • CLI Commands             → Mock APIs, test user flows            │
│                                                                      │
│  LAYER 3: Service Orchestration (HYBRID)                            │
│  ───────────────────────────────────────                            │
│  • Discovery Engine API     → Real Ruvector, mock external APIs     │
│  • SONA Service Layer       → Real algorithms, mock I/O             │
│  • Auth Service             → Real crypto, mock token storage       │
│                                                                      │
│  LAYER 2: Domain Logic (CHICAGO SCHOOL)                             │
│  ──────────────────────────────────────                             │
│  • SONA Algorithms          → No mocks, pure state tests            │
│  • Entity Resolution        → Real fuzzy matching, no mocks         │
│  • CRDT Merge Functions     → Real data structures                  │
│  • Vector Similarity        → Real calculations                     │
│                                                                      │
│  LAYER 1: Infrastructure (CHICAGO SCHOOL)                           │
│  ────────────────────────────────────────                           │
│  • Database Repositories    → Real DB (Testcontainers)              │
│  • Cache Operations         → Real Valkey (Testcontainers)          │
│  • Message Queue            → Real Kafka (Testcontainers)           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Test Pyramid Strategy

### 3.1 Distribution Target

```
               ╱╲
              ╱  ╲             5% E2E Tests
             ╱────╲            ─────────────
            ╱      ╲           • Critical user journeys (5-10 scenarios)
           ╱  E2E   ╲          • Cross-service integration
          ╱──────────╲         • Production-like environment
         ╱            ╲        • Playwright + real backends
        ╱ Integration ╲       • Run time: ~10 minutes
       ╱────────────────╲
      ╱                  ╲    25% Integration Tests
     ╱                    ╲   ──────────────────────
    ╱    Integration       ╲  • Service interactions
   ╱──────────────────────  ╲ • Database + cache + message queue
  ╱                          ╲• Real dependencies (Testcontainers)
 ╱          Unit             ╱• Run time: ~3 minutes
╱────────────────────────────╲
                               70% Unit Tests
                               ─────────────
                               • Component logic
                               • Pure functions
                               • Isolated behaviors
                               • Fast (<1ms per test)
                               • Run time: ~1 minute
```

### 3.2 Test Distribution by Service

| Service | Unit Tests | Integration Tests | Contract Tests | E2E Tests | Total Coverage |
|---------|-----------|------------------|----------------|-----------|----------------|
| **Discovery Engine** | 200 tests (70%) | 60 tests (25%) | 10 tests (5%) | Included in platform E2E | ≥85% |
| **SONA Intelligence** | 150 tests (75%) | 40 tests (20%) | 10 tests (5%) | Included in platform E2E | ≥90% |
| **MCP Server** | 80 tests (60%) | 30 tests (25%) | 20 tests (15%) | 5 E2E flows | ≥85% |
| **API Gateway** | 100 tests (65%) | 40 tests (25%) | 15 tests (10%) | 5 E2E flows | ≥80% |
| **Ruvector Storage** | 120 tests (70%) | 50 tests (30%) | N/A | Included in platform E2E | ≥85% |
| **Auth Service** | 90 tests (70%) | 30 tests (25%) | 5 tests (5%) | 3 E2E flows | ≥85% |
| **Ingestion Pipeline** | 80 tests (65%) | 35 tests (30%) | 5 tests (5%) | Included in platform E2E | ≥80% |
| **CLI Tool** | 60 tests (75%) | 15 tests (20%) | 5 tests (5%) | 10 E2E commands | ≥75% |

### 3.3 Critical User Journeys (E2E Tests)

**Total: 10 E2E Scenarios (Run in CI on every PR)**

1. **Natural Language Search Flow**
   - User searches "scary movies like Stranger Things"
   - SONA personalizes results
   - User selects content → Deep link opens

2. **Cross-Device Watchlist Sync**
   - User adds item on mobile
   - PubNub syncs to TV app (<100ms)
   - User continues on TV

3. **AI Agent Discovery via MCP**
   - Claude discovers ARW manifest
   - Calls semantic_search tool
   - Receives personalized recommendations

4. **OAuth 2.0 Authorization Flow**
   - User logs in via Google OAuth
   - YouTube integration activated
   - User's YouTube watchlist imported

5. **Device Authorization Grant (TV)**
   - Smart TV displays code
   - User enters code on phone
   - TV app authorizes and loads content

6. **Cold Start Personalization**
   - New user (no history)
   - SONA falls back to popularity
   - After 3 interactions, personalization active

7. **Offline-First Watchlist Management**
   - User goes offline
   - Adds/removes watchlist items
   - Reconnects → CRDT merge resolves conflicts

8. **Rate Limit Handling**
   - External API hits rate limit
   - System falls back to cache
   - Queues retry with exponential backoff

9. **Multi-Platform Content Discovery**
   - Content available on Netflix + Prime
   - User searches across both
   - Results show all availability options

10. **Performance Under Load**
    - 100 concurrent searches
    - All complete in <500ms (p95)
    - No errors, no cache stampede

---

## 4. Testing Standards per Service

[Due to length constraints, this section contains summaries. See full document for detailed test requirements for all 8 services]

### 4.1 Discovery Engine (Rust)
- **Coverage**: ≥85% line, ≥90% branch
- **Performance**: <500ms p95 latency
- **Key Tests**: Semantic search, deduplication, caching

### 4.2 SONA Intelligence (Rust)
- **Coverage**: ≥90% (critical algorithms)
- **Performance**: <5ms p99 personalization
- **Key Tests**: Scoring, cold start, preference updates

### 4.3 MCP Server (TypeScript)
- **Coverage**: ≥85%
- **Contract Tests**: ARW compliance, MCP protocol
- **Key Tests**: Tool registration, execution, manifest

### 4.4 API Gateway (TypeScript)
- **Coverage**: ≥80%
- **Contract Tests**: OpenAPI compliance
- **Key Tests**: Request validation, auth, CRUD operations

### 4.5 Ruvector Storage (Rust)
- **Coverage**: ≥85%
- **Performance**: <50ms search at 1M vectors
- **Key Tests**: Vector ops, HNSW index, persistence

### 4.6 Auth Service (Rust)
- **Coverage**: ≥85%
- **Security Tests**: OWASP compliance
- **Key Tests**: JWT, OAuth 2.0, PKCE, device auth

### 4.7 Ingestion Pipeline (Rust)
- **Coverage**: ≥80%
- **Data Quality**: Deduplication accuracy
- **Key Tests**: Normalization, entity resolution

### 4.8 CLI Tool (TypeScript)
- **Coverage**: ≥75%
- **E2E Tests**: All commands
- **Key Tests**: Parsing, execution, output formatting

---

## 5. Test Categories & Requirements

### 5.1 Unit Tests
- **Scope**: Single function/class/module
- **Speed**: <1ms per test
- **Isolation**: No I/O
- **Coverage Target**: 70% of all tests

### 5.2 Integration Tests
- **Scope**: Multiple components + real dependencies
- **Speed**: <100ms per test
- **Dependencies**: Testcontainers (PostgreSQL, Valkey, Kafka)
- **Coverage Target**: 25% of all tests

### 5.3 Contract Tests (Pact)
- **Use Cases**: MCP ↔ AI Agents, API ↔ Frontend
- **Verification**: Provider must honor consumer contracts
- **Coverage Target**: All inter-service APIs

### 5.4 E2E Tests (Playwright)
- **Scope**: Full user journeys
- **Speed**: 1-2 minutes per test
- **Environment**: Production-like
- **Coverage Target**: 10 critical paths

### 5.5 Performance Tests (k6)
- **Types**: Load, stress, soak, spike
- **Targets**: p95 < 500ms, p99 < 1s
- **Load**: 100 concurrent users minimum

### 5.6 Security Tests
- **Tools**: OWASP ZAP, Snyk, Trivy
- **Scope**: OWASP Top 10, dependency scanning
- **Frequency**: Every PR + nightly

---

## 6. Red-Green-Refactor Cycle

### 6.1 The Three Phases

```
🔴 RED Phase: Write Failing Test (2-5 min)
   ↓
✅ GREEN Phase: Make Test Pass (5-15 min)
   ↓
🔵 REFACTOR Phase: Improve Code (5-10 min)
   ↓
   ↻ REPEAT
```

### 6.2 Phase Rules

**🔴 RED Phase**
- Write ONE behavior test
- Confirm it fails for RIGHT reason
- Time limit: 2-5 minutes

**✅ GREEN Phase**
- Minimal code to pass test
- No gold-plating
- Time limit: 5-15 minutes

**🔵 REFACTOR Phase**
- Improve without changing behavior
- Extract functions, improve names
- Tests stay green
- Time limit: 5-10 minutes

### 6.3 Time Boxing
- Full cycle: 15-30 minutes
- If exceeded, break test into smaller pieces

---

## 7. Test Tooling Stack

### 7.1 Rust Testing

```toml
[dev-dependencies]
tokio = { version = "1.35", features = ["test-util"] }
proptest = "1.4"          # Property-based testing
mockall = "0.12"          # Mocking
criterion = "0.5"         # Benchmarking
testcontainers = "0.15"   # Integration testing
pretty_assertions = "1.4" # Better assertions
```

### 7.2 TypeScript Testing

```json
{
  "devDependencies": {
    "jest": "^29.7.0",
    "@testing-library/react": "^14.1.2",
    "supertest": "^6.3.3",
    "msw": "^2.0.11",
    "@playwright/test": "^1.40.1",
    "@pact-foundation/pact": "^12.1.0",
    "@testcontainers/postgresql": "^10.5.1"
  }
}
```

### 7.3 Performance Testing

- **k6**: Load, stress, soak, spike tests
- **Criterion** (Rust): Micro-benchmarks
- **Grafana**: Metrics visualization

### 7.4 Security Testing

- **OWASP ZAP**: Automated security scanning
- **Snyk**: Dependency vulnerability scanning
- **Trivy**: Container image scanning

---

## 8. Continuous Testing Strategy

### 8.1 Pre-Commit Hooks (Husky)

```json
{
  "husky": {
    "hooks": {
      "pre-commit": "lint-staged",
      "pre-push": "npm run test:unit"
    }
  }
}
```

### 8.2 CI Pipeline (GitHub Actions)

```yaml
jobs:
  unit-tests:
    - Run unit tests (parallel)
    - Upload coverage to Codecov
    - Threshold: ≥80%

  integration-tests:
    - Start Testcontainers
    - Run integration tests
    - Cleanup containers

  e2e-tests:
    - Deploy to staging
    - Run Playwright tests
    - Upload test results

  performance-tests:
    - Run k6 load tests
    - Compare against baseline
    - Fail if >10% regression
```

### 8.3 Test Parallelization

- **Rust**: `cargo test -- --test-threads=8`
- **Jest**: `jest --maxWorkers=50%`
- **Playwright**: `workers: 4`

### 8.4 Flaky Test Management

- Detect: Run tests 10x nightly
- Quarantine: `#[ignore]` with issue link
- Target: <1% flaky test rate

---

## 9. Test Data Management

### 9.1 Fixtures and Factories

```rust
// Rust factory pattern
pub struct TestDataFactory;

impl TestDataFactory {
    pub fn create_user_preferences(user_id: &str) -> UserPreferences {
        UserPreferences {
            user_id: user_id.to_string(),
            vector: vec![0.5, 0.3, 0.2],
            timestamp: Utc::now(),
        }
    }
}
```

```typescript
// TypeScript factory pattern
export class ContentFactory {
  static createContent(overrides?: Partial<Content>): Content {
    return {
      entityId: faker.string.uuid(),
      title: faker.lorem.words(3),
      ...overrides
    };
  }
}
```

### 9.2 Database Seeding

```rust
pub async fn seed_test_database(pool: &PgPool) -> Result<()> {
    sqlx::query!("TRUNCATE TABLE contents, users CASCADE")
        .execute(pool).await?;

    sqlx::query!("INSERT INTO users ...")
        .execute(pool).await?;

    Ok(())
}
```

### 9.3 Test Isolation Strategies

1. **Transaction Rollback**: Fast, automatic cleanup
2. **Separate DB per Test**: Complete isolation
3. **Testcontainers**: New container per test

### 9.4 Cleanup Procedures

```rust
impl Drop for TestContext {
    fn drop(&mut self) {
        // Automatic cleanup on test completion
    }
}
```

---

## 10. Quality Gates & Metrics

### 10.1 Coverage Thresholds

| Metric | Target | Blocker Threshold |
|--------|--------|------------------|
| **Line Coverage** | ≥80% | <75% |
| **Branch Coverage** | ≥80% | <75% |
| **Mutation Score** | ≥70% | <60% |
| **Test Execution Time** | <5 min | >10 min |
| **Flaky Test Rate** | <1% | >5% |

### 10.2 CI Quality Gates

```yaml
# Fail PR if:
- Coverage drops below 75%
- Test execution time >10 minutes
- Any E2E test fails
- Performance regression >25%
- Security vulnerabilities (high/critical)
```

### 10.3 Continuous Monitoring

- **Grafana Dashboards**: Test execution trends
- **Coverage Tracking**: Codecov integration
- **Flaky Test Detection**: Nightly runs
- **Performance Baselines**: k6 trend analysis

---

## Document Status

**This is a PLANNING document defining HOW testing will be implemented.**

**Next Steps:**
1. Review and approve TDD methodology
2. Set up tooling and CI pipelines
3. Begin implementation with first service (Discovery Engine)
4. Train team on TDD practices

**Related Documents:**
- [SPARC Specification Part 1](/workspaces/media-gateway/plans/SPARC_SPECIFICATION_PART_1.md)
- [SPARC Architecture](/workspaces/media-gateway/plans/SPARC_ARCHITECTURE_PART_1.md)
- [SPARC Refinement Part 1](/workspaces/media-gateway/plans/SPARC_REFINEMENT_PART_1.md)

---

**Generated**: 2025-12-06
**Authors**: SPARC Specification Agent
**Version**: 1.0.0
