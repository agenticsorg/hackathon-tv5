# SPARC Completion Phase - Part 2: Integration Validation Specification

**Document Version**: 1.0.0
**Last Updated**: 2025-12-06
**Phase**: Completion (5 of 5)
**Status**: Specification
**Platform**: Media Gateway

---

## Table of Contents

1. [Integration Test Strategy Overview](#1-integration-test-strategy-overview)
2. [Service Integration Validation](#2-service-integration-validation)
3. [External Platform Integration Validation](#3-external-platform-integration-validation)
4. [Database Integration Validation](#4-database-integration-validation)
5. [End-to-End User Journey Validation](#5-end-to-end-user-journey-validation)
6. [Integration Environment Specifications](#6-integration-environment-specifications)
7. [Integration Failure Handling](#7-integration-failure-handling)
8. [Acceptance Criteria](#8-acceptance-criteria)

---

## 1. Integration Test Strategy Overview

### 1.1 Testing Philosophy

```
┌─────────────────────────────────────────────────────────────────┐
│              Integration Testing Pyramid                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                        ▲                                        │
│                       ╱ ╲            E2E Tests                  │
│                      ╱   ╲           (10-15%)                   │
│                     ╱─────╲                                     │
│                    ╱       ╲                                    │
│                   ╱─────────╲       Integration Tests           │
│                  ╱           ╲      (30-40%)                    │
│                 ╱─────────────╲                                 │
│                ╱               ╲                                │
│               ╱─────────────────╲   Unit Tests                  │
│              ╱                   ╲  (50-60%)                    │
│             ╱─────────────────────╲                             │
│                                                                 │
│  Focus Areas:                                                   │
│  • Service Contract Validation                                 │
│  • External API Integration                                    │
│  • Database Consistency                                        │
│  • Real-time Communication                                     │
│  • Cross-service Transactions                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Service-to-Service Integration Matrix

```
┌──────────────┬─────────┬─────────┬──────────┬──────────┬─────────┬─────────┬───────────┬──────────┐
│   Service    │   API   │   MCP   │Discovery │   SONA   │  Sync   │  Auth   │ Ingestion │ Playback │
│              │ Gateway │ Server  │ Service  │  Engine  │ Service │ Service │  Service  │ Service  │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ API Gateway  │    -    │  HTTP   │   HTTP   │   HTTP   │  HTTP   │  HTTP   │   HTTP    │   HTTP   │
│              │         │  REST   │   REST   │   REST   │  REST   │  REST   │   REST    │   REST   │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ MCP Server   │  HTTP   │    -    │   gRPC   │   gRPC   │  gRPC   │  gRPC   │   gRPC    │   gRPC   │
│              │  REST   │         │          │          │         │         │           │          │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ Discovery    │  HTTP   │  gRPC   │    -     │   gRPC   │  gRPC   │    -    │   gRPC    │   gRPC   │
│              │         │         │          │          │         │         │           │          │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ SONA Engine  │  HTTP   │  gRPC   │   gRPC   │    -     │  gRPC   │    -    │     -     │   gRPC   │
│              │         │         │          │          │         │         │           │          │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ Sync Service │  HTTP   │  gRPC   │   gRPC   │   gRPC   │    -    │  gRPC   │   gRPC    │   gRPC   │
│              │         │         │          │          │         │         │           │          │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ Auth Service │  HTTP   │  gRPC   │    -     │    -     │  gRPC   │    -    │   gRPC    │   gRPC   │
│              │         │         │          │          │         │         │           │          │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ Ingestion    │  HTTP   │  gRPC   │   gRPC   │    -     │  gRPC   │  gRPC   │     -     │     -    │
│              │         │         │          │          │         │         │           │          │
├──────────────┼─────────┼─────────┼──────────┼──────────┼─────────┼─────────┼───────────┼──────────┤
│ Playback     │  HTTP   │  gRPC   │   gRPC   │   gRPC   │  gRPC   │  gRPC   │     -     │    -     │
│              │         │         │          │          │         │         │           │          │
└──────────────┴─────────┴─────────┴──────────┴──────────┴─────────┴─────────┴───────────┴──────────┘

Integration Test Priority:
  HIGH:    API Gateway ↔ All Services
  HIGH:    MCP Server ↔ All Services
  MEDIUM:  Discovery ↔ SONA, Sync, Ingestion, Playback
  MEDIUM:  Auth ↔ All Services (except Discovery, SONA)
  LOW:     Direct service-to-service (bypass gateway)
```

### 1.3 External API Integration Test Approach

```yaml
external_integrations:
  media_platforms:
    - service: Spotify API
      test_approach: "Contract testing with recorded responses"
      environments:
        - development: "Mock server with VCR recordings"
        - staging: "Sandbox API credentials"
        - production: "Real API with rate limiting"

    - service: Apple Music API
      test_approach: "Contract testing with recorded responses"
      environments:
        - development: "Mock server with VCR recordings"
        - staging: "Developer tokens"
        - production: "Real API with quota management"

    - service: Netflix/HBO/Disney+/Hulu/Prime
      test_approach: "Mock servers only (no public APIs)"
      environments:
        - all: "Custom mock implementations"

  infrastructure:
    - service: PubNub
      test_approach: "Real-time message validation"
      environments:
        - development: "Free tier subscription"
        - staging: "Dedicated test channels"
        - production: "Production channels with monitoring"

    - service: Qdrant
      test_approach: "Vector search validation"
      environments:
        - development: "Local Docker instance"
        - staging: "Dedicated test cluster"
        - production: "Production cluster with replication"

test_data_strategy:
  vcr_recordings:
    enabled: true
    directory: "tests/fixtures/vcr_cassettes"
    update_mode: "once"  # once, new_episodes, all, none

  mock_servers:
    enabled: true
    framework: "WireMock"
    port_range: "9000-9100"

  contract_definitions:
    format: "OpenAPI 3.0 + AsyncAPI"
    storage: "tests/contracts/"
    validation: "Pact + Dredd"
```

### 1.4 Contract Testing Specifications

```
┌─────────────────────────────────────────────────────────────────┐
│                   Contract Testing Flow                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Consumer Service                    Provider Service          │
│  ┌──────────────┐                    ┌──────────────┐          │
│  │              │                    │              │          │
│  │   Generate   │──── Contract ────▶ │   Validate   │          │
│  │  Consumer    │     (Pact)         │   Provider   │          │
│  │   Tests      │                    │    Tests     │          │
│  │              │                    │              │          │
│  └──────────────┘                    └──────────────┘          │
│         │                                    │                 │
│         ▼                                    ▼                 │
│  ┌──────────────┐                    ┌──────────────┐          │
│  │   Publish    │                    │    Verify    │          │
│  │  Contract    │──────────────────▶ │   Against    │          │
│  │  to Broker   │                    │   Contract   │          │
│  └──────────────┘                    └──────────────┘          │
│         │                                    │                 │
│         └────────────┬───────────────────────┘                 │
│                      ▼                                         │
│              ┌──────────────┐                                  │
│              │ Can Deploy?  │                                  │
│              │  (Pact Can   │                                  │
│              │   I Deploy)  │                                  │
│              └──────────────┘                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

Contract Test Framework:
  - Consumer-driven contracts (Pact)
  - Provider verification tests
  - Version compatibility matrix
  - Breaking change detection
```

### 1.5 Integration Test Categories

```yaml
integration_test_categories:

  category_1_api_contracts:
    scope: "Service-to-service API validation"
    test_count: ~150
    coverage_target: 95%
    tools: ["Pact", "Dredd", "REST Assured"]

  category_2_data_flow:
    scope: "Data propagation across services"
    test_count: ~80
    coverage_target: 90%
    tools: ["Testcontainers", "Chaos Mesh"]

  category_3_external_apis:
    scope: "Third-party platform integration"
    test_count: ~60
    coverage_target: 85%
    tools: ["VCR", "WireMock", "Nock"]

  category_4_database:
    scope: "Cross-database consistency"
    test_count: ~50
    coverage_target: 90%
    tools: ["Testcontainers", "Flyway", "Liquibase"]

  category_5_realtime:
    scope: "PubNub sync and WebSocket"
    test_count: ~40
    coverage_target: 85%
    tools: ["Socket.IO Client", "PubNub SDK"]

  category_6_security:
    scope: "Auth flow and token propagation"
    test_count: ~35
    coverage_target: 100%
    tools: ["OAuth2 Test Server", "JWT Inspector"]

  category_7_performance:
    scope: "Integration latency and throughput"
    test_count: ~25
    coverage_target: 80%
    tools: ["k6", "Gatling", "Locust"]

total_integration_tests: ~440
estimated_execution_time: "12-15 minutes (parallel)"
```

---

## 2. Service Integration Validation

### 2.1 API Gateway ↔ Auth Service Integration

#### 2.1.1 Authentication Flow Contract

```gherkin
Feature: API Gateway Authentication Integration

  Background:
    Given Auth Service is running on port 8084
    And API Gateway is running on port 8080
    And Auth Service has valid JWT signing keys

  Scenario: Successful user authentication via gateway
    Given a user with email "test@example.com" and password "SecurePass123!"
    When the client sends POST to "/api/v1/auth/login" via API Gateway
      """json
      {
        "email": "test@example.com",
        "password": "SecurePass123!"
      }
      """
    Then API Gateway forwards request to Auth Service at "http://auth:8084/login"
    And Auth Service validates credentials
    And Auth Service returns JWT token with claims:
      """json
      {
        "sub": "user-uuid-123",
        "email": "test@example.com",
        "roles": ["user"],
        "exp": 1735891200
      }
      """
    And API Gateway receives 200 OK response
    And API Gateway adds security headers:
      | Header                 | Value                |
      | X-Content-Type-Options | nosniff              |
      | X-Frame-Options        | DENY                 |
      | Strict-Transport-Security | max-age=31536000  |
    And client receives 200 OK with JWT token
    And response time is less than 150ms

  Scenario: Invalid credentials error propagation
    Given a user with invalid credentials
    When the client sends POST to "/api/v1/auth/login" with wrong password
    Then Auth Service returns 401 Unauthorized with error:
      """json
      {
        "error": "invalid_credentials",
        "message": "Email or password is incorrect",
        "timestamp": "2025-12-06T10:30:00Z"
      }
      """
    And API Gateway propagates 401 status code
    And API Gateway does NOT log password in error logs
    And API Gateway increments "auth.failed_attempts" metric
    And client receives same error structure

  Scenario: Auth Service timeout handling
    Given Auth Service is experiencing high latency (>5s)
    When the client sends authentication request
    Then API Gateway waits for configured timeout (3s)
    And after 3 seconds, API Gateway returns 504 Gateway Timeout
    And error response contains:
      """json
      {
        "error": "gateway_timeout",
        "message": "Authentication service is temporarily unavailable",
        "retry_after": 5
      }
      """
    And circuit breaker state changes to "half-open"
```

#### 2.1.2 Token Validation Integration

```yaml
token_validation_tests:

  test_valid_token_propagation:
    given:
      - "Valid JWT token in Authorization header"
      - "Token contains user_id and roles claims"
    when:
      - "Client calls protected endpoint /api/v1/discovery/search"
    then:
      - "API Gateway validates token signature"
      - "API Gateway extracts user context"
      - "API Gateway adds X-User-ID header with user UUID"
      - "API Gateway adds X-User-Roles header with comma-separated roles"
      - "API Gateway forwards request to Discovery Service"
      - "Discovery Service receives authenticated context"

  test_expired_token_handling:
    given:
      - "JWT token with exp claim in the past"
    when:
      - "Client calls protected endpoint"
    then:
      - "API Gateway validates token and detects expiration"
      - "API Gateway returns 401 Unauthorized"
      - "Response includes WWW-Authenticate header with error=invalid_token"
      - "Request is NOT forwarded to downstream service"
      - "Client can retry with token refresh"

  test_missing_token:
    given:
      - "Request without Authorization header"
    when:
      - "Client calls protected endpoint"
    then:
      - "API Gateway returns 401 Unauthorized"
      - "Response includes WWW-Authenticate: Bearer realm=API"
      - "Request is NOT forwarded"
      - "API Gateway logs anonymous access attempt"

  test_malformed_token:
    given:
      - "Authorization header with invalid JWT format"
    when:
      - "Client sends request"
    then:
      - "API Gateway attempts to parse token"
      - "Parsing fails, returns 400 Bad Request"
      - "Error message: 'Malformed authorization token'"
      - "Request is NOT forwarded"

contract_specification:
  consumer: "API Gateway"
  provider: "Auth Service"

  interactions:
    - description: "Login request"
      request:
        method: POST
        path: "/login"
        headers:
          Content-Type: "application/json"
        body:
          email: "user@example.com"
          password: "password123"
      response:
        status: 200
        headers:
          Content-Type: "application/json"
        body:
          token: "eyJ..."
          expires_in: 3600
          user:
            id: "uuid"
            email: "user@example.com"

    - description: "Token validation"
      request:
        method: POST
        path: "/validate"
        headers:
          Authorization: "Bearer eyJ..."
      response:
        status: 200
        body:
          valid: true
          user_id: "uuid"
          roles: ["user"]
```

### 2.2 API Gateway ↔ Discovery Service Integration

```gherkin
Feature: Content Discovery Integration

  Background:
    Given Discovery Service is running on port 8081
    And API Gateway is running on port 8080
    And user is authenticated with valid JWT

  Scenario: Search content across platforms
    Given user "user-123" is authenticated
    And user has connected Spotify, Netflix, and Hulu accounts
    When client sends GET to "/api/v1/discovery/search?q=stranger+things"
    Then API Gateway validates JWT token
    And API Gateway extracts user_id from token
    And API Gateway forwards request to Discovery Service:
      """
      GET http://discovery:8081/search?q=stranger+things
      Headers:
        X-User-ID: user-123
        X-Trace-ID: req-abc-123
      """
    And Discovery Service queries connected platforms in parallel
    And Discovery Service aggregates results from:
      | Platform | Results | Response Time |
      | Netflix  | 5       | 120ms         |
      | Hulu     | 3       | 95ms          |
      | Spotify  | 0       | 80ms          |
    And Discovery Service returns unified response:
      """json
      {
        "query": "stranger things",
        "results": [
          {
            "platform": "netflix",
            "title": "Stranger Things",
            "type": "series",
            "seasons": 4
          },
          {
            "platform": "hulu",
            "title": "Stranger Things Aftershow",
            "type": "series"
          }
        ],
        "total": 8,
        "execution_time_ms": 145
      }
      """
    And API Gateway returns 200 OK to client
    And total response time is less than 300ms

  Scenario: Discovery Service partial failure handling
    Given user has 3 connected platforms
    And Netflix API is returning 500 errors
    When client searches for content
    Then Discovery Service queries all platforms
    And Netflix query fails after 2 retries
    And Spotify and Hulu queries succeed
    And Discovery Service returns partial results with warning:
      """json
      {
        "results": [...],
        "warnings": [
          {
            "platform": "netflix",
            "error": "service_unavailable",
            "message": "Netflix is temporarily unavailable"
          }
        ],
        "partial_results": true
      }
      """
    And API Gateway returns 200 OK (not 500)
    And response includes X-Partial-Results: true header
```

### 2.3 MCP Server ↔ Services Integration (gRPC)

```
┌─────────────────────────────────────────────────────────────────┐
│              MCP Server gRPC Integration Flow                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Client (SSE/STDIO)                                           │
│         │                                                       │
│         │ MCP Protocol                                         │
│         ▼                                                       │
│   ┌──────────────┐                                             │
│   │  MCP Server  │                                             │
│   │  (Port 3000) │                                             │
│   └──────┬───────┘                                             │
│          │                                                      │
│          │ gRPC Calls (HTTP/2)                                │
│          │                                                      │
│   ┌──────┴──────────────────────────────────┐                 │
│   │      │        │        │         │      │                 │
│   ▼      ▼        ▼        ▼         ▼      ▼                 │
│ ┌────┐ ┌────┐  ┌────┐  ┌────┐   ┌────┐  ┌────┐               │
│ │Disc│ │SONA│  │Sync│  │Auth│   │Ing │  │Play│               │
│ │8081│ │8082│  │8083│  │8084│   │8085│  │8086│               │
│ └────┘ └────┘  └────┘  └────┘   └────┘  └────┘               │
│                                                                 │
│ Protocol Buffer Definitions:                                   │
│   - discovery.proto                                            │
│   - sona.proto                                                 │
│   - sync.proto                                                 │
│   - auth.proto                                                 │
│   - ingestion.proto                                            │
│   - playback.proto                                             │
└─────────────────────────────────────────────────────────────────┘
```

#### 2.3.1 gRPC Contract Tests

```protobuf
// discovery.proto contract
syntax = "proto3";

service DiscoveryService {
  rpc SearchContent(SearchRequest) returns (SearchResponse);
  rpc GetRecommendations(RecommendationRequest) returns (RecommendationResponse);
}

message SearchRequest {
  string user_id = 1;
  string query = 2;
  repeated string platforms = 3;
  SearchFilters filters = 4;
}

message SearchResponse {
  repeated ContentItem results = 1;
  int32 total = 2;
  int32 execution_time_ms = 3;
  repeated PlatformError errors = 4;
}
```

```gherkin
Feature: MCP Server gRPC Discovery Integration

  Scenario: MCP search_content tool calls Discovery Service
    Given MCP Server is connected to Discovery Service via gRPC
    And Discovery Service gRPC server is healthy
    When MCP client invokes tool "search_content" with:
      """json
      {
        "query": "inception",
        "platforms": ["netflix", "hulu"]
      }
      """
    Then MCP Server creates gRPC SearchRequest:
      """protobuf
      SearchRequest {
        user_id: "user-from-session"
        query: "inception"
        platforms: ["netflix", "hulu"]
      }
      """
    And MCP Server calls DiscoveryService.SearchContent via gRPC
    And gRPC connection uses HTTP/2 multiplexing
    And request includes metadata:
      | Key           | Value              |
      | x-request-id  | mcp-req-123        |
      | x-user-id     | user-from-session  |
      | authorization | Bearer eyJ...      |
    And Discovery Service processes request
    And Discovery Service returns SearchResponse within 500ms
    And MCP Server deserializes protobuf response
    And MCP Server converts to MCP tool response format
    And client receives JSON results

  Scenario: gRPC deadline exceeded handling
    Given Discovery Service is slow (>5s response)
    And MCP Server has gRPC deadline set to 3s
    When MCP client invokes search_content
    Then MCP Server sends gRPC request with deadline=3s
    And after 3 seconds, gRPC client times out
    And MCP Server receives DEADLINE_EXCEEDED status code
    And MCP Server returns error to client:
      """json
      {
        "error": "timeout",
        "message": "Discovery service did not respond within 3s",
        "code": "DEADLINE_EXCEEDED"
      }
      """
    And MCP Server logs timeout metric

  Scenario: gRPC connection retry with exponential backoff
    Given Discovery Service is temporarily unavailable
    When MCP Server attempts gRPC call
    Then gRPC client gets UNAVAILABLE status
    And MCP Server retries with backoff:
      | Attempt | Delay  |
      | 1       | 100ms  |
      | 2       | 200ms  |
      | 3       | 400ms  |
    And after 3 failed attempts, MCP Server returns error
    And error includes "service unavailable after 3 retries"
```

### 2.4 Discovery ↔ SONA Engine Integration

```gherkin
Feature: Semantic Recommendation Integration

  Background:
    Given Discovery Service is running on port 8081
    And SONA Engine is running on port 8082
    And both services are healthy

  Scenario: Discovery requests AI recommendations from SONA
    Given user "user-123" has viewing history:
      | Platform | Title              | Genre       | Rating |
      | Netflix  | The Crown          | Drama       | 4.5    |
      | Hulu     | The Handmaid's Tale| Drama       | 5.0    |
      | Disney+  | The Mandalorian    | Sci-Fi      | 4.8    |
    When Discovery Service receives GET /recommendations?user_id=user-123
    Then Discovery Service queries SONA Engine via gRPC:
      """protobuf
      RecommendationRequest {
        user_id: "user-123"
        context: {
          recent_views: ["crown", "handmaid", "mandalorian"]
          preferred_genres: ["drama", "sci-fi"]
          avg_rating: 4.76
        }
        limit: 10
      }
      """
    And SONA Engine performs vector similarity search in Qdrant
    And SONA Engine generates embeddings for user preferences
    And SONA Engine returns semantically similar content:
      """json
      {
        "recommendations": [
          {
            "title": "House of Cards",
            "platform": "netflix",
            "similarity_score": 0.89,
            "reason": "Similar political drama theme"
          },
          {
            "title": "Westworld",
            "platform": "hbo",
            "similarity_score": 0.85,
            "reason": "Sci-fi with complex narratives"
          }
        ],
        "execution_time_ms": 45
      }
      """
    And Discovery Service merges SONA results with platform data
    And Discovery Service returns enriched recommendations
    And total response time is less than 200ms

  Scenario: SONA Engine vector search validation
    Given SONA Engine has indexed 100,000 content items in Qdrant
    When Discovery Service requests recommendations
    Then SONA Engine queries Qdrant with user embedding vector
    And Qdrant search uses HNSW index for fast retrieval
    And search parameters:
      | Parameter | Value |
      | limit     | 20    |
      | ef        | 128   |
      | score_threshold | 0.7 |
    And Qdrant returns top-k results within 50ms
    And SONA Engine filters results by availability
    And SONA Engine ranks by combined score (similarity * availability)
```

### 2.5 Sync Service ↔ PubNub Integration

```yaml
sync_service_pubnub_integration:

  test_playback_state_sync:
    scenario: "User pauses video on mobile, state syncs to TV"
    given:
      - "User has 2 active devices: mobile and smart TV"
      - "Both devices subscribed to PubNub channel: user.user-123.playback"
      - "User is watching 'Stranger Things S01E01' at 00:12:34"
    when:
      - "Mobile app sends pause event to Sync Service"
      - "Sync Service publishes to PubNub"
    then:
      - "PubNub message structure:"
        ```json
        {
          "event_type": "playback.paused",
          "user_id": "user-123",
          "content_id": "netflix:stranger-things:s01e01",
          "timestamp": 754000,
          "device_id": "mobile-abc",
          "sync_time": "2025-12-06T10:30:00Z"
        }
        ```
      - "Smart TV receives PubNub message within 100ms"
      - "TV app updates playback state to paused at 00:12:34"
      - "Sync Service stores state in Redis for persistence"
      - "Total sync latency: <150ms (p95)"

  test_pubnub_presence_tracking:
    scenario: "Detect when user's device goes offline"
    given:
      - "User has 3 devices connected"
      - "Each device has presence heartbeat to PubNub"
    when:
      - "Mobile device loses internet connection"
    then:
      - "PubNub presence timeout triggers after 30s"
      - "PubNub sends presence event:"
        ```json
        {
          "action": "leave",
          "uuid": "device-mobile-abc",
          "channel": "user.user-123.playback",
          "timestamp": 1735891800
        }
        ```
      - "Sync Service receives presence event"
      - "Sync Service updates device status to 'offline'"
      - "Sync Service stops sending updates to offline device"
      - "When device reconnects, it fetches latest state from Redis"

  test_pubnub_message_history:
    scenario: "Retrieve missed sync events after reconnection"
    given:
      - "User's tablet was offline for 5 minutes"
      - "During offline period, 3 playback events occurred"
    when:
      - "Tablet reconnects and subscribes to channel"
    then:
      - "Sync Service fetches PubNub history"
      - "PubNub returns last 100 messages from channel"
      - "Sync Service filters messages after last_seen_timestamp"
      - "Tablet receives 3 missed events in chronological order"
      - "Tablet applies events to sync state to latest"

  test_pubnub_channel_groups:
    scenario: "Manage multiple sync channels efficiently"
    given:
      - "User has subscriptions to:"
        - "user.user-123.playback"
        - "user.user-123.queue"
        - "user.user-123.preferences"
    when:
      - "Device connects to Sync Service"
    then:
      - "Sync Service adds channels to PubNub channel group: user-123-sync"
      - "Device subscribes to channel group (1 subscription, 3 channels)"
      - "PubNub multiplexes messages from all channels"
      - "Device receives events from all sync categories"
      - "Reduces PubNub connection overhead"

pubnub_integration_contract:
  service: "Sync Service"
  external: "PubNub Real-time Network"

  authentication:
    method: "Publish/Subscribe Keys"
    keys:
      publish_key: "pub-c-xxx"
      subscribe_key: "sub-c-yyy"
    encryption: "AES-256 (optional)"

  channel_naming:
    pattern: "user.{user_id}.{category}"
    examples:
      - "user.user-123.playback"
      - "user.user-123.queue"
      - "device.mobile-abc.status"

  message_size:
    max_size: "32KB per message"
    typical: "1-2KB"

  rate_limits:
    publish: "500 messages/second per key"
    subscribe: "Unlimited"

  quality_of_service:
    delivery: "At least once"
    ordering: "FIFO per channel"
    latency: "<100ms global (p95)"
```

### 2.6 Cross-Service Transaction Testing

```gherkin
Feature: Multi-Service Transaction Consistency

  Scenario: Add content to queue (Auth + Ingestion + Sync)
    Given user is authenticated with valid JWT
    And user wants to add Netflix show to queue
    When client sends POST /api/v1/queue/add:
      """json
      {
        "platform": "netflix",
        "content_id": "80057281",
        "title": "Stranger Things"
      }
      """
    Then API Gateway validates auth token (Auth Service)
    And API Gateway forwards to Ingestion Service
    And Ingestion Service starts transaction
    And Ingestion Service validates content exists on Netflix
    And Ingestion Service inserts into PostgreSQL:
      """sql
      INSERT INTO user_queue (user_id, platform, content_id, added_at)
      VALUES ('user-123', 'netflix', '80057281', NOW());
      ```
    And Ingestion Service publishes event to Sync Service
    And Sync Service broadcasts to PubNub channel
    And all devices receive queue update
    And Ingestion Service commits transaction
    And response returns 201 Created

    # Rollback scenario
    Given Sync Service is unavailable
    When queue add transaction reaches sync step
    Then Sync Service publish fails after retries
    And Ingestion Service rolls back PostgreSQL insert
    And response returns 503 Service Unavailable
    And user does not see inconsistent state

  Scenario: Cross-platform playback position sync
    Given user is watching content on device A
    And user switches to device B
    Then transaction flow:
      1. "Device A sends playback position to Sync Service"
      2. "Sync Service updates Redis cache"
      3. "Sync Service publishes PubNub event"
      4. "Device B receives event and updates UI"
      5. "Sync Service asynchronously updates PostgreSQL"
    And if any step fails:
      - "Redis update failed → retry 3x → return error"
      - "PubNub publish failed → queue for retry → continue"
      - "PostgreSQL update failed → log and retry in background"
    And eventual consistency achieved within 5 seconds
```

---

## 3. External Platform Integration Validation

### 3.1 Spotify API Integration Tests

```gherkin
Feature: Spotify API Integration

  Background:
    Given Spotify API credentials are configured
    And user has authorized Spotify access via OAuth2
    And valid access token is available

  Scenario: Search for music content on Spotify
    Given user searches for "Taylor Swift"
    When Discovery Service calls Spotify Search API:
      ```
      GET https://api.spotify.com/v1/search
      Headers:
        Authorization: Bearer {access_token}
      Params:
        q: "Taylor Swift"
        type: "artist,album,track"
        limit: 20
      ```
    Then Spotify API returns 200 OK with results:
      ```json
      {
        "artists": {
          "items": [
            {
              "id": "06HL4z0CvFAxyc27GXpf02",
              "name": "Taylor Swift",
              "genres": ["pop", "country"]
            }
          ]
        },
        "tracks": {
          "items": [...]
        }
      }
      ```
    And Discovery Service normalizes response to platform-agnostic format
    And response includes Spotify-specific metadata:
      - "spotify_uri"
      - "preview_url"
      - "album_art"
    And total response time is less than 400ms

  Scenario: Spotify rate limiting handling
    Given Discovery Service has made 180 API calls in last 30 seconds
    And Spotify rate limit is 180 calls per 30 seconds
    When Discovery Service attempts another API call
    Then Spotify returns 429 Too Many Requests with header:
      ```
      Retry-After: 15
      ```
    And Discovery Service extracts retry_after value
    And Discovery Service queues request for retry in 15 seconds
    And Discovery Service returns cached results if available
    Or returns partial results from other platforms
    And Discovery Service does NOT crash or hang

  Scenario: Spotify OAuth token refresh
    Given user's Spotify access token expires in 60 seconds
    When Discovery Service attempts API call
    Then Discovery Service detects token expiration
    And Discovery Service calls Spotify Token Endpoint:
      ```
      POST https://accounts.spotify.com/api/token
      Body:
        grant_type: refresh_token
        refresh_token: {refresh_token}
        client_id: {client_id}
        client_secret: {client_secret}
      ```
    And Spotify returns new access token:
      ```json
      {
        "access_token": "new_token",
        "expires_in": 3600
      }
      ```
    And Discovery Service stores new token in Redis
    And Discovery Service retries original API call
    And entire flow completes within 1 second

  Scenario: Spotify API error handling
    Given Spotify API is returning errors
    When Discovery Service calls API
    Then handle error responses:
      | Status | Error               | Action                          |
      | 401    | Unauthorized        | Refresh token, retry            |
      | 403    | Forbidden           | Check scopes, return error      |
      | 404    | Not Found           | Return empty results            |
      | 429    | Too Many Requests   | Exponential backoff, retry      |
      | 500    | Internal Error      | Retry 3x, then fail gracefully  |
      | 503    | Service Unavailable | Circuit breaker open, use cache |
```

#### 3.1.1 Spotify VCR Recordings

```yaml
vcr_cassette: "spotify_search_taylor_swift"

interactions:
  - request:
      method: GET
      uri: "https://api.spotify.com/v1/search"
      headers:
        Authorization: "Bearer REDACTED"
      query:
        q: "Taylor Swift"
        type: "artist,album,track"
        limit: "20"
    response:
      status: 200
      headers:
        Content-Type: "application/json"
        X-RateLimit-Remaining: "179"
      body:
        # Recorded response data
        artists: {...}
        tracks: {...}
      latency: 0.245  # seconds

  - request:
      method: GET
      uri: "https://api.spotify.com/v1/albums/1EoDsNmgTLtmwe1BDAVxV5"
    response:
      status: 200
      body:
        id: "1EoDsNmgTLtmwe1BDAVxV5"
        name: "Midnights"
        release_date: "2022-10-21"
```

### 3.2 Apple Music API Integration Tests

```gherkin
Feature: Apple Music API Integration

  Background:
    Given Apple Music developer token is configured
    And user has authorized Apple Music access

  Scenario: Search Apple Music catalog
    Given user searches for "The Beatles"
    When Discovery Service calls Apple Music Search API:
      ```
      GET https://api.music.apple.com/v1/catalog/us/search
      Headers:
        Authorization: Bearer {developer_token}
        Music-User-Token: {user_token}
      Params:
        term: "The Beatles"
        types: "artists,albums,songs"
        limit: 25
      ```
    Then Apple Music returns results in JSON:API format:
      ```json
      {
        "results": {
          "artists": {
            "data": [
              {
                "id": "136975",
                "type": "artists",
                "attributes": {
                  "name": "The Beatles",
                  "genreNames": ["Rock"]
                }
              }
            ]
          }
        }
      }
      ```
    And Discovery Service parses JSON:API structure
    And Discovery Service extracts attributes and relationships
    And response includes Apple Music links and artwork

  Scenario: Apple Music user library access
    Given user wants to sync their Apple Music library
    When Discovery Service calls Library API:
      ```
      GET https://api.music.apple.com/v1/me/library/songs
      Headers:
        Authorization: Bearer {developer_token}
        Music-User-Token: {user_token}
      ```
    Then Apple Music returns user's saved songs
    And Discovery Service stores library metadata in PostgreSQL
    And library sync completes within 10 seconds for 1000 songs

  Scenario: Apple Music playback tracking
    Given user is listening to a song via Apple Music
    When Playback Service reports playback event
    Then Playback Service calls Apple Music Activity API:
      ```
      POST https://api.music.apple.com/v1/me/recent/played/tracks
      Body:
        {
          "data": [{
            "id": "song-id",
            "type": "songs",
            "attributes": {
              "playParams": {...}
            }
          }]
        }
      ```
    And Apple Music updates user's listening history
    And playback event is logged for analytics
```

### 3.3 Netflix/Streaming Services Mock Integration

```yaml
# Since Netflix, HBO, Disney+, Hulu, Prime don't have public APIs
# we use mock servers based on web scraping data structures

streaming_mock_servers:

  netflix_mock:
    base_url: "http://localhost:9001"
    authentication: "Mock API key"

    endpoints:
      - path: "/catalog/search"
        method: GET
        request:
          params:
            q: "search query"
            page: 1
        response:
          status: 200
          body:
            results:
              - id: "80057281"
                title: "Stranger Things"
                type: "series"
                seasons: 4
                available: true
        latency: "100-200ms"

      - path: "/user/watchlist"
        method: GET
        response:
          body:
            items:
              - content_id: "80057281"
                added_at: "2024-01-15T10:30:00Z"

    test_scenarios:
      - name: "Search returns results"
        given: "Mock server has catalog data"
        when: "Discovery Service searches Netflix"
        then: "Returns mock results matching schema"

      - name: "Watchlist retrieval"
        given: "User has items in watchlist"
        when: "Ingestion Service syncs watchlist"
        then: "Returns user's watchlist items"

  hbo_max_mock:
    base_url: "http://localhost:9002"
    # Similar structure

  disney_plus_mock:
    base_url: "http://localhost:9003"
    # Similar structure

integration_test_approach:
  development:
    strategy: "WireMock servers with recorded responses"
    data_source: "Scraped catalog data (updated quarterly)"
    authentication: "Mock tokens"

  staging:
    strategy: "WireMock with production-like data"
    data_source: "Copy of production mock data"

  production:
    strategy: "Same mock servers with monitoring"
    fallback: "If real integration becomes available, switch"

contract_validation:
  - "Verify mock response schemas match documented structures"
  - "Test error scenarios (404, 500, timeout)"
  - "Validate rate limiting behavior"
  - "Ensure consistent response times"
```

### 3.4 PubNub Real-time Integration Tests

```gherkin
Feature: PubNub Real-time Messaging Integration

  Background:
    Given PubNub publish key and subscribe key are configured
    And Sync Service is connected to PubNub

  Scenario: Publish playback event to PubNub channel
    Given user pauses video at timestamp 12:34
    When Playback Service sends event to Sync Service
    Then Sync Service publishes to PubNub:
      ```javascript
      pubnub.publish({
        channel: 'user.user-123.playback',
        message: {
          event: 'paused',
          timestamp: 754000,
          content_id: 'netflix:stranger-things:s01e01',
          device_id: 'mobile-abc'
        }
      })
      ```
    And PubNub returns publish response:
      ```json
      {
        "timetoken": "17358918001234567"
      }
      ```
    And message is delivered to all subscribed devices within 100ms

  Scenario: Subscribe to PubNub channel and receive messages
    Given user has 2 devices: mobile and TV
    When both devices subscribe to channel "user.user-123.playback"
    Then subscription callback receives messages:
      ```javascript
      pubnub.subscribe({
        channels: ['user.user-123.playback'],
        withPresence: true
      });

      pubnub.addListener({
        message: (event) => {
          // Process event.message
        },
        presence: (event) => {
          // Handle device online/offline
        }
      });
      ```
    And devices receive real-time updates
    And presence events track device connectivity

  Scenario: PubNub message persistence and history
    Given user's device was offline for 10 minutes
    When device reconnects and subscribes
    Then Sync Service fetches message history:
      ```javascript
      pubnub.history({
        channel: 'user.user-123.playback',
        count: 100,
        start: lastReceivedTimetoken
      })
      ```
    And PubNub returns missed messages in chronological order
    And device applies all missed events to sync state

  Scenario: PubNub presence heartbeat and timeout
    Given device is connected and sending presence heartbeats
    When device loses internet connection
    Then PubNub waits for heartbeat timeout (default 300s)
    And PubNub sends presence leave event to all subscribers
    And Sync Service marks device as offline
    And stops sending messages to offline device

  Scenario: PubNub encryption for sensitive data
    Given message contains sensitive user data
    When Sync Service publishes message
    Then message is encrypted with AES-256:
      ```javascript
      pubnub = new PubNub({
        publishKey: 'pub-key',
        subscribeKey: 'sub-key',
        cipherKey: 'encryption-key',  // AES-256
        ssl: true
      });
      ```
    And encrypted message is sent over HTTPS
    And only devices with correct cipher key can decrypt
```

### 3.5 Qdrant Vector Database Integration Tests

```gherkin
Feature: Qdrant Vector Search Integration

  Background:
    Given Qdrant cluster is running on port 6333
    And SONA Engine is connected to Qdrant
    And collection "media_content" exists with 768-dim vectors

  Scenario: Insert content embeddings into Qdrant
    Given SONA Engine generates embedding for "Stranger Things":
      ```python
      embedding = model.encode("Stranger Things sci-fi horror series")
      # Returns 768-dimensional vector
      ```
    When SONA Engine inserts into Qdrant:
      ```python
      client.upsert(
        collection_name="media_content",
        points=[
          {
            "id": "netflix:80057281",
            "vector": embedding,  # 768 dims
            "payload": {
              "title": "Stranger Things",
              "platform": "netflix",
              "genres": ["sci-fi", "horror", "drama"],
              "year": 2016
            }
          }
        ]
      )
      ```
    Then Qdrant stores vector with HNSW index
    And vector is searchable within 1 second
    And payload metadata is stored for filtering

  Scenario: Semantic similarity search in Qdrant
    Given user's preference vector is computed from watch history
    When SONA Engine queries Qdrant for similar content:
      ```python
      results = client.search(
        collection_name="media_content",
        query_vector=user_preference_vector,
        limit=10,
        score_threshold=0.7,
        filter={
          "must": [
            {"key": "year", "range": {"gte": 2020}}
          ]
        }
      )
      ```
    Then Qdrant returns top 10 most similar vectors:
      ```json
      [
        {
          "id": "netflix:80057281",
          "score": 0.92,
          "payload": {
            "title": "Stranger Things",
            "platform": "netflix"
          }
        },
        {
          "id": "hulu:dark",
          "score": 0.87,
          "payload": {
            "title": "Dark",
            "platform": "hulu"
          }
        }
      ]
      ```
    And search completes within 50ms
    And results are ranked by cosine similarity

  Scenario: Qdrant collection schema validation
    Given SONA Engine needs to create new collection
    When SONA Engine calls Qdrant API:
      ```python
      client.create_collection(
        collection_name="media_content",
        vectors_config={
          "size": 768,
          "distance": "Cosine"
        },
        hnsw_config={
          "m": 16,
          "ef_construct": 100
        }
      )
      ```
    Then Qdrant creates collection with specifications:
      | Parameter     | Value   |
      | Vector Size   | 768     |
      | Distance      | Cosine  |
      | Index Type    | HNSW    |
      | M (neighbors) | 16      |
      | ef_construct  | 100     |
    And collection is ready for insertion

  Scenario: Qdrant batch operations performance
    Given SONA Engine has 1000 content items to index
    When SONA Engine batches upserts (100 per batch):
      ```python
      for batch in chunks(content_items, 100):
        client.upsert(collection_name="media_content", points=batch)
      ```
    Then all 1000 items are indexed within 10 seconds
    And batch operations use connection pooling
    And no rate limiting errors occur

  Scenario: Qdrant high availability validation
    Given Qdrant cluster has 3 replicas
    When one replica node fails
    Then Qdrant automatically routes queries to healthy nodes
    And search requests continue without errors
    And replication factor ensures data availability
    And failed node is detected within 30 seconds
```

---

## 4. Database Integration Validation

### 4.1 PostgreSQL Integration Tests

```gherkin
Feature: PostgreSQL Database Integration

  Background:
    Given PostgreSQL 15 is running
    And all microservices have connection pools configured
    And database schema is migrated to latest version

  Scenario: Connection pool management
    Given Auth Service has connection pool configured:
      ```yaml
      pool:
        min: 2
        max: 10
        idle_timeout: 30s
        connection_timeout: 5s
      ```
    When Auth Service starts
    Then PostgreSQL connection pool initializes with 2 connections
    And health check validates connections
    When load increases and service needs more connections
    Then pool scales up to 10 active connections
    When load decreases
    Then idle connections are closed after 30s
    And pool returns to minimum 2 connections

  Scenario: Cross-service database consistency
    Given user updates email in Auth Service
    When Auth Service executes:
      ```sql
      UPDATE users SET email = 'new@example.com' WHERE id = 'user-123';
      ```
    And transaction commits successfully
    Then other services see updated email immediately
    And Ingestion Service queries return new email
    And Sync Service receives database trigger notification
    And all services maintain consistent view

  Scenario: Database migration validation
    Given new migration adds column "preferences" to users table
    When migration runs via Flyway:
      ```sql
      -- V002__add_user_preferences.sql
      ALTER TABLE users ADD COLUMN preferences JSONB DEFAULT '{}';
      CREATE INDEX idx_users_preferences ON users USING GIN(preferences);
      ```
    Then migration completes without errors
    And all services detect schema change
    And ORM models are compatible with new schema
    And backward compatibility is maintained
    And rollback migration is tested

  Scenario: Transaction isolation testing
    Given two concurrent requests try to update same user row
    When Request A starts transaction:
      ```sql
      BEGIN;
      SELECT * FROM users WHERE id = 'user-123' FOR UPDATE;
      UPDATE users SET login_count = login_count + 1;
      ```
    And Request B tries to update same row:
      ```sql
      BEGIN;
      SELECT * FROM users WHERE id = 'user-123' FOR UPDATE;
      -- This will block until Request A commits
      ```
    Then Request B waits for Request A to commit
    And isolation level READ COMMITTED prevents dirty reads
    And both transactions complete successfully
    And final login_count is correct (no lost updates)

  Scenario: Database deadlock detection and retry
    Given two transactions acquire locks in different order
    When Transaction A locks user-123 then tries to lock user-456
    And Transaction B locks user-456 then tries to lock user-123
    Then PostgreSQL detects deadlock within 1 second
    And PostgreSQL aborts one transaction with error:
      ```
      ERROR: deadlock detected
      DETAIL: Process 1234 waits for ShareLock on transaction 5678
      ```
    And application catches deadlock error
    And application retries transaction with exponential backoff
    And retry succeeds after lock is released
```

#### 4.1.1 PostgreSQL Performance Tests

```yaml
database_performance_tests:

  test_query_performance:
    scenario: "User authentication query performance"
    query: |
      SELECT id, email, password_hash, roles
      FROM users
      WHERE email = $1
      LIMIT 1
    parameters: ["test@example.com"]
    acceptance_criteria:
      - execution_time_p95: "<5ms"
      - execution_time_p99: "<10ms"
      - uses_index: "idx_users_email"
    test_approach:
      - "Run EXPLAIN ANALYZE to verify index usage"
      - "Execute query 1000 times, measure latency distribution"
      - "Verify no sequential scans on users table"

  test_connection_pool_saturation:
    scenario: "Handle spike in database connections"
    setup:
      - "Configure pool max_connections = 10"
      - "Generate 50 concurrent requests"
    then:
      - "First 10 requests acquire connections immediately"
      - "Remaining 40 requests queue with timeout"
      - "Requests complete as connections are released"
      - "No connection refused errors"
      - "Total time < 2 seconds for all 50 requests"

  test_bulk_insert_performance:
    scenario: "Ingest 1000 playback events"
    query: |
      INSERT INTO playback_events (user_id, content_id, timestamp, event_type)
      VALUES ($1, $2, $3, $4)
    approach: "Batch insert using COPY or multi-row INSERT"
    acceptance_criteria:
      - "1000 inserts complete in <500ms"
      - "Use prepared statements for efficiency"
      - "Transaction commits atomically"
```

### 4.2 Redis Cache Integration Tests

```gherkin
Feature: Redis Cache Integration

  Background:
    Given Redis 7 is running on port 6379
    And services use Redis for caching and session storage

  Scenario: Cache user session data
    Given user logs in successfully
    When Auth Service creates session:
      ```redis
      SET session:abc-123 '{"user_id":"user-123","expires_at":"2025-12-07T10:30:00Z"}' EX 86400
      ```
    Then session is stored in Redis with 24-hour TTL
    And subsequent requests retrieve session:
      ```redis
      GET session:abc-123
      ```
    And session retrieval takes <1ms
    When session expires after 24 hours
    Then Redis automatically deletes key
    And user must re-authenticate

  Scenario: Cache discovery search results
    Given user searches for "inception"
    When Discovery Service executes search
    Then Discovery Service caches results in Redis:
      ```redis
      SETEX search:inception:user-123 300 '{"results":[...],"timestamp":"2025-12-06T10:30:00Z"}'
      ```
    And cache TTL is 5 minutes (300 seconds)
    When user searches "inception" again within 5 minutes
    Then Discovery Service returns cached results
    And avoids calling external APIs
    And response time is <50ms (cache hit)

  Scenario: Redis pub/sub for cache invalidation
    Given multiple service instances share Redis cache
    When Auth Service updates user data:
      ```sql
      UPDATE users SET email = 'new@example.com' WHERE id = 'user-123';
      ```
    Then Auth Service publishes invalidation message:
      ```redis
      PUBLISH cache:invalidate '{"type":"user","id":"user-123"}'
      ```
    And all service instances subscribed to cache:invalidate receive message
    And each instance deletes cached user data:
      ```redis
      DEL user:user-123
      ```
    And next request fetches fresh data from PostgreSQL

  Scenario: Redis connection failure handling
    Given Redis server becomes unavailable
    When service attempts to read from cache:
      ```python
      try:
          cached = redis.get('key')
      except RedisConnectionError:
          # Fallback to database
          cached = None
      ```
    Then service catches connection error
    And service falls back to database query
    And service logs Redis unavailability
    And service continues operating (degraded but functional)
    When Redis becomes available again
    Then service reconnects automatically
    And caching resumes

  Scenario: Redis cluster data sharding
    Given Redis cluster has 3 master nodes
    When service stores key "user:123":
      ```
      Cluster routes to node based on CRC16(key) % 16384
      ```
    Then key is stored on correct shard
    And reads are directed to same shard
    When cluster reshards data
    Then keys are migrated without downtime
    And service experiences minimal latency spike
```

### 4.3 Cross-Database Consistency Tests

```gherkin
Feature: Cross-Database Consistency

  Scenario: PostgreSQL + Redis consistency
    Given user updates profile in PostgreSQL
    When Auth Service executes:
      ```python
      # Update database
      db.execute("UPDATE users SET name = $1 WHERE id = $2", new_name, user_id)
      db.commit()

      # Invalidate cache
      redis.delete(f"user:{user_id}")
      ```
    Then PostgreSQL contains updated data
    And Redis cache is invalidated
    And next read fetches fresh data from PostgreSQL
    And cache is repopulated with new data

    # Test failure scenario
    Given Redis is unavailable during update
    When Auth Service updates PostgreSQL
    Then PostgreSQL update succeeds
    And Redis invalidation fails (logged but not fatal)
    And stale cache expires naturally via TTL
    And eventual consistency is maintained

  Scenario: PostgreSQL + Qdrant consistency
    Given new content is added to PostgreSQL catalog
    When Ingestion Service inserts content:
      ```python
      # Insert to PostgreSQL
      content_id = db.insert("INSERT INTO content (...) VALUES (...)")

      # Generate embedding and insert to Qdrant
      embedding = sona.generate_embedding(content_metadata)
      qdrant.upsert(collection="media_content", points=[{
          "id": content_id,
          "vector": embedding,
          "payload": content_metadata
      }])
      ```
    Then content exists in both PostgreSQL and Qdrant
    And vector search can find content
    And metadata is consistent across databases

    # Test rollback scenario
    Given Qdrant insertion fails
    When Ingestion Service attempts to add content
    Then PostgreSQL transaction is rolled back
    And content is not added to either database
    And data consistency is maintained

  Scenario: Multi-database transaction saga
    Given user performs action requiring updates to PostgreSQL, Redis, and Qdrant
    When Sync Service executes saga:
      ```python
      saga = Saga()

      # Step 1: Update PostgreSQL
      saga.add_step(
          execute=lambda: db.update_playback_position(user_id, position),
          compensate=lambda: db.rollback_playback_position(user_id, old_position)
      )

      # Step 2: Update Redis cache
      saga.add_step(
          execute=lambda: redis.set(f"position:{user_id}", position),
          compensate=lambda: redis.delete(f"position:{user_id}")
      )

      # Step 3: Update Qdrant usage vectors
      saga.add_step(
          execute=lambda: qdrant.update_user_vector(user_id),
          compensate=lambda: qdrant.restore_user_vector(user_id)
      )

      saga.execute()
      ```
    Then all steps execute in order
    And if any step fails, saga executes compensating transactions
    And system returns to consistent state
    And saga completion is logged for audit
```

---

## 5. End-to-End User Journey Validation

### 5.1 User Registration and Authentication Flow

```
┌─────────────────────────────────────────────────────────────────┐
│           E2E User Registration Journey                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Client                                                      │
│     │                                                           │
│     │ POST /api/v1/auth/register                               │
│     ▼                                                           │
│  2. API Gateway (8080)                                         │
│     │                                                           │
│     │ Validate request                                         │
│     ▼                                                           │
│  3. Auth Service (8084)                                        │
│     │                                                           │
│     ├─▶ Check email uniqueness (PostgreSQL)                   │
│     ├─▶ Hash password (bcrypt)                                │
│     ├─▶ Insert user record (PostgreSQL)                       │
│     ├─▶ Generate JWT token                                    │
│     └─▶ Store session (Redis)                                 │
│                                                                 │
│  4. Response to Client                                         │
│     {                                                           │
│       "user_id": "user-123",                                   │
│       "token": "eyJ...",                                       │
│       "expires_in": 3600                                       │
│     }                                                           │
│                                                                 │
│  5. Client stores token                                        │
│                                                                 │
│  6. Client makes authenticated request                         │
│     GET /api/v1/discovery/search                               │
│     Headers: Authorization: Bearer eyJ...                      │
│                                                                 │
│  7. API Gateway validates token → Success                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

```gherkin
Feature: End-to-End User Registration and Authentication

  Scenario: Complete user registration flow
    Given client application is running
    And all backend services are healthy
    When user fills registration form:
      | Field    | Value              |
      | Email    | new@example.com    |
      | Password | SecurePass123!     |
      | Name     | Test User          |
    And user submits form
    Then client sends POST to https://api.mediagateway.com/api/v1/auth/register
    And request body contains:
      ```json
      {
        "email": "new@example.com",
        "password": "SecurePass123!",
        "name": "Test User"
      }
      ```
    And API Gateway receives request
    And API Gateway validates request schema
    And API Gateway forwards to Auth Service
    And Auth Service validates email format
    And Auth Service checks email uniqueness in PostgreSQL
    And Auth Service hashes password with bcrypt (cost=12)
    And Auth Service inserts user into database:
      ```sql
      INSERT INTO users (id, email, password_hash, name, created_at)
      VALUES (gen_random_uuid(), 'new@example.com', '$2b$12...', 'Test User', NOW());
      ```
    And Auth Service generates JWT token with claims:
      ```json
      {
        "sub": "user-uuid",
        "email": "new@example.com",
        "roles": ["user"],
        "iat": 1735891200,
        "exp": 1735894800
      }
      ```
    And Auth Service stores session in Redis:
      ```
      SET session:{token_id} {user_data} EX 3600
      ```
    And Auth Service returns 201 Created
    And API Gateway forwards response to client
    And client receives response within 300ms
    And client stores JWT token in secure storage
    And user is logged in successfully

  Scenario: Login and access protected resource
    Given user has registered account
    When user enters email and password
    And user clicks "Login"
    Then client sends POST to /api/v1/auth/login
    And Auth Service validates credentials
    And Auth Service returns JWT token
    And client stores token
    When user navigates to "Discover Content"
    Then client sends GET to /api/v1/discovery/search?q=action
    And client includes header: Authorization: Bearer {token}
    And API Gateway validates JWT signature
    And API Gateway extracts user_id from token
    And API Gateway forwards to Discovery Service with headers:
      | Header      | Value      |
      | X-User-ID   | user-uuid  |
      | X-User-Roles| user       |
    And Discovery Service receives authenticated request
    And Discovery Service queries user's connected platforms
    And Discovery Service returns personalized results
    And client displays results to user
    And total flow completes within 500ms
```

### 5.2 Content Discovery E2E Journey

```gherkin
Feature: End-to-End Content Discovery

  Background:
    Given user "user-123" is authenticated
    And user has connected accounts:
      | Platform     | Status    |
      | Netflix      | Connected |
      | Spotify      | Connected |
      | Hulu         | Connected |

  Scenario: Search for content across all platforms
    When user enters search query "stranger things"
    And user clicks "Search"
    Then client sends GET /api/v1/discovery/search?q=stranger+things
    And API Gateway validates JWT token
    And API Gateway forwards to Discovery Service
    And Discovery Service initiates parallel platform searches:
      ```
      ┌─────────────────────────────────────┐
      │   Discovery Service                 │
      │                                     │
      │   ┌──────────┐  ┌──────────┐       │
      │   │ Netflix  │  │ Spotify  │       │
      │   │  Search  │  │  Search  │       │
      │   └────┬─────┘  └────┬─────┘       │
      │        │             │              │
      │   ┌────┴──────┐ ┌────┴──────┐      │
      │   │   Hulu    │ │  Disney+  │      │
      │   │  Search   │ │  Search   │      │
      │   └─────┬─────┘ └─────┬─────┘      │
      │         │             │             │
      │         └──────┬──────┘             │
      │                ▼                    │
      │         Aggregate Results           │
      └─────────────────────────────────────┘
      ```
    And Netflix search completes in 120ms with 5 results
    And Spotify search completes in 80ms with 2 results
    And Hulu search completes in 95ms with 3 results
    And Disney+ search completes in 110ms with 0 results
    And Discovery Service aggregates all results
    And Discovery Service queries SONA Engine for AI ranking:
      ```protobuf
      RankingRequest {
        user_id: "user-123"
        results: [/* all platform results */]
        context: {
          search_query: "stranger things"
          user_preferences: {/* from history */}
        }
      }
      ```
    And SONA Engine returns ranked results
    And Discovery Service returns unified response:
      ```json
      {
        "query": "stranger things",
        "total": 10,
        "results": [
          {
            "platform": "netflix",
            "title": "Stranger Things",
            "type": "series",
            "relevance_score": 0.98,
            "available": true
          },
          {
            "platform": "spotify",
            "title": "Stranger Things Soundtrack",
            "type": "playlist",
            "relevance_score": 0.85
          }
        ],
        "execution_time_ms": 156
      }
      ```
    And client renders results with platform badges
    And user sees combined results in <500ms

  Scenario: Get AI-powered recommendations
    When user navigates to "For You" page
    Then client sends GET /api/v1/discovery/recommendations
    And Discovery Service queries SONA Engine:
      ```protobuf
      RecommendationRequest {
        user_id: "user-123"
        limit: 20
        filters: {
          exclude_watched: true
          platforms: ["netflix", "hulu", "disney+"]
        }
      }
      ```
    And SONA Engine retrieves user's watch history from PostgreSQL
    And SONA Engine generates user preference embedding
    And SONA Engine queries Qdrant for similar content vectors
    And Qdrant returns top 50 similar items
    And SONA Engine filters by platform availability
    And SONA Engine ranks by composite score (similarity * popularity * freshness)
    And SONA Engine returns top 20 recommendations
    And Discovery Service enriches with platform metadata
    And client displays personalized recommendations
    And user sees recommendations within 400ms
```

### 5.3 Cross-Platform Sync E2E Journey

```gherkin
Feature: End-to-End Cross-Platform Sync

  Background:
    Given user "user-123" has 3 devices registered:
      | Device ID  | Type        | Status |
      | mobile-abc | iPhone      | Online |
      | tv-def     | Smart TV    | Online |
      | tablet-ghi | iPad        | Online |
    And all devices are subscribed to PubNub channel "user.user-123.playback"

  Scenario: Sync playback position across devices
    Given user is watching "Stranger Things S01E01" on mobile at 00:12:34
    When user pauses video on mobile
    Then mobile app sends event to Playback Service:
      ```json
      {
        "event_type": "playback.paused",
        "user_id": "user-123",
        "device_id": "mobile-abc",
        "content_id": "netflix:80057281:s01e01",
        "timestamp_ms": 754000,
        "platform": "netflix"
      }
      ```
    And Playback Service validates event
    And Playback Service updates PostgreSQL:
      ```sql
      INSERT INTO playback_positions (user_id, content_id, position_ms, updated_at)
      VALUES ('user-123', 'netflix:80057281:s01e01', 754000, NOW())
      ON CONFLICT (user_id, content_id)
      DO UPDATE SET position_ms = 754000, updated_at = NOW();
      ```
    And Playback Service publishes to Sync Service
    And Sync Service updates Redis cache:
      ```redis
      SET playback:user-123:netflix:80057281:s01e01 754000 EX 86400
      ```
    And Sync Service publishes to PubNub:
      ```javascript
      pubnub.publish({
        channel: 'user.user-123.playback',
        message: {
          event: 'position_updated',
          content_id: 'netflix:80057281:s01e01',
          position_ms: 754000,
          source_device: 'mobile-abc'
        }
      })
      ```
    And PubNub delivers message to TV and tablet within 80ms
    And TV app receives update and displays toast: "Resume at 12:34?"
    And tablet app updates progress bar to 12:34
    And all devices show consistent playback position
    And total sync latency is <200ms

  Scenario: Resume playback on different device
    Given user paused "Stranger Things" at 12:34 on mobile 1 hour ago
    When user opens Netflix on Smart TV
    And user selects "Stranger Things S01E01"
    Then TV app sends GET /api/v1/playback/position?content_id=netflix:80057281:s01e01
    And API Gateway validates token
    And API Gateway forwards to Playback Service
    And Playback Service checks Redis cache:
      ```redis
      GET playback:user-123:netflix:80057281:s01e01
      # Returns: 754000
      ```
    And cache hit, Playback Service returns cached position
    And TV app receives response:
      ```json
      {
        "content_id": "netflix:80057281:s01e01",
        "position_ms": 754000,
        "last_updated": "2025-12-06T09:30:00Z",
        "last_device": "mobile-abc"
      }
      ```
    And TV app displays resume prompt: "Resume at 12:34?"
    And user clicks "Resume"
    And TV app starts playback at 00:12:34
    And TV app sends playback started event
    And sync process updates all devices
    And mobile app shows "Playing on Smart TV"
```

### 5.4 End-to-End Playback Session

```
┌─────────────────────────────────────────────────────────────────┐
│              Complete Playback Session Flow                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. User selects content                                       │
│     │                                                           │
│     ├─▶ Client → Discovery Service                            │
│     │   "Get content details"                                  │
│     │                                                           │
│  2. User clicks "Play"                                         │
│     │                                                           │
│     ├─▶ Client → Playback Service                             │
│     │   "Initialize playback session"                          │
│     │                                                           │
│     ├─▶ Playback Service → Platform (Netflix)                 │
│     │   "Get streaming URL"                                    │
│     │                                                           │
│  3. Video starts playing                                       │
│     │                                                           │
│     ├─▶ Client → Playback Service                             │
│     │   "Send heartbeat every 30s"                             │
│     │                                                           │
│     ├─▶ Playback Service → Sync Service                       │
│     │   "Update position every 30s"                            │
│     │                                                           │
│     ├─▶ Sync Service → PubNub                                 │
│     │   "Broadcast to all devices"                             │
│     │                                                           │
│  4. User pauses/resumes                                        │
│     │                                                           │
│     ├─▶ Events flow through same chain                        │
│     │                                                           │
│  5. User finishes watching                                     │
│     │                                                           │
│     ├─▶ Client → Playback Service                             │
│     │   "Mark as watched"                                      │
│     │                                                           │
│     ├─▶ Playback Service → PostgreSQL                         │
│     │   "Update watch history"                                 │
│     │                                                           │
│     ├─▶ Playback Service → SONA Engine                        │
│     │   "Update user preferences"                              │
│     │                                                           │
│     └─▶ SONA Engine → Qdrant                                  │
│         "Update user vector for better recommendations"        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

```gherkin
Feature: Complete Playback Session E2E

  Scenario: Full playback lifecycle
    Given user is logged in
    And user has Netflix connected
    When user searches for "The Crown"
    Then Discovery Service returns results
    When user selects "The Crown S01E01"
    Then client requests playback session:
      POST /api/v1/playback/session
      Body: {
        "content_id": "netflix:the-crown:s01e01",
        "platform": "netflix",
        "device_id": "mobile-abc"
      }
    And Playback Service creates session in PostgreSQL
    And Playback Service requests streaming URL from Netflix (mock)
    And Playback Service returns session token and URL
    And client initializes video player with URL
    When video starts playing
    Then client sends heartbeat every 30 seconds:
      PUT /api/v1/playback/session/{session_id}/heartbeat
      Body: {
        "position_ms": 45000,
        "state": "playing"
      }
    And Playback Service updates session position
    And Playback Service forwards to Sync Service
    And Sync Service broadcasts position to all devices
    When user pauses at 5:30
    Then client sends pause event
    And Sync Service updates all devices
    When user resumes
    Then playback continues from 5:30
    When user finishes episode (reaches 95% or end)
    Then Playback Service marks content as watched
    And watch history is updated in PostgreSQL
    And SONA Engine updates user preference vector
    And Qdrant stores updated user embedding
    And user sees "Episode Watched ✓" in UI
    And next episode is recommended
```

---

## 6. Integration Environment Specifications

### 6.1 Staging Environment Architecture

```yaml
staging_environment:

  infrastructure:
    cloud_provider: "AWS"
    region: "us-east-1"
    vpc:
      cidr: "10.1.0.0/16"
      subnets:
        - public_1a: "10.1.1.0/24"
        - public_1b: "10.1.2.0/24"
        - private_1a: "10.1.10.0/24"
        - private_1b: "10.1.20.0/24"

  services:
    api_gateway:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "512m"
        memory: "1GB"
      health_check: "GET /health"

    mcp_server:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "512m"
        memory: "1GB"

    discovery_service:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "1024m"
        memory: "2GB"

    sona_engine:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "2048m"
        memory: "4GB"  # Higher for ML models

    sync_service:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "512m"
        memory: "1GB"

    auth_service:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "512m"
        memory: "1GB"

    ingestion_service:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "1024m"
        memory: "2GB"

    playback_service:
      deployment: "ECS Fargate"
      replicas: 2
      resources:
        cpu: "512m"
        memory: "1GB"

  databases:
    postgresql:
      service: "Amazon RDS"
      instance_type: "db.t3.medium"
      engine_version: "15.4"
      storage: "100GB gp3"
      multi_az: true
      backup_retention: 7

    redis:
      service: "Amazon ElastiCache"
      node_type: "cache.t3.medium"
      engine_version: "7.0"
      num_cache_nodes: 2

    qdrant:
      deployment: "ECS Fargate"
      replicas: 3
      resources:
        cpu: "2048m"
        memory: "8GB"
      storage: "EBS 200GB"

  external_services:
    pubnub:
      tier: "Developer (Free)"
      max_messages: "1M/month"

    spotify:
      credentials: "Staging API Keys"

    apple_music:
      credentials: "Developer Tokens"

  monitoring:
    - "CloudWatch Logs"
    - "CloudWatch Metrics"
    - "X-Ray Tracing"
    - "Health Check Dashboard"

  networking:
    load_balancer: "Application Load Balancer"
    ssl_certificate: "ACM Certificate (staging.mediagateway.com)"
    dns: "Route53"
```

### 6.2 Test Data Fixtures

```yaml
test_data_fixtures:

  users:
    - id: "test-user-001"
      email: "test1@example.com"
      password_hash: "$2b$12$..." # Password: TestPass123!
      name: "Test User 1"
      roles: ["user"]
      created_at: "2025-01-01T00:00:00Z"

    - id: "test-user-002"
      email: "premium@example.com"
      password_hash: "$2b$12$..."
      name: "Premium User"
      roles: ["user", "premium"]
      connected_platforms:
        - platform: "netflix"
          account_id: "netflix-123"
          status: "active"
        - platform: "spotify"
          account_id: "spotify-456"
          status: "active"

  content_catalog:
    - id: "netflix:80057281"
      platform: "netflix"
      title: "Stranger Things"
      type: "series"
      seasons: 4
      genres: ["sci-fi", "horror", "drama"]
      release_year: 2016
      rating: 8.7

    - id: "spotify:album-123"
      platform: "spotify"
      title: "Stranger Things Soundtrack"
      type: "album"
      artist: "Various Artists"
      tracks: 15

  watch_history:
    - user_id: "test-user-002"
      content_id: "netflix:80057281:s01e01"
      watched_at: "2025-12-01T10:00:00Z"
      position_ms: 754000
      completed: false

    - user_id: "test-user-002"
      content_id: "netflix:the-crown:s01e01"
      watched_at: "2025-11-30T20:00:00Z"
      completed: true

  vector_embeddings:
    # Pre-computed embeddings for common content
    - content_id: "netflix:80057281"
      embedding: [0.12, 0.34, 0.56, ...]  # 768 dims

  mock_api_responses:
    spotify_search:
      query: "taylor swift"
      response_file: "fixtures/spotify/search_taylor_swift.json"

    netflix_catalog:
      response_file: "fixtures/netflix/catalog_sample.json"

fixture_loading:
  strategy: "Testcontainers with SQL scripts"

  postgresql_init:
    - "fixtures/sql/01_schema.sql"
    - "fixtures/sql/02_users.sql"
    - "fixtures/sql/03_content.sql"
    - "fixtures/sql/04_history.sql"

  redis_init:
    - "fixtures/redis/sessions.redis"
    - "fixtures/redis/cache.redis"

  qdrant_init:
    - "fixtures/qdrant/embeddings.json"
```

### 6.3 Mock Service Configurations

```yaml
mock_services:

  wiremock_config:
    version: "2.35.0"
    port: 9000

    mappings:
      - name: "Spotify Search API"
        request:
          method: GET
          urlPattern: "/v1/search?.*"
          headers:
            Authorization:
              matches: "Bearer .*"
        response:
          status: 200
          jsonBody:
            artists:
              items: [...]
          headers:
            Content-Type: "application/json"
          fixedDelayMilliseconds: 100

      - name: "Netflix Catalog API"
        request:
          method: GET
          urlPath: "/catalog/search"
        response:
          status: 200
          bodyFileName: "netflix_catalog.json"

      - name: "Rate Limit Test"
        request:
          method: GET
          urlPath: "/rate-limited"
        response:
          status: 429
          headers:
            Retry-After: "30"
          body: "Rate limit exceeded"

  vcr_config:
    record_mode: "once"  # Record once, replay thereafter
    cassette_library_dir: "fixtures/vcr_cassettes"
    filter_sensitive_data:
      - pattern: "Bearer [A-Za-z0-9-._~+/]+"
        replacement: "Bearer REDACTED"
      - pattern: "api_key=[A-Za-z0-9]+"
        replacement: "api_key=REDACTED"

    cassettes:
      - name: "spotify_artist_search"
        path: "spotify/artist_search_beatles.yml"

      - name: "apple_music_library"
        path: "apple/user_library_songs.yml"
```

### 6.4 Testcontainers Specifications

```java
// Testcontainers configuration for integration tests
public class IntegrationTestEnvironment {

    @Container
    public static PostgreSQLContainer<?> postgres = new PostgreSQLContainer<>("postgres:15")
        .withDatabaseName("media_gateway_test")
        .withUsername("test")
        .withPassword("test")
        .withInitScript("fixtures/sql/schema.sql");

    @Container
    public static GenericContainer<?> redis = new GenericContainer<>("redis:7-alpine")
        .withExposedPorts(6379);

    @Container
    public static GenericContainer<?> qdrant = new GenericContainer<>("qdrant/qdrant:v1.7.0")
        .withExposedPorts(6333, 6334)
        .withEnv("QDRANT__SERVICE__HTTP_PORT", "6333");

    @Container
    public static MockServerContainer mockServer = new MockServerContainer(
        DockerImageName.parse("mockserver/mockserver:5.15.0")
    ).withExposedPorts(1080);

    @BeforeAll
    public static void setUp() {
        // Configure services to use Testcontainers
        System.setProperty("DB_URL", postgres.getJdbcUrl());
        System.setProperty("REDIS_URL", "redis://" + redis.getHost() + ":" + redis.getFirstMappedPort());
        System.setProperty("QDRANT_URL", "http://" + qdrant.getHost() + ":" + qdrant.getMappedPort(6333));
        System.setProperty("MOCK_API_URL", "http://" + mockServer.getHost() + ":" + mockServer.getFirstMappedPort());

        // Load test data
        loadFixtures();
    }

    private static void loadFixtures() {
        // Load SQL fixtures
        // Load Redis data
        // Load Qdrant vectors
        // Configure mock API responses
    }
}
```

```yaml
docker_compose_test_env:
  version: "3.9"

  services:
    postgres:
      image: postgres:15
      environment:
        POSTGRES_DB: media_gateway_test
        POSTGRES_USER: test
        POSTGRES_PASSWORD: test
      ports:
        - "5432:5432"
      volumes:
        - ./fixtures/sql:/docker-entrypoint-initdb.d

    redis:
      image: redis:7-alpine
      ports:
        - "6379:6379"

    qdrant:
      image: qdrant/qdrant:v1.7.0
      ports:
        - "6333:6333"
        - "6334:6334"
      volumes:
        - qdrant_data:/qdrant/storage

    wiremock:
      image: wiremock/wiremock:2.35.0
      ports:
        - "9000:8080"
      volumes:
        - ./fixtures/wiremock:/home/wiremock
      command: ["--global-response-templating", "--verbose"]

    # All microservices in test mode
    api-gateway:
      build:
        context: ./services/api-gateway
        dockerfile: Dockerfile.test
      environment:
        NODE_ENV: test
        AUTH_SERVICE_URL: http://auth:8084
        DISCOVERY_SERVICE_URL: http://discovery:8081
      ports:
        - "8080:8080"
      depends_on:
        - postgres
        - redis

    # ... other services ...

  volumes:
    qdrant_data:
```

---

## 7. Integration Failure Handling

### 7.1 Circuit Breaker Validation

```gherkin
Feature: Circuit Breaker Pattern Validation

  Background:
    Given Discovery Service has circuit breaker configured for Netflix API:
      ```yaml
      circuit_breaker:
        failure_threshold: 5      # Open after 5 failures
        timeout: 3000            # 3 second timeout
        reset_timeout: 30000     # Try again after 30s
        half_open_requests: 3    # Test with 3 requests
      ```

  Scenario: Circuit breaker opens after consecutive failures
    Given Netflix API is returning 500 errors
    When Discovery Service makes 1st request
    Then request fails, circuit breaker state: CLOSED
    When Discovery Service makes 2nd request
    Then request fails, circuit breaker state: CLOSED
    When Discovery Service makes 3rd request
    Then request fails, circuit breaker state: CLOSED
    When Discovery Service makes 4th request
    Then request fails, circuit breaker state: CLOSED
    When Discovery Service makes 5th request
    Then request fails, circuit breaker state: OPEN
    And circuit breaker blocks subsequent requests
    When Discovery Service makes 6th request
    Then request is rejected immediately without calling Netflix
    And response includes error:
      ```json
      {
        "error": "circuit_breaker_open",
        "message": "Netflix service is temporarily unavailable",
        "retry_after": 30
      }
      ```
    And response time is <10ms (fast fail)

  Scenario: Circuit breaker transitions to half-open
    Given circuit breaker is OPEN for Netflix API
    When 30 seconds pass (reset_timeout)
    Then circuit breaker transitions to HALF_OPEN
    And circuit breaker allows 3 test requests (half_open_requests)

    # Success scenario
    When all 3 test requests succeed
    Then circuit breaker transitions to CLOSED
    And normal traffic resumes

    # Failure scenario
    When 1 of 3 test requests fails
    Then circuit breaker immediately transitions to OPEN
    And reset timer restarts for another 30 seconds

  Scenario: Per-service circuit breaker isolation
    Given circuit breaker is OPEN for Netflix
    And circuit breaker is CLOSED for Spotify
    When Discovery Service searches for content
    Then Spotify API calls succeed normally
    And Netflix API calls are short-circuited
    And response includes partial results from Spotify only
    And user sees warning: "Netflix results unavailable"
```

#### 7.1.1 Circuit Breaker Implementation Example

```typescript
// Circuit breaker implementation in Discovery Service
class CircuitBreaker {
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';
  private failureCount: number = 0;
  private lastFailureTime: number = 0;
  private halfOpenAttempts: number = 0;

  constructor(
    private failureThreshold: number = 5,
    private timeout: number = 3000,
    private resetTimeout: number = 30000,
    private halfOpenRequests: number = 3
  ) {}

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime >= this.resetTimeout) {
        this.state = 'HALF_OPEN';
        this.halfOpenAttempts = 0;
      } else {
        throw new Error('Circuit breaker is OPEN');
      }
    }

    try {
      const result = await Promise.race([
        fn(),
        this.timeoutPromise()
      ]);

      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess() {
    if (this.state === 'HALF_OPEN') {
      this.halfOpenAttempts++;
      if (this.halfOpenAttempts >= this.halfOpenRequests) {
        this.state = 'CLOSED';
        this.failureCount = 0;
      }
    } else {
      this.failureCount = 0;
    }
  }

  private onFailure() {
    this.failureCount++;
    this.lastFailureTime = Date.now();

    if (this.state === 'HALF_OPEN' || this.failureCount >= this.failureThreshold) {
      this.state = 'OPEN';
    }
  }

  private timeoutPromise(): Promise<never> {
    return new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Request timeout')), this.timeout)
    );
  }
}
```

### 7.2 Fallback Behavior Testing

```gherkin
Feature: Service Fallback Mechanisms

  Scenario: Discovery Service falls back to cache when platforms unavailable
    Given user searches for "inception"
    And Discovery Service has cached results from 10 minutes ago
    And Netflix and Hulu APIs are both unavailable
    When Discovery Service attempts to search platforms
    Then Netflix API call fails (circuit breaker open)
    And Hulu API call fails (circuit breaker open)
    And Discovery Service falls back to cached results:
      ```
      1. Check Redis cache: search:inception:user-123
      2. Cache hit, retrieve cached results
      3. Add metadata: { "cached": true, "age_minutes": 10 }
      4. Return cached results to client
      ```
    And response includes header: X-Cache-Status: HIT
    And response includes warning:
      ```json
      {
        "warnings": [{
          "code": "stale_results",
          "message": "Results may be outdated. Some platforms are unavailable."
        }]
      }
      ```
    And user sees results with "Cached results" badge

  Scenario: SONA Engine falls back to popularity when vector search fails
    Given user requests recommendations
    And Qdrant vector database is unavailable
    When SONA Engine attempts vector similarity search
    Then Qdrant connection fails
    And SONA Engine catches error
    And SONA Engine falls back to popularity-based recommendations:
      ```sql
      SELECT c.id, c.title, c.platform, COUNT(w.id) as watch_count
      FROM content c
      JOIN watch_history w ON c.id = w.content_id
      WHERE c.platform IN (SELECT platform FROM user_platforms WHERE user_id = $1)
      AND c.id NOT IN (SELECT content_id FROM watch_history WHERE user_id = $1)
      GROUP BY c.id
      ORDER BY watch_count DESC, c.release_date DESC
      LIMIT 20;
      ```
    And results are returned from PostgreSQL
    And response includes metadata:
      ```json
      {
        "algorithm": "popularity_fallback",
        "reason": "Vector search unavailable"
      }
      ```
    And recommendations are less personalized but still relevant

  Scenario: Sync Service falls back to polling when PubNub fails
    Given user's devices are syncing via PubNub
    And PubNub service becomes unavailable
    When device tries to subscribe to PubNub channel
    Then subscription fails with connection error
    And device falls back to HTTP polling:
      ```
      Every 5 seconds:
        GET /api/v1/sync/state?since={last_update_timestamp}
      ```
    And Sync Service returns state changes since last poll
    And device continues to function (degraded experience)
    And when PubNub becomes available:
      - Device reconnects to PubNub
      - Switches back to real-time updates
      - Stops polling
```

### 7.3 Graceful Degradation Verification

```yaml
graceful_degradation_tests:

  test_partial_platform_availability:
    scenario: "User has 5 connected platforms, 2 are down"
    given:
      - "User connected to: Netflix, Hulu, Disney+, Spotify, Apple Music"
      - "Netflix API: Unavailable (500 errors)"
      - "Hulu API: Unavailable (timeout)"
      - "Disney+, Spotify, Apple Music: Available"
    when:
      - "User searches for 'star wars'"
    then:
      - "Discovery Service queries all 5 platforms in parallel"
      - "Netflix query fails → circuit breaker opens"
      - "Hulu query times out → circuit breaker opens"
      - "Disney+ returns 10 results"
      - "Spotify returns 2 results"
      - "Apple Music returns 3 results"
      - "Response includes 15 total results (not 0)"
      - "Response includes errors array:"
        ```json
        {
          "results": [...15 items...],
          "errors": [
            {"platform": "netflix", "error": "unavailable"},
            {"platform": "hulu", "error": "timeout"}
          ],
          "partial_results": true
        }
        ```
      - "User experience: Degraded but functional"

  test_database_read_replica_failover:
    scenario: "Primary database fails, failover to read replica"
    given:
      - "PostgreSQL primary: Unavailable"
      - "PostgreSQL read replica: Available"
    when:
      - "Discovery Service attempts to read user data"
    then:
      - "Connection to primary fails"
      - "Connection pool detects failure within 5s"
      - "Service switches to read replica connection string"
      - "Read operations continue successfully"
      - "Write operations fail with error: 'Database in read-only mode'"
      - "Critical writes are queued for retry"
      - "Non-critical writes are skipped"
      - "System remains operational for read-heavy workloads"

  test_redis_cache_unavailable:
    scenario: "Redis cache completely unavailable"
    given:
      - "Redis cluster is down"
      - "All services configured to use Redis for caching"
    when:
      - "Any service attempts cache read/write"
    then:
      - "Redis connection fails immediately"
      - "Service catches exception"
      - "Service bypasses cache, queries database directly"
      - "Response latency increases (50ms → 200ms)"
      - "Database load increases"
      - "But system continues to function"
      - "Monitoring alerts: 'Redis unavailable, degraded performance'"

  test_external_api_complete_failure:
    scenario: "All streaming platforms APIs unavailable"
    given:
      - "All platform APIs: Netflix, Hulu, Disney+ down"
    when:
      - "User searches for content"
    then:
      - "Discovery Service attempts all API calls"
      - "All calls fail (circuit breakers open)"
      - "Discovery Service falls back to local database:"
        ```sql
        SELECT * FROM cached_content
        WHERE title ILIKE '%search_query%'
        AND last_updated > NOW() - INTERVAL '7 days';
        ```
      - "Returns results from local cache (may be stale)"
      - "Response includes prominent warning:"
        ```json
        {
          "results": [...],
          "warning": "Showing cached results. Live platform data is unavailable.",
          "data_age": "2 hours"
        }
        ```
      - "User can still browse and queue content"
      - "Playback may fail (no streaming URLs available)"

graceful_degradation_priorities:
  critical_functions:  # Must remain operational
    - "User authentication"
    - "View cached content"
    - "Read watch history"
    - "View queue"

  degraded_functions:  # Acceptable to fail gracefully
    - "Live platform search (use cache)"
    - "Real-time recommendations (use popularity)"
    - "Cross-device sync (use polling)"
    - "Playback analytics (queue for later)"

  optional_functions:  # Can disable temporarily
    - "AI-powered recommendations"
    - "Content metadata enrichment"
    - "Usage statistics"
    - "A/B testing"
```

### 7.4 Retry Logic Validation

```gherkin
Feature: Exponential Backoff Retry Logic

  Scenario: Retry transient failures with exponential backoff
    Given Discovery Service calls Netflix API
    And Netflix returns 503 Service Unavailable (transient error)
    When Discovery Service receives 503 response
    Then Discovery Service initiates retry with exponential backoff:
      | Attempt | Delay  | Action                    |
      | 1       | 0ms    | Immediate (original call) |
      | 2       | 100ms  | Retry with backoff        |
      | 3       | 200ms  | Retry with backoff        |
      | 4       | 400ms  | Retry with backoff        |
    And each retry includes jitter: delay * random(0.5, 1.5)
    And if 4th attempt succeeds, return results
    And if all attempts fail, circuit breaker opens
    And total max retry time: 700ms

  Scenario: Do NOT retry non-retryable errors
    Given Discovery Service calls Spotify API with invalid token
    And Spotify returns 401 Unauthorized
    When Discovery Service receives 401 response
    Then Discovery Service does NOT retry (authorization error is permanent)
    And error is immediately returned to client
    And client can handle auth refresh

  Scenario: Retry with idempotency key
    Given Ingestion Service posts content to database
    And request includes idempotency key: "ing-req-abc-123"
    When network fails after request sent but before response received
    Then Ingestion Service retries request with same idempotency key
    And database checks idempotency_keys table:
      ```sql
      SELECT * FROM idempotency_keys WHERE key = 'ing-req-abc-123';
      ```
    And if key exists, database returns cached response (no duplicate insert)
    And if key does not exist, database processes request normally
    And idempotent retry prevents duplicate data
```

---

## 8. Acceptance Criteria

### 8.1 Integration Test Coverage Requirements

```yaml
integration_test_coverage_criteria:

  service_to_service:
    target: 95%
    measurement: "Contract test coverage for all API interactions"
    acceptance:
      - "All service pairs have contract tests"
      - "All error scenarios are tested"
      - "All timeout scenarios are tested"

  external_apis:
    target: 85%
    measurement: "Mock/VCR coverage for external platform APIs"
    acceptance:
      - "Happy path scenarios covered"
      - "Rate limiting scenarios covered"
      - "Authentication refresh scenarios covered"
      - "Error handling scenarios covered"

  database_integration:
    target: 90%
    measurement: "Testcontainers test coverage for DB operations"
    acceptance:
      - "Connection pool management tested"
      - "Transaction consistency tested"
      - "Migration compatibility tested"
      - "Read replica failover tested"

  end_to_end:
    target: 80%
    measurement: "User journey coverage"
    acceptance:
      - "All critical user journeys have E2E tests"
      - "Cross-service workflows tested"
      - "Multi-device sync tested"
```

### 8.2 Performance Acceptance Criteria

```yaml
integration_performance_criteria:

  response_times:
    api_gateway_routing:
      p50: "<50ms"
      p95: "<100ms"
      p99: "<200ms"

    service_to_service_grpc:
      p50: "<20ms"
      p95: "<50ms"
      p99: "<100ms"

    end_to_end_search:
      p50: "<300ms"
      p95: "<500ms"
      p99: "<1000ms"

    cross_platform_sync:
      p50: "<100ms"
      p95: "<200ms"
      p99: "<500ms"

  throughput:
    api_gateway:
      target: "1000 req/s sustained"
      burst: "2000 req/s for 60s"

    discovery_service:
      target: "500 searches/s"

    sync_service:
      target: "5000 events/s"

  database:
    postgresql_connection_pool:
      checkout_time_p95: "<10ms"
      max_connections: "10 per service instance"

    redis_operations:
      get_p95: "<1ms"
      set_p95: "<2ms"

    qdrant_vector_search:
      search_p95: "<50ms"
      batch_insert_1000: "<5s"
```

### 8.3 Reliability Acceptance Criteria

```yaml
reliability_criteria:

  circuit_breakers:
    - "All external API calls protected by circuit breakers"
    - "Circuit breaker opens after 5 consecutive failures"
    - "Circuit breaker transitions to half-open after 30s"
    - "Circuit breaker metrics exported to monitoring"

  retries:
    - "Transient errors retried with exponential backoff"
    - "Max 3 retries for API calls"
    - "Non-retryable errors (4xx) fail immediately"
    - "Idempotency keys used for write operations"

  graceful_degradation:
    - "Partial platform failures return partial results"
    - "Cache fallback when APIs unavailable"
    - "System remains operational with degraded features"
    - "User-visible warnings for degraded state"

  fault_tolerance:
    - "Single service failure does not bring down system"
    - "Database read replica failover < 30s"
    - "Redis unavailability does not cause errors (bypass cache)"
    - "PubNub failure falls back to polling"
```

### 8.4 Test Execution Requirements

```yaml
test_execution_criteria:

  integration_test_suite:
    total_tests: "~440 integration tests"
    execution_time: "<15 minutes (parallel execution)"
    parallelization: "8-10 workers"
    flakiness_rate: "<2%"

  ci_pipeline:
    stages:
      - name: "Unit Tests"
        time: "3 minutes"
      - name: "Integration Tests"
        time: "15 minutes"
        requires: "Testcontainers + Docker"
      - name: "E2E Tests"
        time: "10 minutes"
        requires: "Full staging environment"

    failure_handling:
      - "Automatic retry for flaky tests (max 1 retry)"
      - "Test failure blocks deployment"
      - "Test reports published to dashboard"

  test_data:
    - "Fixtures loaded in <30s"
    - "Test isolation: Each test uses separate database schema"
    - "Cleanup: All test data removed after test completion"

  test_environments:
    - "Local: Docker Compose + Testcontainers"
    - "CI: GitHub Actions with Docker"
    - "Staging: Full AWS environment"
```

### 8.5 Documentation Requirements

```yaml
documentation_criteria:

  integration_test_docs:
    required_sections:
      - "Test architecture overview"
      - "Running tests locally"
      - "CI/CD pipeline configuration"
      - "Test data fixtures guide"
      - "Troubleshooting common failures"
      - "Adding new integration tests"

  contract_specifications:
    - "OpenAPI specs for all REST APIs"
    - "Protobuf definitions for all gRPC services"
    - "AsyncAPI specs for PubNub events"
    - "Contract test examples"

  runbooks:
    - "Service deployment procedures"
    - "Database migration procedures"
    - "Incident response playbooks"
    - "Monitoring and alerting guide"
```

---

## Summary

This SPARC Completion Phase Part 2 specification provides comprehensive integration validation requirements for the Media Gateway platform:

### Key Deliverables

1. **Service Integration Matrix**: Complete mapping of all 8 microservice interactions
2. **External API Validation**: Contracts for Spotify, Apple Music, Netflix, HBO, Disney+, Hulu, Prime Video, PubNub, Qdrant
3. **Database Integration**: PostgreSQL, Redis, and Qdrant consistency tests
4. **E2E User Journeys**: Authentication, discovery, sync, and playback flows
5. **Test Environments**: Staging infrastructure, fixtures, mocks, and Testcontainers
6. **Failure Handling**: Circuit breakers, fallbacks, and graceful degradation

### Integration Test Scope

- **~440 integration tests** across 7 categories
- **95% service contract coverage**
- **85% external API coverage**
- **90% database integration coverage**
- **80% E2E user journey coverage**

### Performance Targets

- API Gateway routing: <100ms (p95)
- Service-to-service gRPC: <50ms (p95)
- End-to-end search: <500ms (p95)
- Cross-platform sync: <200ms (p95)

### Reliability Requirements

- Circuit breakers on all external calls
- Exponential backoff retry logic
- Graceful degradation with cache fallback
- <2% test flakiness rate

This specification ensures the Media Gateway platform integrates seamlessly across all services, external platforms, and databases while maintaining high reliability and performance standards.

---

**Next Steps**: Proceed to implementation phase using this specification as the blueprint for integration test development.
