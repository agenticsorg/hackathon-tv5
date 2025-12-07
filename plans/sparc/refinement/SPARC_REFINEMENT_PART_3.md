# SPARC Refinement — Part 3: Performance Benchmark Specifications

**Document Version:** 1.0.0
**SPARC Phase:** Refinement
**Date:** 2025-12-06
**Status:** Planning

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Performance Targets by Service](#2-performance-targets-by-service)
3. [Load Testing Strategy](#3-load-testing-strategy)
4. [Benchmark Scenarios](#4-benchmark-scenarios)
5. [Resource Budgets](#5-resource-budgets)
6. [Database Performance](#6-database-performance)
7. [Caching Performance](#7-caching-performance)
8. [Network Performance](#8-network-performance)
9. [Benchmark Tooling](#9-benchmark-tooling)
10. [Performance Regression Detection](#10-performance-regression-detection)
11. [Optimization Priorities](#11-optimization-priorities)

---

## 1. Executive Summary

### 1.1 Purpose

This document defines comprehensive performance benchmark specifications for the Media Gateway platform during the SPARC Refinement phase. These benchmarks establish measurable targets, testing methodologies, and regression detection systems to ensure the platform meets its sub-500ms search latency and 99.9% availability requirements.

### 1.2 Scope

**In Scope:**
- Service-level performance targets and SLOs
- Load testing strategies for all traffic patterns
- Benchmark scenarios for critical user journeys
- Resource utilization budgets (CPU, memory, network, storage)
- Database and caching performance metrics
- Automated performance regression detection
- Optimization prioritization framework

**Out of Scope:**
- Actual benchmark implementation code
- Production load testing (requires separate environment)
- Capacity planning for >1M users (Phase 2)
- Client-side performance optimization

### 1.3 Document Purpose

This is a **PLANNING document** that defines:
- What to measure (metrics)
- When to measure (scenarios)
- How to measure (tooling)
- Acceptable thresholds (targets)

The actual benchmark implementations will be created during TDD implementation sprints.

---

## 2. Performance Targets by Service

### 2.1 API Gateway

| Metric | Target | Maximum | Measurement Method |
|--------|--------|---------|-------------------|
| **Latency (p50)** | 20ms | 50ms | Response time from client request to first byte |
| **Latency (p95)** | 50ms | 100ms | 95th percentile response time |
| **Latency (p99)** | 100ms | 200ms | 99th percentile response time |
| **Throughput** | 5,000 RPS | - | Requests per second sustained for 5 minutes |
| **Error Rate** | <0.1% | 1% | HTTP 5xx responses / total requests |
| **Availability** | 99.9% | - | Uptime measured over 30-day window |

**Rationale:**
- API Gateway is the entry point; latency here impacts all downstream services
- 20ms p50 target leaves 280ms budget for downstream services (300ms total search goal)
- 5,000 RPS supports 100K concurrent users with 3-5 req/min average

**Testing Conditions:**
- Load distributed across all endpoints (/api/search, /api/recommendations, /mcp/*)
- 30% authenticated, 70% anonymous traffic
- Regional distribution: 80% US, 10% EU, 10% APAC

---

### 2.2 Discovery Service (Search)

| Metric | Target | Maximum | Measurement Method |
|--------|--------|---------|-------------------|
| **Latency (p50)** | 150ms | 300ms | Time from query received to results returned |
| **Latency (p95)** | 400ms | 600ms | 95th percentile end-to-end latency |
| **Latency (p99)** | 600ms | 1000ms | 99th percentile latency |
| **Throughput** | 2,000 RPS | - | Search queries per second |
| **Cache Hit Rate** | >40% | - | Queries served from Redis cache |
| **Vector Search Latency** | <50ms | 100ms | Qdrant HNSW search time (p95) |
| **Availability** | 99.9% | - | Service uptime |

**Rationale:**
- 150ms p50 allows for: NL parsing (85ms) + embedding (25ms) + vector search (40ms)
- Cache hit rate of 40% reduces load on vector DB by 40%
- 2,000 RPS = 40% of total API Gateway traffic (search is primary use case)

**Testing Conditions:**
- Mix of short queries (3-5 words) and long queries (10-20 words)
- 50% exact matches (cached), 50% novel queries
- Includes availability filtering and SONA re-ranking

**Decomposition:**
```
Search Request Breakdown (Target: 150ms p50)
├─ API Gateway routing: 5ms
├─ Authentication: 3ms
├─ NL query parsing (GPT-4o-mini): 85ms
├─ Query embedding generation: 25ms
├─ Qdrant vector search (HNSW): 40ms
├─ Availability filtering (PostgreSQL): 15ms
├─ SONA re-ranking: 20ms
├─ Response serialization: 7ms
└─ Network overhead: 10ms
Total: 210ms (60ms buffer for p95)
```

---

### 2.3 Recommendation Service (SONA)

| Metric | Target | Maximum | Measurement Method |
|--------|--------|---------|-------------------|
| **Personalization Latency (p50)** | 5ms | 20ms | Time to score 100 candidates with user LoRA |
| **Personalization Latency (p95)** | 15ms | 50ms | 95th percentile inference time |
| **LoRA Load Time** | <10ms | 30ms | Time to load user adapter into memory |
| **Throughput** | 1,500 RPS | - | Recommendation requests per second |
| **Model Accuracy** | >80% CTR | - | Click-through rate on top 10 recommendations |
| **Memory per User** | 1MB | 2MB | LoRA adapter size in memory |
| **Availability** | 99.9% | - | Service uptime |

**Rationale:**
- 5ms personalization keeps SONA overhead minimal in search path
- LoRA adapters must fit in memory (10K users × 1MB = 10GB per replica)
- 1,500 RPS = 30% of total API traffic (recommendations are secondary use case)

**Testing Conditions:**
- 70% warm cache (LoRA already loaded), 30% cold cache (load from Redis)
- Candidate pool sizes: 50, 100, 500 items
- Mix of new users (global LoRA) and established users (personalized LoRA)

**SONA Architecture Constraints:**
```
SONA Memory Budget (per replica)
├─ Base model (frozen): 400MB
├─ Global LoRA (trending): 4MB
├─ User LoRA cache (10K users × 1MB): 10GB
├─ ONNX runtime overhead: 100MB
├─ OS/system overhead: 500MB
└─ Total per replica: ~11GB
Target: n1-highmem-4 (26GB RAM, supports 2 replicas per node)
```

---

### 2.4 Sync Service

| Metric | Target | Maximum | Measurement Method |
|--------|--------|---------|-------------------|
| **Cross-Device Latency (p50)** | 50ms | 100ms | Time from action on Device A to update on Device B |
| **Cross-Device Latency (p95)** | 100ms | 200ms | 95th percentile sync latency |
| **WebSocket Connection Setup** | <200ms | 500ms | Time to establish connection and subscribe |
| **CRDT Operation Size** | <500 bytes | 1KB | Serialized CRDT operation payload |
| **Concurrent Connections** | 10,000 | - | WebSocket connections per replica |
| **PubNub Latency** | <50ms | 100ms | Message delivery time via PubNub network |
| **Availability** | 99.5% | - | Service uptime |

**Rationale:**
- 50ms sync latency creates seamless cross-device experience
- 10,000 connections per replica supports 100K users with 10 replicas
- CRDT operations kept small for network efficiency

**Testing Conditions:**
- Synchronization of watchlist adds, removes, and progress updates
- 2-5 devices per user (typical household)
- Network conditions: 50% WiFi (low latency), 30% LTE (medium), 20% 3G (high)

**Sync Flow Breakdown:**
```
Cross-Device Sync Flow (Target: 50ms p50)
├─ Device A: Local CRDT operation: 2ms
├─ Device A: Serialize and publish to PubNub: 5ms
├─ PubNub network transmission: 40ms
├─ Device B: Receive from PubNub: 5ms
├─ Device B: Deserialize and merge CRDT: 3ms
├─ Device B: UI update: 5ms
└─ Total: 60ms (10ms buffer for p95)
```

---

### 2.5 Auth Service

| Metric | Target | Maximum | Measurement Method |
|--------|--------|---------|-------------------|
| **Token Validation (p50)** | 5ms | 15ms | JWT verification time |
| **Token Validation (p95)** | 10ms | 25ms | 95th percentile validation time |
| **Login Flow (OAuth)** | <2s | 5s | End-to-end OAuth PKCE flow |
| **Device Pairing** | <30s | 60s | Time from code generation to device authenticated |
| **Token Refresh** | <100ms | 300ms | Time to rotate refresh token |
| **Throughput** | 3,000 RPS | - | Token validations per second |
| **Availability** | 99.9% | - | Service uptime |

**Rationale:**
- 5ms validation is negligible overhead in request path
- 2s OAuth flow meets user expectation for login
- 3,000 RPS = every request requires token validation

**Testing Conditions:**
- 90% token validations (cached public keys), 10% token refreshes
- Mix of web (PKCE), mobile (PKCE), and TV (device grant) flows
- Redis session store at 80% capacity

---

### 2.6 Ingestion Service

| Metric | Target | Maximum | Measurement Method |
|--------|--------|---------|-------------------|
| **Ingestion Rate** | 10,000 items/hour | - | Content items processed per hour |
| **Platform API Latency** | <500ms | 2s | Average response time from external APIs |
| **Embedding Generation** | 500 items/second | - | Batch embedding throughput |
| **Entity Resolution** | <100ms | 300ms | Deduplication time per item |
| **Database Write Latency** | <50ms | 150ms | PostgreSQL + Qdrant upsert time |
| **Error Rate** | <1% | 5% | Failed ingestions / total items |
| **Data Freshness** | <1 hour | 2 hours | Time from platform update to searchable |

**Rationale:**
- 10,000 items/hour supports 150+ platforms with hourly refresh
- 1-hour freshness meets user expectation for new content discovery
- 1% error rate acceptable for non-critical metadata updates

**Testing Conditions:**
- Parallel ingestion from 5 platforms simultaneously
- Mix of new items (inserts) and updates (upserts)
- Rate limiting and circuit breaker patterns active

---

### 2.7 MCP Server

| Metric | Target | Maximum | Measurement Method |
|--------|--------|---------|-------------------|
| **MCP Tool Invocation Overhead** | <10ms | 50ms | Time from MCP request to downstream API call |
| **MCP Tool Latency (total)** | <500ms | 2s | End-to-end tool execution time |
| **Token Efficiency** | 85% reduction | - | Tokens used vs HTML scraping baseline |
| **Concurrent MCP Sessions** | 1,000 | - | Simultaneous Claude/GPT-4 sessions |
| **Throughput** | 500 RPS | - | MCP tool invocations per second |
| **Availability** | 99.9% | - | Service uptime |

**Rationale:**
- 10ms overhead ensures MCP doesn't add significant latency
- 85% token reduction is ARW specification target
- 1,000 sessions supports enterprise AI agent use cases

**Testing Conditions:**
- Mix of read operations (90%) and write operations (10%)
- STDIO and SSE transport protocols
- OAuth-protected actions with token validation

---

## 3. Load Testing Strategy

### 3.1 Baseline Testing (Normal Load)

**Objective:** Establish performance baseline under expected production load

**Configuration:**
```yaml
Load Profile: Normal
- Duration: 30 minutes
- Concurrent Users: 10,000
- Request Rate: 1,000 RPS (average)
- Traffic Mix:
  - Search: 60%
  - Recommendations: 20%
  - Watchlist operations: 10%
  - Content details: 10%
- User Behavior:
  - Think time: 5-10 seconds
  - Session duration: 10 minutes
  - Authenticated: 30%
```

**Success Criteria:**
- All p95 latencies within target thresholds
- CPU utilization <70% average
- Memory utilization <80% average
- Error rate <0.1%
- No pod restarts or OOM kills

**Measurement:**
- Latency histograms (p50, p95, p99) per endpoint
- Throughput (RPS) per service
- Resource utilization (CPU, memory, network) per pod
- Database connection pool usage
- Cache hit rates

---

### 3.2 Stress Testing (2x Expected Load)

**Objective:** Validate system behavior under high load and autoscaling

**Configuration:**
```yaml
Load Profile: Stress
- Duration: 60 minutes
- Concurrent Users: 20,000
- Request Rate: 2,000 RPS (average), 3,500 RPS (peak)
- Traffic Mix: Same as baseline
- Ramp-up: 0 → 20K users over 10 minutes
- Sustained: 20K users for 40 minutes
- Ramp-down: 20K → 0 over 10 minutes
```

**Success Criteria:**
- All p95 latencies within **maximum** thresholds
- Horizontal Pod Autoscaler (HPA) triggers within 2 minutes
- New pods ready within 5 minutes
- CPU utilization <90% peak
- Error rate <1%
- Graceful degradation if limits reached

**Measurement:**
- Autoscaling metrics (pod count over time)
- Queue depths (if applicable)
- Circuit breaker activations
- Cache eviction rates
- Database slow query counts

---

### 3.3 Spike Testing (Sudden 10x Load)

**Objective:** Test resilience against sudden traffic surges (e.g., viral content)

**Configuration:**
```yaml
Load Profile: Spike
- Duration: 20 minutes
- Concurrent Users: 0 → 100,000 → 0
- Request Rate: 100 RPS → 10,000 RPS → 100 RPS
- Traffic Mix: 90% search (viral query), 10% other
- Spike Pattern:
  - Baseline: 100 RPS for 5 minutes
  - Spike: 10,000 RPS instant surge
  - Sustained: 10,000 RPS for 10 minutes
  - Recovery: Drop to 100 RPS
```

**Success Criteria:**
- System survives spike without cascading failures
- Rate limiting activated within 1 second
- Error rate <5% during spike
- Recovery to normal within 5 minutes after spike ends
- No data loss or corruption

**Measurement:**
- Rate limiter triggers and 429 responses
- Circuit breaker state transitions
- Database connection pool saturation
- Redis memory evictions
- PagerDuty alert count

---

### 3.4 Soak Testing (Sustained Load 24 Hours)

**Objective:** Detect memory leaks, resource exhaustion, and performance degradation over time

**Configuration:**
```yaml
Load Profile: Soak
- Duration: 24 hours
- Concurrent Users: 15,000 (constant)
- Request Rate: 1,500 RPS (constant)
- Traffic Mix: Same as baseline
- No ramp-up/down
```

**Success Criteria:**
- Latency does NOT degrade over 24 hours (max 10% increase)
- Memory usage stabilizes (no continuous growth)
- No pod restarts due to OOM
- Error rate remains <0.1%
- Database connection leaks: 0

**Measurement:**
- Latency trend over 24 hours (hourly p95)
- Memory growth rate (MB/hour)
- Goroutine/thread count trend (Rust async tasks)
- Database connection pool usage over time
- Redis memory usage trend

---

### 3.5 Chaos Testing (Failure Injection)

**Objective:** Validate resilience patterns (circuit breakers, retries, graceful degradation)

**Configuration:**
```yaml
Load Profile: Chaos
- Duration: 60 minutes
- Concurrent Users: 10,000
- Request Rate: 1,000 RPS
- Failure Injections:
  - 10min: Kill 1 Discovery Service pod
  - 20min: Network delay to Qdrant (+200ms)
  - 30min: PostgreSQL read replica failure
  - 40min: Redis cache flush
  - 50min: PubNub network partition
```

**Success Criteria:**
- Zero downtime (availability maintained)
- Automatic recovery within 2 minutes per failure
- Circuit breakers activate appropriately
- Graceful degradation to cached data
- Error rate <5% during failures, <0.5% after recovery

**Measurement:**
- Pod restart count and timing
- Circuit breaker state changes
- Fallback activation count
- Cache miss rate after Redis flush
- Sync latency after PubNub partition

---

## 4. Benchmark Scenarios

### 4.1 Scenario 1: Natural Language Search

**User Journey:**
```
User opens app → Types "scary movies like Stranger Things" → Receives results
```

**Benchmark Steps:**
1. **API Gateway:** Route `/api/search` request → 5ms
2. **Auth Service:** Validate JWT (if authenticated) → 5ms
3. **Discovery Service:**
   - Parse natural language query → 85ms
   - Generate query embedding (768-dim) → 25ms
   - Vector search in Qdrant (HNSW, top 100) → 40ms
   - Filter by user subscriptions (PostgreSQL) → 15ms
4. **SONA Service:** Re-rank top 100 with personalization → 20ms
5. **API Gateway:** Return JSON response → 5ms

**Expected Latency:** 200ms (p50), 400ms (p95)

**Load Pattern:**
- 1,000 concurrent searches
- 50% unique queries (cache miss), 50% repeated queries (cache hit)
- Query complexity: 3-20 words

**Success Criteria:**
- p50 latency <300ms
- p95 latency <600ms
- Cache hit rate >40%

---

### 4.2 Scenario 2: Personalized Recommendations

**User Journey:**
```
User opens "For You" tab → Receives 20 personalized recommendations
```

**Benchmark Steps:**
1. **API Gateway:** Route `/api/recommendations` request → 5ms
2. **Auth Service:** Validate JWT → 5ms
3. **SONA Service:**
   - Load user LoRA adapter (if not cached) → 10ms
   - Retrieve candidate pool (top 500 popular items) → 15ms
   - Score candidates with SONA model → 20ms
   - Apply context filters (time, device, mood) → 5ms
   - Sort and return top 20 → 5ms

**Expected Latency:** 65ms (p50), 100ms (p95)

**Load Pattern:**
- 500 concurrent recommendation requests
- 70% warm LoRA cache, 30% cold cache
- Mix of new users (global LoRA) and established users (personalized)

**Success Criteria:**
- p50 latency <100ms
- p95 latency <200ms
- LoRA cache hit rate >70%

---

### 4.3 Scenario 3: Cross-Device Sync

**User Journey:**
```
User adds item to watchlist on phone → Item appears on TV within 100ms
```

**Benchmark Steps:**
1. **Device A (Phone):**
   - User taps "Add to Watchlist" → 0ms (client action)
   - Generate CRDT operation (OR-Set add) → 2ms
   - Serialize operation → 3ms
   - Publish to PubNub channel → 5ms
2. **PubNub Network:** Transmit message → 40ms
3. **Device B (TV):**
   - Receive message from PubNub → 5ms
   - Deserialize CRDT operation → 3ms
   - Merge with local state → 2ms
   - Update UI → 10ms

**Expected Latency:** 70ms (p50), 100ms (p95)

**Load Pattern:**
- 10,000 devices syncing simultaneously
- 5 operations per minute per user
- Network conditions: 50% WiFi, 30% LTE, 20% 3G

**Success Criteria:**
- p50 latency <100ms
- p95 latency <200ms
- PubNub message delivery success rate >99.9%

---

### 4.4 Scenario 4: User Authentication

**User Journey:**
```
New user clicks "Login with Google" → Completes OAuth flow → Authenticated
```

**Benchmark Steps:**
1. **Web App:** Redirect to Google OAuth → 50ms (network)
2. **Google:** User authorizes → 1,500ms (user action)
3. **Auth Service:**
   - Receive authorization code → 10ms
   - Exchange code for tokens (PKCE validation) → 200ms (Google API)
   - Create session in Redis → 10ms
   - Issue JWT access token → 5ms
   - Return to client → 10ms

**Expected Latency:** 1,785ms (p50), 3,000ms (p95)

**Load Pattern:**
- 100 concurrent login attempts (realistic for 10K DAU)
- Mix of Google, GitHub OAuth providers
- 30% mobile, 70% web

**Success Criteria:**
- p50 latency <2s
- p95 latency <5s
- OAuth callback success rate >99%

---

### 4.5 Scenario 5: Content Ingestion

**User Journey:**
```
New movie released → Ingestion service fetches metadata → Searchable within 1 hour
```

**Benchmark Steps:**
1. **Ingestion Service:**
   - Fetch from platform API (e.g., YouTube Data API) → 300ms
   - Normalize to CanonicalContent schema → 20ms
   - Entity resolution (deduplication) → 50ms
   - Generate embedding (batch of 100) → 200ms
   - Upsert to PostgreSQL → 30ms
   - Upsert to Qdrant → 40ms
   - Publish event to Pub/Sub → 10ms

**Expected Latency:** 650ms per item (p50), 1,000ms (p95)

**Load Pattern:**
- 10,000 items per hour (2.8 items/second)
- Parallel ingestion from 5 platforms
- 70% updates (existing items), 30% inserts (new items)

**Success Criteria:**
- Ingestion rate >10,000 items/hour
- End-to-end data freshness <1 hour
- Error rate <1%

---

## 5. Resource Budgets

### 5.1 CPU Utilization

| Service | Average Target | Peak Maximum | Trigger Autoscaling |
|---------|----------------|--------------|---------------------|
| **API Gateway** | <50% | <90% | >70% for 2 minutes |
| **Discovery Service** | <60% | <90% | >70% for 2 minutes |
| **SONA Engine** | <70% | <95% | >80% for 2 minutes |
| **Sync Service** | <50% | <85% | >70% for 2 minutes |
| **Auth Service** | <40% | <80% | >60% for 2 minutes |
| **Ingestion Service** | <80% | <95% | >85% for 5 minutes |
| **MCP Server** | <40% | <80% | >60% for 2 minutes |

**Rationale:**
- 70% average threshold prevents saturation and leaves headroom for spikes
- Autoscaling at 70% ensures new pods ready before hitting 90% peak
- Ingestion service allowed higher CPU (batch processing workload)

**Monitoring:**
```prometheus
# Alert when CPU sustained >80% for 5 minutes
ALERT HighCPUUsage
  IF rate(container_cpu_usage_seconds_total[5m]) > 0.8
  FOR 5m
  LABELS { severity="warning" }
  ANNOTATIONS {
    summary="High CPU usage on {{ $labels.pod }}",
    description="CPU has been above 80% for 5 minutes"
  }
```

---

### 5.2 Memory Utilization

| Service | Allocated RAM | Average Target | Peak Maximum | OOM Kill Threshold |
|---------|---------------|----------------|--------------|-------------------|
| **API Gateway** | 2GB | <1.2GB (60%) | <1.6GB (80%) | 2GB |
| **Discovery Service** | 4GB | <2.8GB (70%) | <3.2GB (80%) | 4GB |
| **SONA Engine** | 16GB | <12GB (75%) | <14GB (87%) | 16GB |
| **Sync Service** | 2GB | <1.4GB (70%) | <1.6GB (80%) | 2GB |
| **Auth Service** | 1GB | <600MB (60%) | <800MB (80%) | 1GB |
| **Ingestion Service** | 4GB | <2.8GB (70%) | <3.2GB (80%) | 4GB |
| **MCP Server** | 2GB | <1.2GB (60%) | <1.6GB (80%) | 2GB |

**Rationale:**
- SONA Engine requires 16GB for LoRA adapters (10K users × 1MB + base model)
- 80% threshold prevents OOM kills while maximizing utilization
- Rust services have predictable memory (no GC), lower risk of leaks

**Monitoring:**
```prometheus
# Alert when memory growth indicates leak
ALERT MemoryLeak
  IF rate(container_memory_usage_bytes[1h]) > 10485760  # 10MB/hour growth
  FOR 6h
  LABELS { severity="warning" }
  ANNOTATIONS {
    summary="Possible memory leak in {{ $labels.pod }}",
    description="Memory growing at {{ $value }} bytes/hour"
  }
```

---

### 5.3 Network Utilization

| Service | Bandwidth Target | Peak Maximum | Error Rate | Timeout Rate |
|---------|------------------|--------------|------------|--------------|
| **API Gateway** | <50Mbps | <200Mbps | <0.1% | <0.01% |
| **Discovery Service** | <30Mbps | <100Mbps | <0.1% | <0.01% |
| **SONA Engine** | <10Mbps | <30Mbps | <0.1% | <0.01% |
| **Sync Service** | <100Mbps | <500Mbps | <0.5% | <0.05% |
| **Auth Service** | <5Mbps | <20Mbps | <0.1% | <0.01% |
| **Ingestion Service** | <20Mbps | <100Mbps | <1% | <0.1% |

**Rationale:**
- Sync Service highest bandwidth (WebSocket connections + PubNub traffic)
- 0.1% error rate accounts for transient network issues
- Ingestion Service higher error tolerance (external API flakiness)

**Network Efficiency:**
```
Payload Size Budgets
├─ API Gateway → Client: <50KB per response
├─ Discovery Service: JSON response ~25KB (25 results × 1KB each)
├─ SONA Engine: JSON response ~10KB (20 recommendations)
├─ Sync Service: CRDT operation <500 bytes
└─ MCP Server: Tool response ~5KB (ARW structured data)
```

---

### 5.4 Database Connection Pools

| Database | Pool Size | Target Utilization | Alert Threshold |
|----------|-----------|-------------------|-----------------|
| **PostgreSQL (Primary)** | 100 connections | <70 (70%) | >80 (80%) |
| **PostgreSQL (Read Replicas)** | 300 connections (100 each × 3) | <210 (70%) | >240 (80%) |
| **Redis** | 50 connections per service | <35 (70%) | >40 (80%) |
| **Qdrant** | 20 connections per service | <14 (70%) | >16 (80%) |

**Rationale:**
- 70% utilization prevents connection starvation
- PostgreSQL connection pooling via PgBouncer reduces overhead
- Read replicas handle 80% of queries (read-heavy workload)

**Connection Leak Detection:**
```sql
-- PostgreSQL: Find long-running idle connections
SELECT pid, usename, state, state_change, now() - state_change AS idle_time
FROM pg_stat_activity
WHERE state = 'idle in transaction'
  AND now() - state_change > interval '5 minutes';
```

---

## 6. Database Performance

### 6.1 Query Latency Targets

| Query Type | p50 Target | p95 Target | p99 Target | Slow Query Threshold |
|------------|------------|------------|------------|----------------------|
| **Primary Key Lookup** | <5ms | <10ms | <20ms | 50ms |
| **Indexed Search** | <20ms | <50ms | <100ms | 200ms |
| **Join Query (2 tables)** | <30ms | <80ms | <150ms | 300ms |
| **Aggregation** | <50ms | <150ms | <300ms | 500ms |
| **Full-Text Search** | <100ms | <300ms | <600ms | 1000ms |

**Rationale:**
- Primary key lookups are cache-backed (Redis), sub-10ms expected
- Indexed searches cover most queries (user_id, content_id)
- Slow query threshold triggers optimization review

**Example Queries:**
```sql
-- Primary Key Lookup (p50: <5ms)
SELECT * FROM content WHERE id = $1;

-- Indexed Search (p50: <20ms)
SELECT * FROM watchlist WHERE user_id = $1 ORDER BY added_at DESC LIMIT 50;

-- Join Query (p50: <30ms)
SELECT c.*, ca.platform, ca.region
FROM content c
JOIN content_availability ca ON c.id = ca.content_id
WHERE ca.user_subscriptions && $1::text[];

-- Aggregation (p50: <50ms)
SELECT platform, COUNT(*) as count
FROM content_availability
WHERE region = 'US'
GROUP BY platform;
```

---

### 6.2 Connection Pool Metrics

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| **Connection Pool Utilization** | <70% | >80% |
| **Connection Wait Time** | <10ms | >50ms |
| **Connection Acquisition Timeout** | <100ms | >500ms |
| **Idle Connections** | <30% of pool | >50% of pool |
| **Connection Errors** | <0.1% | >1% |

**PgBouncer Configuration:**
```ini
[databases]
media_gateway = host=127.0.0.1 port=5432 dbname=media_gateway

[pgbouncer]
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 100
reserve_pool_size = 20
reserve_pool_timeout = 5
server_idle_timeout = 600
server_lifetime = 3600
```

---

### 6.3 Index Performance

**Required Indexes:**
```sql
-- Discovery Service: Content search
CREATE INDEX idx_content_title_trgm ON content USING gin (title gin_trgm_ops);
CREATE INDEX idx_content_type ON content (type);

-- Watchlist queries
CREATE INDEX idx_watchlist_user_id ON watchlist (user_id, added_at DESC);

-- Availability filtering
CREATE INDEX idx_availability_content_region ON content_availability (content_id, region);

-- User lookups
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_oauth_provider ON users (oauth_provider, oauth_sub);

-- Audit logs (time-series)
CREATE INDEX idx_audit_logs_timestamp ON audit_logs (timestamp DESC);
CREATE INDEX idx_audit_logs_user_action ON audit_logs (user_id, action, timestamp DESC);
```

**Index Hit Ratio Target:** >99%

**Monitoring:**
```sql
-- Check index hit ratio (should be >99%)
SELECT
  schemaname,
  tablename,
  100 * idx_scan / (seq_scan + idx_scan) AS index_hit_ratio
FROM pg_stat_user_tables
WHERE (seq_scan + idx_scan) > 0
ORDER BY index_hit_ratio ASC;
```

---

### 6.4 Replication Lag

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| **Replication Lag (Read Replicas)** | <100ms | >500ms |
| **WAL Apply Rate** | >1MB/s | <500KB/s |
| **Replica Downtime** | 0 seconds | >60 seconds |

**Monitoring:**
```sql
-- Check replication lag on primary
SELECT
  client_addr,
  state,
  sync_state,
  pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) AS lag_bytes,
  EXTRACT(EPOCH FROM (now() - replay_time)) AS lag_seconds
FROM pg_stat_replication;
```

---

## 7. Caching Performance

### 7.1 Multi-Tier Cache Strategy

#### Layer 1: Client-Side Cache

| Content Type | Cache Duration | Invalidation Strategy |
|--------------|----------------|----------------------|
| **Static Assets (JS, CSS, images)** | 1 year | Immutable (content-hash filenames) |
| **API Responses (search)** | 5 seconds | Revalidate on stale |
| **User Profile** | 5 minutes | Event-driven invalidation |

**Implementation:**
```http
# Static assets
Cache-Control: public, max-age=31536000, immutable

# API responses
Cache-Control: public, max-age=5, stale-while-revalidate=30

# User profile
Cache-Control: private, max-age=300
```

---

#### Layer 2: API Gateway Cache (Cloud Run)

| Content Type | Cache Duration | Cache Size | Hit Rate Target |
|--------------|----------------|------------|-----------------|
| **Popular Search Queries** | 30 seconds | 1GB | >40% |
| **Content Details** | 5 minutes | 2GB | >60% |
| **Platform Availability** | 5 minutes | 500MB | >70% |

**Implementation:**
```typescript
// Fastify caching middleware
fastify.register(require('@fastify/caching'), {
  privacy: 'public',
  expiresIn: 30, // seconds
  cache: {
    max: 10000, // 10K entries
    ttl: 30000  // 30 seconds
  }
});
```

---

#### Layer 3: Redis Cache (Memorystore)

| Content Type | Cache Duration | Memory Budget | Eviction Policy |
|--------------|----------------|---------------|-----------------|
| **User Sessions** | 7 days | 2GB | LRU (least recently used) |
| **Content Metadata** | 5 minutes | 1.5GB | LRU |
| **Rate Limit Counters** | 1 minute | 500MB | TTL expiration |
| **User LoRA Adapters** | 1 hour | 500MB | LRU |

**Total Redis Memory:** 5GB HA instance

**Hit Rate Targets:**
- **L1 (Client):** >40%
- **L2 (API Gateway):** >85%
- **L3 (Redis):** >90%

**Monitoring:**
```redis
# Redis: Check hit rate
INFO stats
# Look for:
# keyspace_hits:1000000
# keyspace_misses:100000
# Hit rate = 1000000 / (1000000 + 100000) = 90.9%
```

---

### 7.2 Cache Performance Metrics

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| **Cache Hit Rate (L1)** | >40% | <30% |
| **Cache Hit Rate (L2)** | >85% | <70% |
| **Cache Hit Rate (L3)** | >90% | <80% |
| **Cache Latency (Redis)** | <5ms | >10ms |
| **Cache Eviction Rate** | <5%/hour | >10%/hour |
| **Cache Memory Usage** | <75% allocated | >90% allocated |

---

### 7.3 Cache Invalidation Strategy

**Event-Driven Invalidation:**
```typescript
// Pub/Sub event when content updated
pubsub.subscribe('content.updated', async (contentId) => {
  // Invalidate all cache layers
  await redis.del(`content:${contentId}`);
  await redis.del(`search:*`); // Wildcard invalidation for search results

  // Publish to client via WebSocket
  websocket.broadcast({
    type: 'CACHE_INVALIDATE',
    key: `content:${contentId}`
  });
});
```

**Lazy Invalidation (Time-to-Live):**
```typescript
// Set TTL on cache entries
await redis.setex(`content:${id}`, 300, JSON.stringify(content)); // 5 minutes

// Check last_updated timestamp on cache hit
const cached = await redis.get(`content:${id}`);
if (cached && cached.last_updated < content.last_updated) {
  // Stale cache, refetch
  return fetchFromDatabase(id);
}
```

---

## 8. Network Performance

### 8.1 Bandwidth Utilization

| Network Path | Average Target | Peak Maximum | Error Rate | Retry Rate |
|--------------|----------------|--------------|------------|------------|
| **Client → Cloud LB** | <30Mbps | <200Mbps | <0.1% | <1% |
| **LB → API Gateway** | <50Mbps | <300Mbps | <0.05% | <0.5% |
| **Service ↔ Service (gRPC)** | <20Mbps | <100Mbps | <0.01% | <0.1% |
| **Service → PostgreSQL** | <10Mbps | <50Mbps | <0.01% | <0.5% |
| **Service → Redis** | <5Mbps | <30Mbps | <0.01% | <0.1% |
| **Service → Qdrant** | <15Mbps | <80Mbps | <0.05% | <0.5% |
| **PubNub Network** | <100Mbps | <500Mbps | <0.1% | <1% |

**Rationale:**
- Internal network (GKE) has low error rates (<0.01%)
- External APIs (PubNub) higher error tolerance (0.1%)
- Retry rate <1% prevents retry storms

---

### 8.2 Network Latency Budget

| Network Hop | Target Latency | Maximum Latency |
|-------------|----------------|-----------------|
| **Client → Cloud LB** | <20ms (US) | <100ms (global) |
| **LB → API Gateway** | <2ms | <10ms |
| **API Gateway → Service** | <3ms | <10ms |
| **Service ↔ Service (gRPC)** | <2ms | <10ms |
| **Service → PostgreSQL** | <2ms | <10ms |
| **Service → Redis** | <1ms | <5ms |
| **Service → Qdrant** | <2ms | <10ms |
| **PubNub Message Delivery** | <50ms | <100ms |

**Total Network Budget (End-to-End):**
```
Client Request → Response
├─ Client → Cloud LB: 20ms
├─ LB → API Gateway: 2ms
├─ API Gateway → Discovery: 3ms
├─ Discovery → Qdrant: 2ms
├─ Discovery → PostgreSQL: 2ms
├─ Discovery → SONA: 2ms
├─ SONA → Response: 2ms
├─ API Gateway → Client: 20ms
└─ Total Network Overhead: ~53ms

Remaining for processing: 247ms (to meet 300ms target)
```

---

### 8.3 Timeout Configuration

| Service Call | Timeout | Retry Policy |
|--------------|---------|--------------|
| **Client → API Gateway** | 30s | None (client-side retry) |
| **API Gateway → Service** | 10s | None (fail fast) |
| **Service → Database** | 5s | 3 retries, exponential backoff |
| **Service → Redis** | 1s | 2 retries, linear backoff |
| **Service → Qdrant** | 3s | 2 retries, exponential backoff |
| **Service → External API** | 10s | 5 retries, exponential backoff + circuit breaker |

**Rationale:**
- Client timeout (30s) prevents hung connections
- Database timeout (5s) allows for slow queries but prevents indefinite waits
- External API timeout (10s) accounts for network variability

---

### 8.4 Payload Size Optimization

| API Endpoint | Average Size | Maximum Size | Compression |
|--------------|--------------|--------------|-------------|
| **POST /api/search (request)** | 200 bytes | 1KB | None (small) |
| **POST /api/search (response)** | 25KB | 100KB | gzip (70% reduction) |
| **GET /api/content/:id** | 5KB | 20KB | gzip |
| **POST /api/recommendations** | 10KB | 30KB | gzip |
| **WebSocket /sync (message)** | 500 bytes | 2KB | None (small) |
| **MCP Tool Response** | 5KB | 50KB | gzip |

**Compression Strategy:**
```http
# Enable gzip compression for responses >1KB
Accept-Encoding: gzip, deflate, br
Content-Encoding: gzip
```

---

## 9. Benchmark Tooling

### 9.1 Load Testing Tools

#### k6 (Primary Load Testing)

**Purpose:** HTTP/WebSocket load testing for all services

**Installation:**
```bash
# Install k6
brew install k6

# Or via Docker
docker pull grafana/k6
```

**Example k6 Script:**
```javascript
// k6-search-benchmark.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '5m', target: 1000 },  // Ramp-up to 1000 VUs
    { duration: '10m', target: 1000 }, // Sustain 1000 VUs
    { duration: '5m', target: 0 },     // Ramp-down
  ],
  thresholds: {
    http_req_duration: ['p(95)<400', 'p(99)<600'], // 95% <400ms, 99% <600ms
    http_req_failed: ['rate<0.01'],                 // Error rate <1%
  },
};

export default function () {
  const queries = [
    'scary movies like Stranger Things',
    'romantic comedies',
    'action movies with cars',
  ];

  const query = queries[Math.floor(Math.random() * queries.length)];

  const res = http.post('https://media-gateway.example.com/api/search', JSON.stringify({
    query: query,
    limit: 25,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time <400ms': (r) => r.timings.duration < 400,
    'results returned': (r) => JSON.parse(r.body).results.length > 0,
  });

  sleep(5); // Think time
}
```

**Running k6:**
```bash
# Run locally
k6 run k6-search-benchmark.js

# Run in cloud (k6 Cloud)
k6 cloud k6-search-benchmark.js

# Output to Grafana
k6 run --out influxdb=http://localhost:8086/k6 k6-search-benchmark.js
```

---

#### Grafana (Visualization)

**Purpose:** Real-time visualization of k6 metrics

**Setup:**
```bash
# Install Grafana
docker run -d -p 3000:3000 grafana/grafana

# Import k6 dashboard
# Dashboard ID: 2587 (k6 Load Testing Results)
```

**Key Dashboards:**
- **HTTP Request Duration:** p50, p95, p99 latencies
- **Request Rate:** RPS over time
- **Error Rate:** Failed requests / total requests
- **Virtual Users:** Active VU count

---

#### Custom Rust Benchmarks (Micro-benchmarks)

**Purpose:** Benchmark hot code paths in Rust services

**Installation:**
```bash
# criterion.rs (Rust benchmarking)
cargo add --dev criterion
```

**Example Benchmark:**
```rust
// benches/sona_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use media_gateway::sona::SONAEngine;

fn benchmark_sona_scoring(c: &mut Criterion) {
    let engine = SONAEngine::new();
    let user_lora = engine.load_user_lora("user-123");
    let candidates = vec![/* 100 content items */];

    c.bench_function("sona_score_100_candidates", |b| {
        b.iter(|| {
            engine.score_candidates(black_box(&user_lora), black_box(&candidates))
        });
    });
}

criterion_group!(benches, benchmark_sona_scoring);
criterion_main!(benches);
```

**Running:**
```bash
cargo bench --bench sona_benchmark
```

**Output:**
```
sona_score_100_candidates
                        time:   [4.8 ms 5.1 ms 5.4 ms]
                        change: [-2.5% +0.3% +3.2%]
```

---

### 9.2 Database Benchmarking

#### pgbench (PostgreSQL)

**Purpose:** Benchmark PostgreSQL throughput and latency

**Installation:**
```bash
# Included with PostgreSQL
sudo apt-get install postgresql-contrib
```

**Example:**
```bash
# Initialize test database
pgbench -i -s 50 media_gateway

# Run benchmark: 10 clients, 100 transactions each
pgbench -c 10 -t 100 media_gateway

# Custom script (read-heavy workload)
echo "SELECT * FROM content WHERE id = random_int(1, 1000000);" > read-workload.sql
pgbench -c 50 -t 1000 -f read-workload.sql media_gateway
```

**Output:**
```
transaction type: <builtin: TPC-B (sort of)>
scaling factor: 50
query mode: simple
number of clients: 10
number of threads: 1
number of transactions per client: 100
number of transactions actually processed: 1000/1000
latency average = 15.234 ms
tps = 656.123 (including connections establishing)
```

---

#### Redis Benchmarking

**Purpose:** Benchmark Redis cache performance

**Installation:**
```bash
# Included with Redis
sudo apt-get install redis-tools
```

**Example:**
```bash
# Benchmark GET/SET operations
redis-benchmark -h localhost -p 6379 -t get,set -n 100000 -c 50

# Benchmark specific operations
redis-benchmark -h localhost -p 6379 -n 100000 -c 50 -d 1000
```

**Output:**
```
====== SET ======
  100000 requests completed in 1.23 seconds
  50 parallel clients
  1000 bytes payload
  Throughput: 81301.08 requests per second
  Latency: p50=0.31ms, p95=0.87ms, p99=1.23ms

====== GET ======
  100000 requests completed in 0.98 seconds
  Throughput: 102040.82 requests per second
  Latency: p50=0.24ms, p95=0.65ms, p99=0.95ms
```

---

### 9.3 APM and Observability

#### Prometheus (Metrics Collection)

**Purpose:** Time-series metrics storage and querying

**Configuration:**
```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'media-gateway'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
```

**Key Metrics:**
```prometheus
# Request latency histogram
http_request_duration_seconds{service="discovery",endpoint="/api/search"}

# Request rate
rate(http_requests_total{service="discovery"}[5m])

# Error rate
rate(http_requests_total{service="discovery",status=~"5.."}[5m])
  / rate(http_requests_total{service="discovery"}[5m])

# CPU usage
rate(container_cpu_usage_seconds_total{service="discovery"}[5m])

# Memory usage
container_memory_usage_bytes{service="discovery"}
```

---

#### Cloud Monitoring (GCP)

**Purpose:** Managed monitoring for GKE, Cloud SQL, Redis

**Setup:**
```bash
# Enable Cloud Monitoring API
gcloud services enable monitoring.googleapis.com

# Install monitoring agent (if not using GKE)
curl -sSO https://dl.google.com/cloudagents/add-monitoring-agent-repo.sh
sudo bash add-monitoring-agent-repo.sh
sudo apt-get update
sudo apt-get install stackdriver-agent
```

**Custom Metrics:**
```go
// Export custom metrics to Cloud Monitoring
import (
    monitoring "cloud.google.com/go/monitoring/apiv3"
)

func recordLatency(ctx context.Context, latency time.Duration) {
    client, _ := monitoring.NewMetricClient(ctx)

    req := &monitoringpb.CreateTimeSeriesRequest{
        Name: "projects/media-gateway",
        TimeSeries: []*monitoringpb.TimeSeries{{
            Metric: &metricpb.Metric{
                Type: "custom.googleapis.com/search/latency",
            },
            Points: []*monitoringpb.Point{{
                Value: &monitoringpb.TypedValue{
                    Value: &monitoringpb.TypedValue_DoubleValue{
                        DoubleValue: latency.Seconds(),
                    },
                },
            }},
        }},
    }

    client.CreateTimeSeries(ctx, req)
}
```

---

## 10. Performance Regression Detection

### 10.1 Automated Benchmark Suite

**Objective:** Detect performance regressions in CI/CD pipeline

**GitHub Actions Workflow:**
```yaml
# .github/workflows/performance-benchmarks.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Rust benchmarks
        run: |
          cargo bench --bench sona_benchmark > bench-results.txt
          cargo bench --bench vector_search_benchmark >> bench-results.txt

      - name: Compare with baseline
        run: |
          python scripts/compare-benchmarks.py \
            --baseline benchmarks/baseline.json \
            --current bench-results.txt \
            --threshold 5  # 5% regression threshold

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: bench-results.txt

      - name: Fail on regression
        run: |
          if [ -f regression-detected.txt ]; then
            echo "Performance regression detected!"
            cat regression-detected.txt
            exit 1
          fi
```

---

### 10.2 Threshold-Based Alerts

**Regression Thresholds:**
```python
# scripts/compare-benchmarks.py
THRESHOLDS = {
    'sona_score_100_candidates': {
        'max_latency_ms': 10,       # Fail if >10ms (baseline: 5ms)
        'regression_pct': 20,       # Fail if >20% slower
    },
    'vector_search_1000_dims': {
        'max_latency_ms': 100,      # Fail if >100ms (baseline: 40ms)
        'regression_pct': 25,       # Fail if >25% slower
    },
    'crdt_merge_operation': {
        'max_latency_ms': 5,        # Fail if >5ms (baseline: 2ms)
        'regression_pct': 50,       # Fail if >50% slower
    },
}

def compare_benchmarks(baseline, current):
    regressions = []

    for benchmark_name, baseline_result in baseline.items():
        current_result = current.get(benchmark_name)

        if current_result is None:
            regressions.append(f"Benchmark {benchmark_name} missing in current run")
            continue

        threshold = THRESHOLDS.get(benchmark_name, {})
        max_latency = threshold.get('max_latency_ms', float('inf'))
        regression_pct = threshold.get('regression_pct', 100)

        # Check absolute threshold
        if current_result['latency_ms'] > max_latency:
            regressions.append(
                f"{benchmark_name}: {current_result['latency_ms']:.2f}ms "
                f"exceeds maximum {max_latency}ms"
            )

        # Check regression percentage
        pct_change = ((current_result['latency_ms'] - baseline_result['latency_ms'])
                      / baseline_result['latency_ms'] * 100)

        if pct_change > regression_pct:
            regressions.append(
                f"{benchmark_name}: {pct_change:.1f}% regression "
                f"(baseline: {baseline_result['latency_ms']:.2f}ms, "
                f"current: {current_result['latency_ms']:.2f}ms)"
            )

    return regressions
```

---

### 10.3 Trend Analysis

**Objective:** Track performance trends over time and predict regressions

**InfluxDB Time-Series Storage:**
```sql
-- Store benchmark results in InfluxDB
INSERT benchmark,test=sona_score_100_candidates,branch=main latency=5.2 1638360000000000000

-- Query 30-day trend
SELECT MEAN(latency) FROM benchmark
WHERE test = 'sona_score_100_candidates'
  AND time > now() - 30d
GROUP BY time(1d)

-- Detect trend (linear regression)
SELECT LINEAR_REG(latency, time) FROM benchmark
WHERE test = 'sona_score_100_candidates'
  AND time > now() - 7d
```

**Grafana Dashboard:**
```json
{
  "dashboard": {
    "title": "Performance Trends",
    "panels": [
      {
        "title": "SONA Scoring Latency (30-day trend)",
        "targets": [
          {
            "query": "SELECT MEAN(latency) FROM benchmark WHERE test = 'sona_score_100_candidates' GROUP BY time(1d)"
          }
        ],
        "alert": {
          "conditions": [
            {
              "type": "query",
              "query": "A",
              "evaluator": {
                "type": "gt",
                "params": [10]
              }
            }
          ],
          "frequency": "5m",
          "handler": 1,
          "name": "SONA latency regression"
        }
      }
    ]
  }
}
```

---

### 10.4 CI Integration

**Pre-Merge Performance Gates:**
```yaml
# .github/workflows/pr-checks.yml
name: PR Checks

on:
  pull_request:
    branches: [main]

jobs:
  performance-check:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Fetch all history for comparison

      - name: Run benchmarks on PR branch
        run: |
          cargo bench --bench sona_benchmark -- --save-baseline pr-branch

      - name: Checkout main branch
        run: git checkout main

      - name: Run benchmarks on main branch
        run: |
          cargo bench --bench sona_benchmark -- --save-baseline main-branch

      - name: Compare benchmarks
        run: |
          cargo bench --bench sona_benchmark -- --baseline main-branch --load-baseline pr-branch

      - name: Comment on PR
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const results = fs.readFileSync('benchmark-comparison.txt', 'utf8');

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Performance Benchmark Results\n\n\`\`\`\n${results}\n\`\`\``
            });
```

---

## 11. Optimization Priorities

### 11.1 Priority Framework

**P0 (Critical): User-Facing Latency**
- **Target:** All p95 latencies within SLO
- **Impact:** Direct user experience
- **Optimization Focus:**
  - Search latency <400ms p95
  - SONA personalization <100ms p95
  - Sync latency <100ms p95

**P0 Optimizations:**
1. Qdrant HNSW index tuning (M=16, efConstruction=200)
2. SONA LoRA cache warming (preload top 10K users)
3. PubNub channel optimization (reduce payload size)

---

**P1 (High): Throughput Capacity**
- **Target:** Support 100K concurrent users
- **Impact:** Scalability and growth
- **Optimization Focus:**
  - API Gateway: 5,000 RPS sustained
  - Discovery Service: 2,000 RPS sustained
  - Database: <80% connection pool utilization

**P1 Optimizations:**
1. Horizontal Pod Autoscaler tuning (scale at 70% CPU)
2. PostgreSQL read replica scaling (5 replicas)
3. Redis cluster mode for cache sharding

---

**P2 (Medium): Resource Efficiency**
- **Target:** <70% average CPU, <80% average memory
- **Impact:** Cost optimization and headroom
- **Optimization Focus:**
  - Reduce memory footprint per pod
  - Lower CPU usage per request
  - Improve cache hit rates

**P2 Optimizations:**
1. Rust async task tuning (reduce thread pool size)
2. Database query optimization (add missing indexes)
3. Cache TTL tuning (reduce eviction rate)

---

**P3 (Low): Cost Optimization**
- **Target:** <$4,000/month infrastructure cost
- **Impact:** Long-term sustainability
- **Optimization Focus:**
  - Preemptible nodes for non-Tier 1 services
  - Committed use discounts (1-year GCP commit)
  - Cloud Run scale-to-zero for Admin Dashboard

**P3 Optimizations:**
1. Move ingestion to preemptible nodes (60% discount)
2. Optimize storage (Nearline class for backups)
3. Right-size instance types (reduce over-provisioning)

---

### 11.2 Optimization Workflow

```
Performance Issue Detected
├─> 1. Identify bottleneck (profiling, metrics)
├─> 2. Assign priority (P0-P3)
├─> 3. Create optimization task
├─> 4. Implement fix
├─> 5. Benchmark before/after
├─> 6. Deploy to staging
├─> 7. Run load tests
├─> 8. Verify improvement >10%
├─> 9. Deploy to production
└─> 10. Monitor for 7 days
```

---

### 11.3 Profiling Tools

**Rust Profiling:**
```bash
# CPU profiling with perf
cargo build --release
sudo perf record --call-graph dwarf ./target/release/discovery-service
sudo perf report

# Flamegraph generation
cargo install flamegraph
cargo flamegraph --bin discovery-service

# Memory profiling with valgrind
cargo build
valgrind --tool=massif ./target/debug/discovery-service
ms_print massif.out.12345
```

**PostgreSQL Profiling:**
```sql
-- Enable pg_stat_statements extension
CREATE EXTENSION pg_stat_statements;

-- Find slowest queries
SELECT
  query,
  calls,
  mean_exec_time,
  max_exec_time,
  total_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 20;

-- Explain analyze slow query
EXPLAIN (ANALYZE, BUFFERS, VERBOSE)
SELECT * FROM content WHERE title LIKE '%scary%';
```

---

### 11.4 Continuous Optimization

**Weekly Performance Review:**
1. Review Grafana dashboards (latency, throughput, errors)
2. Identify top 3 bottlenecks
3. Create optimization tickets (prioritized P0-P3)
4. Schedule optimization sprints (1 week)
5. Re-benchmark and validate improvements

**Monthly Capacity Planning:**
1. Review traffic growth trends
2. Project resource needs (6 months)
3. Plan infrastructure scaling (GKE nodes, DB replicas)
4. Update cost estimates
5. Adjust autoscaling thresholds

---

## Summary

This performance benchmark specification defines:

1. **Service-Level Targets:** Latency (p50/p95/p99), throughput, availability for all 7 core services
2. **Load Testing Strategy:** Baseline, stress, spike, soak, and chaos testing methodologies
3. **Benchmark Scenarios:** 5 critical user journeys with expected latencies
4. **Resource Budgets:** CPU (<70%), memory (<80%), network (<70%), database connections (<80%)
5. **Database Performance:** Query latency targets, connection pool metrics, replication lag thresholds
6. **Caching Performance:** Multi-tier cache strategy with hit rate targets (L1: >40%, L2: >85%, L3: >90%)
7. **Network Performance:** Bandwidth utilization, latency budgets, timeout configuration
8. **Benchmark Tooling:** k6 for load testing, Prometheus for metrics, custom Rust benchmarks for micro-optimizations
9. **Regression Detection:** Automated CI benchmarks, threshold-based alerts, trend analysis
10. **Optimization Priorities:** P0 (latency), P1 (throughput), P2 (efficiency), P3 (cost)

**Next Steps:**
1. Implement benchmark suite in `/benchmarks` directory
2. Set up Prometheus + Grafana monitoring
3. Configure CI/CD performance gates
4. Baseline current system performance
5. Iteratively optimize based on P0-P3 priorities

---

**Document Status:** Planning Complete
**Review Required:** Performance Engineering Team, SRE Team
**Next Phase:** SPARC Refinement Part 4 - TDD Implementation

---

END OF PART 3
