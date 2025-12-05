# AI-Native Media Discovery Platform - System Architecture

**Version:** 1.0
**Date:** 2025-12-05
**Status:** Design Phase

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Overview](#system-overview)
3. [Architecture Diagrams](#architecture-diagrams)
4. [Core Components](#core-components)
5. [Data Flow & Processing Pipelines](#data-flow--processing-pipelines)
6. [API Structure & Integration](#api-structure--integration)
7. [Performance & Caching Strategy](#performance--caching-strategy)
8. [Scalability & Fault Tolerance](#scalability--fault-tolerance)
9. [Security & Privacy](#security--privacy)
10. [Database Architecture](#database-architecture)
11. [Real-Time Availability System](#real-time-availability-system)
12. [Technology Stack](#technology-stack)
13. [Architecture Decision Records](#architecture-decision-records)
14. [Deployment Architecture](#deployment-architecture)
15. [Monitoring & Observability](#monitoring--observability)

---

## Executive Summary

This document defines the technical architecture for an AI-native media discovery platform capable of processing natural language queries, aggregating content from multiple streaming platforms, and providing instant verified answers to millions of users.

**Key Architecture Principles:**
- **Hybrid Architecture**: Microservices for core services + Serverless for elastic scaling
- **Event-Driven Design**: Asynchronous processing for real-time updates
- **CQRS Pattern**: Separate read/write paths for optimal performance
- **Multi-Layer Caching**: Aggressive caching at multiple levels
- **API-First Design**: Clean contracts between all components

**Quality Attributes:**
- **Performance**: <200ms p95 response time for queries
- **Scalability**: Support 10M+ concurrent users
- **Availability**: 99.99% uptime SLA
- **Consistency**: Eventual consistency for content, strong for user data
- **Security**: Zero-trust architecture, end-to-end encryption

---

## System Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Layer                              │
│  [Web App] [Mobile Apps] [Smart TV] [Voice Assistants]          │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    API Gateway Layer                             │
│  [Kong/AWS API Gateway] - Auth, Rate Limit, Routing             │
└────────────────────────┬────────────────────────────────────────┘
                         │
            ┌────────────┴────────────┐
            ▼                         ▼
┌──────────────────────┐    ┌──────────────────────┐
│   Query Service      │    │  User Service        │
│   (Microservice)     │    │  (Microservice)      │
└──────────┬───────────┘    └──────────┬───────────┘
           │                           │
           ▼                           ▼
┌──────────────────────┐    ┌──────────────────────┐
│  AI/NLP Engine       │    │  Content Catalog     │
│  (Serverless)        │    │  (Microservice)      │
└──────────┬───────────┘    └──────────┬───────────┘
           │                           │
           ▼                           ▼
┌──────────────────────────────────────────────────┐
│              Data & Cache Layer                   │
│  [Vector DB] [PostgreSQL] [Redis] [CDN]          │
└──────────────────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────┐
│         External Integration Layer                │
│  [Netflix API] [Hulu API] [Disney+ API] etc.     │
└──────────────────────────────────────────────────┘
```

### Architecture Style: **Hybrid Microservices + Serverless**

**Rationale:**
- Microservices for stateful, core business logic (User Service, Content Catalog)
- Serverless for variable workloads (AI processing, content verification)
- Event-driven for real-time updates and decoupling
- Cost-effective scaling based on actual usage patterns

---

## Architecture Diagrams

### 1. System Context Diagram (C4 Level 1)

```
                    ┌─────────────────┐
                    │     End Users   │
                    │ (Web/Mobile/TV) │
                    └────────┬────────┘
                             │
                             │ HTTPS
                             ▼
              ┌──────────────────────────────┐
              │  Media Discovery Platform    │
              │                              │
              │  - Natural Language Search   │
              │  - Content Aggregation       │
              │  - Personalization           │
              │  - Real-time Verification    │
              └──────┬───────────────────────┘
                     │
        ┌────────────┼────────────┐
        │            │            │
        ▼            ▼            ▼
  ┌─────────┐  ┌─────────┐  ┌─────────┐
  │Netflix  │  │  Hulu   │  │Disney+  │
  │   API   │  │   API   │  │   API   │
  └─────────┘  └─────────┘  └─────────┘
```

### 2. Container Diagram (C4 Level 2)

```
┌──────────────────────────────────────────────────────────────────┐
│                         Client Applications                       │
│  [React Web App] [React Native Mobile] [Smart TV App]            │
└───────────────────────────┬──────────────────────────────────────┘
                            │ HTTPS/REST/GraphQL
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                      API Gateway (Kong)                           │
│  - Authentication (JWT)                                           │
│  - Rate Limiting                                                  │
│  - Request Routing                                                │
│  - API Versioning                                                 │
└───────────────────────────┬──────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│ Query Service │  │ User Service  │  │Content Service│
│               │  │               │  │               │
│ - NL Query    │  │ - Auth        │  │ - Catalog     │
│ - Intent      │  │ - Profiles    │  │ - Metadata    │
│   Detection   │  │ - Preferences │  │ - Search Index│
│ - Search      │  │ - History     │  │               │
│   Execution   │  │               │  │               │
└───────┬───────┘  └───────┬───────┘  └───────┬───────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│AI/NLP Engine  │  │ Recommendation│  │Availability   │
│(AWS Lambda)   │  │    Engine     │  │  Verifier     │
│               │  │  (Serverless) │  │  (Serverless) │
│- GPT-4/Claude │  │               │  │               │
│- Embeddings   │  │- Collaborative│  │- Real-time    │
│- Intent Class │  │- Content-based│  │  API Polling  │
└───────┬───────┘  └───────┬───────┘  └───────┬───────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┼───────────────────────────────┐
        │                   │                               │
        ▼                   ▼                               ▼
┌───────────────┐  ┌───────────────┐           ┌───────────────┐
│  Vector DB    │  │  PostgreSQL   │           │  Redis Cache  │
│  (Pinecone)   │  │               │           │               │
│               │  │ - Users       │           │ - Query Cache │
│- Embeddings   │  │ - Content     │           │ - Session     │
│- Semantic     │  │ - Relations   │           │ - Hot Data    │
│  Search       │  │               │           │               │
└───────────────┘  └───────────────┘           └───────────────┘
        │                   │                               │
        └───────────────────┼───────────────────────────────┘
                            │
                    ┌───────┴───────┐
                    │  Event Bus    │
                    │  (Kafka)      │
                    │               │
                    │- Content Sync │
                    │- User Events  │
                    │- Availability │
                    └───────────────┘
```

### 3. Component Diagram - Query Service (C4 Level 3)

```
┌──────────────────────────────────────────────────────────────┐
│                      Query Service                            │
│                                                               │
│  ┌────────────────┐         ┌────────────────┐              │
│  │ Query Parser   │────────▶│  Intent        │              │
│  │                │         │  Classifier    │              │
│  │ - Tokenization │         │                │              │
│  │ - Entity Ext.  │         │ - Search       │              │
│  │ - Normalization│         │ - Browse       │              │
│  └────────┬───────┘         │ - Recommend    │              │
│           │                 └────────┬───────┘              │
│           ▼                          │                       │
│  ┌────────────────┐                 │                       │
│  │ Query Enricher │◀────────────────┘                       │
│  │                │                                          │
│  │ - User Context │                                          │
│  │ - Location     │                                          │
│  │ - Preferences  │                                          │
│  └────────┬───────┘                                          │
│           │                                                  │
│           ▼                                                  │
│  ┌────────────────┐         ┌────────────────┐              │
│  │ Search Engine  │────────▶│ Result Ranker  │              │
│  │                │         │                │              │
│  │ - Vector Search│         │ - Relevance    │              │
│  │ - Hybrid Search│         │ - Personalize  │              │
│  │ - Filters      │         │ - Availability │              │
│  └────────────────┘         └────────┬───────┘              │
│                                      │                       │
│                                      ▼                       │
│                             ┌────────────────┐              │
│                             │ Response       │              │
│                             │ Formatter      │              │
│                             │                │              │
│                             │ - JSON/GraphQL │              │
│                             │ - Enrichment   │              │
│                             └────────────────┘              │
└──────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. API Gateway Layer

**Technology:** Kong API Gateway (Primary) + AWS API Gateway (Backup)

**Responsibilities:**
- Request routing and load balancing
- Authentication and authorization (JWT validation)
- Rate limiting and throttling
- API versioning and backward compatibility
- Request/response transformation
- SSL/TLS termination
- DDoS protection

**Configuration:**
```yaml
# Kong Gateway Configuration
services:
  - name: query-service
    url: http://query-service:8080
    routes:
      - paths: ['/api/v1/search']
        methods: ['GET', 'POST']
    plugins:
      - name: rate-limiting
        config:
          minute: 60
          hour: 1000
      - name: jwt
      - name: cors
```

### 2. Query Service (Microservice)

**Technology:** Node.js + TypeScript + Express

**Core Functions:**
- Natural language query parsing
- Intent classification (search, browse, recommend)
- Query enrichment with user context
- Semantic search orchestration
- Result ranking and filtering
- Response formatting

**API Endpoints:**
```typescript
POST /api/v1/search
  Request: { query: string, context?: UserContext }
  Response: { results: Content[], metadata: SearchMetadata }

GET /api/v1/search/suggestions
  Query: { partial: string }
  Response: { suggestions: string[] }

POST /api/v1/search/semantic
  Request: { embedding: number[], filters: Filter[] }
  Response: { results: Content[] }
```

**Scalability:**
- Horizontal scaling with Kubernetes HPA
- Stateless design for easy replication
- Circuit breakers for downstream dependencies

### 3. AI/NLP Engine (Serverless)

**Technology:** AWS Lambda + Python + OpenAI API / Anthropic Claude

**Core Functions:**
- Query understanding and intent detection
- Named entity recognition (movies, genres, actors)
- Embedding generation for semantic search
- Query expansion and reformulation
- Context-aware result re-ranking

**Architecture:**
```python
# Lambda Function Structure
def query_handler(event, context):
    query = event['query']

    # Intent Classification
    intent = classify_intent(query)  # GPT-4 API

    # Entity Extraction
    entities = extract_entities(query)  # NER model

    # Generate Embedding
    embedding = generate_embedding(query)  # text-embedding-3-large

    # Query Enrichment
    enriched = enrich_query(query, entities, user_context)

    return {
        'intent': intent,
        'entities': entities,
        'embedding': embedding,
        'enriched_query': enriched
    }
```

**Cost Optimization:**
- Warm container reuse
- Batch processing for embeddings
- Caching for common queries
- Model selection based on complexity

### 4. User Service (Microservice)

**Technology:** Go + gRPC

**Core Functions:**
- User authentication and authorization
- Profile management
- Viewing history tracking
- Preference management
- Subscription tracking
- Personalization data

**Data Model:**
```go
type User struct {
    ID              string
    Email           string
    PasswordHash    string
    Profile         UserProfile
    Preferences     UserPreferences
    ViewingHistory  []ViewingRecord
    Subscriptions   []Subscription
    CreatedAt       time.Time
    UpdatedAt       time.Time
}

type UserPreferences struct {
    Genres          []string
    Languages       []string
    ContentRating   string
    Platforms       []string
    NotificationSettings NotificationConfig
}
```

**Security:**
- Password hashing with bcrypt (cost factor 12)
- JWT with RS256 signing
- Refresh token rotation
- Rate limiting per user

### 5. Content Service (Microservice)

**Technology:** Node.js + TypeScript + GraphQL

**Core Functions:**
- Content catalog management
- Metadata aggregation
- Search index maintenance
- Content relationship mapping
- Platform availability tracking

**GraphQL Schema:**
```graphql
type Content {
  id: ID!
  title: String!
  type: ContentType!
  description: String
  genres: [String!]!
  releaseYear: Int
  rating: Float
  cast: [Person!]!
  crew: [Person!]!
  availability: [PlatformAvailability!]!
  metadata: ContentMetadata
}

type PlatformAvailability {
  platform: Platform!
  available: Boolean!
  url: String
  price: Price
  lastVerified: DateTime!
}

type Query {
  content(id: ID!): Content
  search(query: SearchInput!): SearchResult!
  trending(limit: Int): [Content!]!
  recommendations(userId: ID!, limit: Int): [Content!]!
}
```

### 6. Recommendation Engine (Serverless)

**Technology:** Python + AWS Lambda + PyTorch

**Algorithms:**
- Collaborative Filtering (user-user, item-item)
- Content-Based Filtering (embeddings similarity)
- Hybrid approach with weighted ensemble
- Cold-start handling with popularity and metadata

**Architecture:**
```python
class RecommendationEngine:
    def __init__(self):
        self.collaborative_model = CollaborativeFilter()
        self.content_model = ContentBasedFilter()
        self.hybrid_weights = {'collaborative': 0.6, 'content': 0.4}

    def recommend(self, user_id: str, n: int = 10) -> List[Content]:
        # Collaborative filtering
        collab_scores = self.collaborative_model.predict(user_id)

        # Content-based filtering
        content_scores = self.content_model.predict(user_id)

        # Hybrid ensemble
        final_scores = self._weighted_ensemble(
            collab_scores,
            content_scores
        )

        # Re-rank with availability
        available_content = self._filter_available(final_scores)

        return available_content[:n]
```

### 7. Availability Verifier (Serverless)

**Technology:** Node.js + AWS Lambda + Step Functions

**Core Functions:**
- Real-time platform API polling
- Availability status verification
- Price checking
- Geographic restriction detection
- Cache invalidation on changes

**Verification Flow:**
```javascript
// Step Functions State Machine
{
  "StartAt": "FetchPlatforms",
  "States": {
    "FetchPlatforms": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:...:getPlatforms",
      "Next": "ParallelVerify"
    },
    "ParallelVerify": {
      "Type": "Parallel",
      "Branches": [
        {"StartAt": "VerifyNetflix", ...},
        {"StartAt": "VerifyHulu", ...},
        {"StartAt": "VerifyDisneyPlus", ...}
      ],
      "Next": "AggregateResults"
    },
    "AggregateResults": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:...:aggregate",
      "Next": "UpdateCache"
    },
    "UpdateCache": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:...:cache",
      "End": true
    }
  }
}
```

---

## Data Flow & Processing Pipelines

### 1. Query Processing Pipeline

```
User Query
    │
    ▼
[API Gateway] ──────────────┐
    │                       │ (Rate Limit Check)
    ▼                       │
[Query Service]             │
    │                       │
    ├──▶ [Cache Check] ─────┤ HIT ──▶ Return Cached Result
    │         │              │
    │         │ MISS         │
    │         ▼              │
    ├──▶ [AI/NLP Engine]    │
    │    - Intent Detection  │
    │    - Entity Extraction │
    │    - Embedding Gen.    │
    │         │              │
    │         ▼              │
    ├──▶ [User Context]     │
    │    - Preferences       │
    │    - History           │
    │    - Location          │
    │         │              │
    │         ▼              │
    ├──▶ [Search Engine]    │
    │    ┌─────┴─────┐      │
    │    ▼           ▼      │
    │ [Vector DB] [SQL DB]  │
    │    │           │      │
    │    └─────┬─────┘      │
    │          ▼            │
    ├──▶ [Result Ranker]    │
    │    - Relevance        │
    │    - Personalization  │
    │    - Availability     │
    │         │              │
    │         ▼              │
    ├──▶ [Availability]     │
    │    [Verification]      │
    │         │              │
    │         ▼              │
    └──▶ [Cache Write]      │
             │              │
             ▼              │
    [Response Formatter] ───┘
             │
             ▼
         [Client]
```

**Pipeline Characteristics:**
- **Latency Budget:** 200ms p95
  - API Gateway: 5ms
  - Cache Lookup: 2ms
  - NLP Processing: 50ms
  - Search Execution: 80ms
  - Availability Check: 40ms
  - Ranking & Formatting: 23ms
- **Throughput:** 10,000 queries/second
- **Cache Hit Rate:** Target 85%

### 2. Content Ingestion Pipeline

```
[Streaming Platform APIs]
    │
    ├──▶ Netflix
    ├──▶ Hulu
    ├──▶ Disney+
    ├──▶ Prime Video
    └──▶ HBO Max
         │
         ▼
[Content Scraper] (Scheduled Jobs)
         │
         ├──▶ Rate Limiter
         │
         ▼
[Data Validation]
    - Schema Check
    - Deduplication
    - Quality Check
         │
         ▼
[Kafka Topic: content-updates]
         │
         ├──────────────┬──────────────┐
         ▼              ▼              ▼
[Content Service]  [Vector DB]   [Search Index]
    (PostgreSQL)    (Embeddings)   (Elasticsearch)
         │              │              │
         └──────────────┴──────────────┘
                       │
                       ▼
            [Cache Invalidation]
                  (Redis)
```

**Pipeline Schedule:**
- Full sync: Daily at 2 AM UTC
- Incremental updates: Every 15 minutes
- Real-time availability checks: On-demand + hourly

### 3. Recommendation Generation Pipeline

```
[User Events]
    │
    ├──▶ Page Views
    ├──▶ Searches
    ├──▶ Clicks
    ├──▶ Watch Events
    └──▶ Ratings
         │
         ▼
[Kafka Topic: user-events]
         │
         ├──────────────┬──────────────┐
         ▼              ▼              ▼
[Real-time]      [Batch ETL]    [Feature Store]
[Aggregation]    (Spark Jobs)   (Redis + S3)
         │              │              │
         └──────────────┴──────────────┘
                       │
                       ▼
        [ML Model Training Pipeline]
            - Collaborative Filter
            - Content-Based Model
            - Hybrid Ensemble
                       │
                       ▼
            [Model Registry + Serving]
            (MLflow + Lambda)
                       │
                       ▼
        [Recommendation Service]
```

**Batch Processing:**
- Spark jobs run hourly for feature extraction
- Model retraining: Daily at 3 AM UTC
- A/B testing: 10% traffic to new models

---

## API Structure & Integration

### 1. Internal API Architecture

**API Style:** RESTful + GraphQL Hybrid

**REST APIs:** For simple CRUD operations
**GraphQL:** For complex queries with flexible data requirements

#### REST API Endpoints

```yaml
# Query API
POST   /api/v1/search
GET    /api/v1/search/suggestions
POST   /api/v1/search/semantic
GET    /api/v1/trending
GET    /api/v1/recommendations

# User API
POST   /api/v1/auth/login
POST   /api/v1/auth/register
POST   /api/v1/auth/refresh
GET    /api/v1/users/:id
PATCH  /api/v1/users/:id
GET    /api/v1/users/:id/history
POST   /api/v1/users/:id/preferences

# Content API
GET    /api/v1/content/:id
GET    /api/v1/content/:id/availability
GET    /api/v1/content/:id/similar
GET    /api/v1/platforms
GET    /api/v1/genres

# Admin API
POST   /api/v1/admin/content/sync
GET    /api/v1/admin/metrics
POST   /api/v1/admin/cache/invalidate
```

#### GraphQL Schema

```graphql
type Query {
  # Search
  search(input: SearchInput!): SearchResult!

  # Content
  content(id: ID!): Content
  contentByIds(ids: [ID!]!): [Content!]!
  trending(limit: Int = 20): [Content!]!

  # Recommendations
  recommendations(userId: ID!, limit: Int = 10): [Content!]!

  # User
  me: User!
  user(id: ID!): User

  # Platforms
  platforms: [Platform!]!
  platformAvailability(contentId: ID!): [PlatformAvailability!]!
}

type Mutation {
  # User
  login(email: String!, password: String!): AuthPayload!
  register(input: RegisterInput!): AuthPayload!
  updateProfile(input: ProfileInput!): User!

  # Interactions
  recordView(contentId: ID!): Boolean!
  rateContent(contentId: ID!, rating: Float!): Boolean!
  addToWatchlist(contentId: ID!): Boolean!
}

type Subscription {
  # Real-time availability updates
  availabilityChanged(contentId: ID!): PlatformAvailability!

  # New content alerts
  newContentForUser: Content!
}
```

### 2. External Streaming Platform Integration

**Integration Pattern:** Adapter Pattern with Circuit Breakers

#### Platform Adapter Interface

```typescript
interface StreamingPlatformAdapter {
  // Authentication
  authenticate(): Promise<AuthToken>;

  // Content Queries
  searchContent(query: string): Promise<Content[]>;
  getContentDetails(id: string): Promise<ContentDetail>;
  checkAvailability(id: string): Promise<Availability>;

  // Metadata
  getGenres(): Promise<Genre[]>;
  getTrendingContent(): Promise<Content[]>;

  // Rate Limiting
  getRateLimits(): RateLimitConfig;
}

class NetflixAdapter implements StreamingPlatformAdapter {
  private client: HttpClient;
  private circuitBreaker: CircuitBreaker;
  private rateLimiter: RateLimiter;

  async authenticate(): Promise<AuthToken> {
    // Netflix-specific OAuth flow
  }

  async checkAvailability(id: string): Promise<Availability> {
    return this.circuitBreaker.execute(async () => {
      await this.rateLimiter.acquire();
      return this.client.get(`/content/${id}/availability`);
    });
  }
}
```

#### Platform Registry

```typescript
class PlatformRegistry {
  private adapters: Map<string, StreamingPlatformAdapter> = new Map();

  register(platform: string, adapter: StreamingPlatformAdapter) {
    this.adapters.set(platform, adapter);
  }

  async checkAvailability(
    contentId: string,
    platforms: string[]
  ): Promise<Map<string, Availability>> {
    // Parallel execution with Promise.all
    const results = await Promise.all(
      platforms.map(async (platform) => {
        const adapter = this.adapters.get(platform);
        const availability = await adapter.checkAvailability(contentId);
        return [platform, availability];
      })
    );

    return new Map(results);
  }
}
```

#### Circuit Breaker Configuration

```typescript
const circuitBreakerConfig = {
  timeout: 3000,           // 3 second timeout
  errorThresholdPercentage: 50,  // Open after 50% failures
  resetTimeout: 30000,     // Try again after 30 seconds
  volumeThreshold: 10,     // Minimum requests before opening

  fallback: async (error) => {
    // Return cached data on circuit open
    return cacheService.get(`availability:${contentId}`);
  }
};
```

### 3. Rate Limiting Strategy

**Per-Platform Rate Limits:**

```yaml
# Rate limit configuration
platforms:
  netflix:
    requests_per_second: 10
    requests_per_hour: 5000
    burst: 20

  hulu:
    requests_per_second: 5
    requests_per_hour: 2000
    burst: 10

  disney_plus:
    requests_per_second: 15
    requests_per_hour: 10000
    burst: 30
```

**Implementation:**

```typescript
class TokenBucketRateLimiter {
  private tokens: number;
  private lastRefill: number;
  private capacity: number;
  private refillRate: number;

  async acquire(): Promise<void> {
    this.refill();

    if (this.tokens < 1) {
      const waitTime = (1 - this.tokens) / this.refillRate * 1000;
      await this.sleep(waitTime);
      this.refill();
    }

    this.tokens -= 1;
  }

  private refill() {
    const now = Date.now();
    const timePassed = (now - this.lastRefill) / 1000;
    const tokensToAdd = timePassed * this.refillRate;

    this.tokens = Math.min(this.capacity, this.tokens + tokensToAdd);
    this.lastRefill = now;
  }
}
```

### 4. API Versioning Strategy

**URL-Based Versioning:** `/api/v1/`, `/api/v2/`

**Version Support Policy:**
- Current version (v1): Full support
- Previous version: 12 months deprecation period
- Older versions: Retired

**Breaking Change Process:**
1. Announce deprecation 6 months in advance
2. Add warnings to deprecated endpoints
3. Provide migration guide
4. Monitor usage of deprecated endpoints
5. Retire after support period

---

## Performance & Caching Strategy

### Multi-Layer Caching Architecture

```
[Client Layer]
    │ Browser Cache (24h)
    │ Service Worker Cache
    ▼
[CDN Layer]
    │ CloudFront (Static Assets: 1 year)
    │ API Responses (Query Cache: 5 min)
    ▼
[API Gateway Cache]
    │ Popular queries (15 min)
    ▼
[Application Cache - Redis]
    │ ┌─────────────────────────────────┐
    │ │ Cache Tiers:                    │
    │ │ - Hot Tier (1h TTL)             │
    │ │ - Warm Tier (6h TTL)            │
    │ │ - Cold Tier (24h TTL)           │
    │ └─────────────────────────────────┘
    ▼
[Database Cache]
    │ PostgreSQL Query Cache
    │ Vector DB Index Cache
    ▼
[Data Layer]
```

### 1. Cache Strategy by Data Type

#### Query Results Cache

```typescript
class QueryCache {
  private redis: RedisClient;

  async get(query: string, context: UserContext): Promise<SearchResult | null> {
    // Generate cache key with query + user context hash
    const key = this.generateKey(query, context);

    // Try cache
    const cached = await this.redis.get(key);
    if (cached) {
      // Async refresh if near expiry (cache-aside pattern)
      if (this.isNearExpiry(key)) {
        this.refreshAsync(query, context);
      }
      return JSON.parse(cached);
    }

    return null;
  }

  async set(
    query: string,
    context: UserContext,
    result: SearchResult
  ): Promise<void> {
    const key = this.generateKey(query, context);
    const ttl = this.calculateTTL(result);

    await this.redis.setex(key, ttl, JSON.stringify(result));
  }

  private calculateTTL(result: SearchResult): number {
    // Hot queries: 1 hour
    if (result.popularity > 1000) return 3600;

    // Warm queries: 6 hours
    if (result.popularity > 100) return 21600;

    // Cold queries: 24 hours
    return 86400;
  }
}
```

#### Availability Cache

```typescript
class AvailabilityCache {
  private redis: RedisClient;

  // Availability data is critical - use write-through cache
  async updateAvailability(
    contentId: string,
    availability: Map<string, Availability>
  ): Promise<void> {
    const key = `availability:${contentId}`;

    // Write to both cache and database
    await Promise.all([
      this.redis.setex(key, 3600, JSON.stringify(availability)),
      this.database.updateAvailability(contentId, availability)
    ]);

    // Publish invalidation event
    await this.publishInvalidation(contentId);
  }
}
```

#### User Session Cache

```typescript
class SessionCache {
  private redis: RedisClient;

  async getSession(token: string): Promise<Session | null> {
    const key = `session:${token}`;
    const data = await this.redis.get(key);

    if (data) {
      // Sliding expiration - extend TTL on access
      await this.redis.expire(key, 3600);
      return JSON.parse(data);
    }

    return null;
  }
}
```

### 2. Cache Invalidation Strategy

**Invalidation Patterns:**

1. **Time-Based (TTL):** Default for most data
2. **Event-Based:** For real-time updates
3. **Manual:** Admin-triggered for corrections

```typescript
class CacheInvalidationService {
  private redis: RedisClient;
  private kafka: KafkaProducer;

  // Event-driven invalidation
  async onContentUpdate(event: ContentUpdateEvent): Promise<void> {
    const contentId = event.contentId;

    // Invalidate related cache keys
    await Promise.all([
      this.redis.del(`content:${contentId}`),
      this.redis.del(`availability:${contentId}`),
      this.redis.del(`similar:${contentId}`),
      // Invalidate all queries that might have returned this content
      this.invalidateQueryCache(contentId)
    ]);

    // Broadcast invalidation to all instances
    await this.kafka.send({
      topic: 'cache-invalidation',
      messages: [{
        key: contentId,
        value: JSON.stringify(event)
      }]
    });
  }

  // Batch invalidation for efficiency
  async invalidateQueryCache(contentId: string): Promise<void> {
    // Use Redis SCAN to find matching keys
    const pattern = `query:*:${contentId}:*`;
    const keys = await this.redis.scanKeys(pattern);

    if (keys.length > 0) {
      await this.redis.del(...keys);
    }
  }
}
```

### 3. Performance Optimization Techniques

#### Database Query Optimization

```sql
-- Indexes for fast lookups
CREATE INDEX idx_content_title ON content USING gin(to_tsvector('english', title));
CREATE INDEX idx_content_genres ON content USING gin(genres);
CREATE INDEX idx_availability_platform ON availability(platform_id, content_id);
CREATE INDEX idx_user_history ON user_history(user_id, watched_at DESC);

-- Materialized views for expensive aggregations
CREATE MATERIALIZED VIEW trending_content AS
SELECT
  c.id,
  c.title,
  COUNT(DISTINCT uh.user_id) as unique_viewers,
  AVG(r.rating) as avg_rating
FROM content c
JOIN user_history uh ON c.id = uh.content_id
LEFT JOIN ratings r ON c.id = r.content_id
WHERE uh.watched_at > NOW() - INTERVAL '7 days'
GROUP BY c.id, c.title
ORDER BY unique_viewers DESC, avg_rating DESC
LIMIT 100;

-- Refresh every hour
CREATE OR REPLACE FUNCTION refresh_trending()
RETURNS void AS $$
BEGIN
  REFRESH MATERIALIZED VIEW CONCURRENTLY trending_content;
END;
$$ LANGUAGE plpgsql;
```

#### Connection Pooling

```typescript
// PostgreSQL connection pool
const pool = new Pool({
  host: process.env.DB_HOST,
  port: 5432,
  database: process.env.DB_NAME,
  user: process.env.DB_USER,
  password: process.env.DB_PASSWORD,

  // Connection pool settings
  max: 20,                    // Maximum connections
  min: 5,                     // Minimum connections
  idleTimeoutMillis: 30000,   // Close idle connections after 30s
  connectionTimeoutMillis: 2000, // Fail fast if can't connect

  // Statement timeout to prevent long-running queries
  statement_timeout: 5000     // 5 second query timeout
});
```

#### Response Compression

```typescript
// Express middleware
app.use(compression({
  level: 6,              // Compression level (1-9)
  threshold: 1024,       // Only compress responses > 1KB
  filter: (req, res) => {
    if (req.headers['x-no-compression']) {
      return false;
    }
    return compression.filter(req, res);
  }
}));
```

#### Lazy Loading & Pagination

```typescript
interface PaginationInput {
  limit: number;    // Max 100
  cursor?: string;  // Opaque cursor for pagination
}

interface PaginatedResult<T> {
  data: T[];
  nextCursor?: string;
  hasMore: boolean;
}

// Cursor-based pagination (better for large datasets)
async function paginateResults(
  query: string,
  input: PaginationInput
): Promise<PaginatedResult<Content>> {
  const limit = Math.min(input.limit, 100);

  // Decode cursor
  const offset = input.cursor ? decodeCursor(input.cursor) : 0;

  // Fetch limit + 1 to check if more results exist
  const results = await database.query(query, { offset, limit: limit + 1 });

  const hasMore = results.length > limit;
  const data = hasMore ? results.slice(0, limit) : results;
  const nextCursor = hasMore ? encodeCursor(offset + limit) : undefined;

  return { data, nextCursor, hasMore };
}
```

### 4. Performance Monitoring

**Key Metrics:**

```typescript
interface PerformanceMetrics {
  // Latency metrics (p50, p95, p99)
  queryLatency: LatencyStats;
  searchLatency: LatencyStats;
  apiLatency: LatencyStats;

  // Throughput
  requestsPerSecond: number;
  queriesPerSecond: number;

  // Cache metrics
  cacheHitRate: number;
  cacheSize: number;

  // Database metrics
  dbConnectionPoolUtilization: number;
  dbQueryDuration: LatencyStats;

  // Error rates
  errorRate: number;
  timeoutRate: number;
}
```

**Monitoring Tools:**
- Prometheus for metrics collection
- Grafana for visualization
- CloudWatch for AWS metrics
- Datadog for APM and tracing

---

## Scalability & Fault Tolerance

### 1. Horizontal Scaling Strategy

#### Auto-Scaling Configuration

```yaml
# Kubernetes HPA (Horizontal Pod Autoscaler)
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: query-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: query-service
  minReplicas: 5
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

#### Load Balancing

```yaml
# AWS Application Load Balancer Configuration
LoadBalancer:
  Type: application
  Scheme: internet-facing

  Listeners:
    - Protocol: HTTPS
      Port: 443
      DefaultActions:
        - Type: forward
          TargetGroupArn: !Ref QueryServiceTargetGroup

  HealthCheck:
    Protocol: HTTP
    Path: /health
    Interval: 30
    Timeout: 5
    HealthyThreshold: 2
    UnhealthyThreshold: 3

  LoadBalancingAlgorithm: least_outstanding_requests

  StickySessions:
    Enabled: false  # Stateless design - no sticky sessions needed
```

### 2. Database Scaling

#### PostgreSQL Scaling Strategy

**Read Replicas:**

```
┌─────────────────┐
│   Primary DB    │ (Writes)
│   (Master)      │
└────────┬────────┘
         │
         │ Replication
         │
    ┌────┴──────────────────┐
    │                       │
    ▼                       ▼
┌──────────┐          ┌──────────┐
│ Replica 1│ (Reads)  │ Replica 2│ (Reads)
└──────────┘          └──────────┘
```

**Connection Routing:**

```typescript
class DatabaseRouter {
  private primary: Pool;
  private replicas: Pool[];
  private replicaIndex: number = 0;

  async write(query: string, params: any[]): Promise<any> {
    // All writes go to primary
    return this.primary.query(query, params);
  }

  async read(query: string, params: any[]): Promise<any> {
    // Round-robin across replicas
    const replica = this.getNextReplica();
    return replica.query(query, params);
  }

  private getNextReplica(): Pool {
    const replica = this.replicas[this.replicaIndex];
    this.replicaIndex = (this.replicaIndex + 1) % this.replicas.length;
    return replica;
  }
}
```

**Partitioning Strategy:**

```sql
-- Partition large tables by date
CREATE TABLE user_history (
    id BIGSERIAL,
    user_id UUID NOT NULL,
    content_id UUID NOT NULL,
    watched_at TIMESTAMP NOT NULL,
    duration_seconds INT
) PARTITION BY RANGE (watched_at);

-- Create monthly partitions
CREATE TABLE user_history_2025_01 PARTITION OF user_history
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

CREATE TABLE user_history_2025_02 PARTITION OF user_history
    FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');

-- Auto-create partitions with pg_partman
SELECT create_parent('public.user_history', 'watched_at', 'native', 'monthly');
```

#### Vector Database Scaling

**Pinecone Scaling:**

```typescript
const pineconeConfig = {
  environment: 'production',

  // Index configuration
  index: {
    name: 'content-embeddings',
    dimension: 1536,  // text-embedding-3-large
    metric: 'cosine',

    // Pod-based scaling
    pods: 4,          // Start with 4 pods
    replicas: 2,      // 2 replicas for HA
    podType: 'p1.x2', // Performance-optimized

    // Metadata filtering
    metadataConfig: {
      indexed: ['genre', 'year', 'platform', 'rating']
    }
  },

  // Query configuration
  query: {
    topK: 100,
    includeMetadata: true,
    includeValues: false
  }
};

// Auto-scaling logic
class VectorDBScaler {
  async scaleIfNeeded(metrics: IndexMetrics): Promise<void> {
    // Scale up if query latency > 100ms p95
    if (metrics.p95Latency > 100) {
      await this.scaleUp();
    }

    // Scale down if utilization < 30% for 1 hour
    if (metrics.utilization < 0.3 && metrics.duration > 3600) {
      await this.scaleDown();
    }
  }
}
```

### 3. Fault Tolerance Mechanisms

#### Circuit Breaker Pattern

```typescript
class CircuitBreaker {
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';
  private failureCount: number = 0;
  private lastFailureTime: number = 0;
  private successCount: number = 0;

  constructor(private config: CircuitBreakerConfig) {}

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime > this.config.resetTimeout) {
        this.state = 'HALF_OPEN';
      } else {
        throw new CircuitBreakerOpenError('Circuit breaker is OPEN');
      }
    }

    try {
      const result = await Promise.race([
        operation(),
        this.timeout()
      ]);

      this.onSuccess();
      return result;

    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess(): void {
    if (this.state === 'HALF_OPEN') {
      this.successCount++;
      if (this.successCount >= this.config.halfOpenSuccessThreshold) {
        this.state = 'CLOSED';
        this.failureCount = 0;
        this.successCount = 0;
      }
    }
  }

  private onFailure(): void {
    this.failureCount++;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= this.config.failureThreshold) {
      this.state = 'OPEN';
    }
  }

  private async timeout(): Promise<never> {
    await new Promise(resolve => setTimeout(resolve, this.config.timeout));
    throw new TimeoutError('Operation timed out');
  }
}
```

#### Retry Strategy with Exponential Backoff

```typescript
class RetryStrategy {
  async executeWithRetry<T>(
    operation: () => Promise<T>,
    options: RetryOptions = {}
  ): Promise<T> {
    const {
      maxRetries = 3,
      initialDelay = 1000,
      maxDelay = 10000,
      backoffFactor = 2,
      retryableErrors = [TimeoutError, NetworkError]
    } = options;

    let lastError: Error;
    let delay = initialDelay;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error;

        // Don't retry if not a retryable error
        if (!this.isRetryable(error, retryableErrors)) {
          throw error;
        }

        // Don't retry if last attempt
        if (attempt === maxRetries) {
          break;
        }

        // Wait before retry with exponential backoff + jitter
        const jitter = Math.random() * 0.3 * delay;
        await this.sleep(delay + jitter);

        delay = Math.min(delay * backoffFactor, maxDelay);
      }
    }

    throw new MaxRetriesExceededError(
      `Failed after ${maxRetries} retries`,
      lastError
    );
  }

  private isRetryable(error: Error, retryableErrors: any[]): boolean {
    return retryableErrors.some(ErrorType => error instanceof ErrorType);
  }
}
```

#### Bulkhead Pattern

```typescript
class BulkheadIsolation {
  private semaphores: Map<string, Semaphore> = new Map();

  constructor(private config: Map<string, BulkheadConfig>) {
    // Create semaphores for each resource
    config.forEach((bulkheadConfig, resource) => {
      this.semaphores.set(
        resource,
        new Semaphore(bulkheadConfig.maxConcurrent)
      );
    });
  }

  async execute<T>(
    resource: string,
    operation: () => Promise<T>
  ): Promise<T> {
    const semaphore = this.semaphores.get(resource);

    if (!semaphore) {
      throw new Error(`No bulkhead configured for ${resource}`);
    }

    // Acquire permit (blocks if at capacity)
    await semaphore.acquire();

    try {
      return await operation();
    } finally {
      semaphore.release();
    }
  }
}

// Configuration
const bulkheadConfig = new Map([
  ['netflix-api', { maxConcurrent: 10 }],
  ['hulu-api', { maxConcurrent: 5 }],
  ['database', { maxConcurrent: 50 }],
  ['ai-service', { maxConcurrent: 20 }]
]);
```

#### Health Checks

```typescript
// Health check endpoint
app.get('/health', async (req, res) => {
  const health = await performHealthChecks();

  const status = health.every(check => check.healthy) ? 200 : 503;

  res.status(status).json({
    status: status === 200 ? 'healthy' : 'unhealthy',
    timestamp: new Date().toISOString(),
    checks: health
  });
});

async function performHealthChecks(): Promise<HealthCheck[]> {
  return Promise.all([
    checkDatabase(),
    checkRedis(),
    checkVectorDB(),
    checkExternalAPIs(),
    checkDiskSpace(),
    checkMemory()
  ]);
}

async function checkDatabase(): Promise<HealthCheck> {
  try {
    await pool.query('SELECT 1');
    return { name: 'database', healthy: true };
  } catch (error) {
    return {
      name: 'database',
      healthy: false,
      error: error.message
    };
  }
}
```

### 4. Disaster Recovery

**Backup Strategy:**

```yaml
Backup Schedule:
  Database:
    Full Backup: Daily at 2 AM UTC
    Incremental: Every 6 hours
    Point-in-Time Recovery: Enabled (7 days retention)
    Cross-Region Replication: Enabled

  Redis:
    RDB Snapshots: Every 6 hours
    AOF: Enabled with fsync every second

  Vector DB:
    Snapshots: Daily
    Metadata Backup: Continuous

Recovery Time Objective (RTO): 1 hour
Recovery Point Objective (RPO): 15 minutes
```

**Disaster Recovery Plan:**

```typescript
class DisasterRecovery {
  async failoverToSecondaryRegion(): Promise<void> {
    // 1. Update DNS to point to secondary region
    await this.updateDNS({
      primary: 'us-west-2',
      secondary: 'us-east-1'
    });

    // 2. Promote read replica to primary
    await this.promoteReplica('us-east-1');

    // 3. Update application configuration
    await this.updateConfig({
      region: 'us-east-1',
      databaseEndpoint: 'db-east.example.com'
    });

    // 4. Verify health of secondary region
    await this.verifyHealth('us-east-1');

    // 5. Notify ops team
    await this.notifyOps({
      event: 'FAILOVER_COMPLETE',
      timestamp: new Date(),
      region: 'us-east-1'
    });
  }
}
```

---

## Security & Privacy

### 1. Authentication & Authorization

#### JWT-Based Authentication

```typescript
interface JWTPayload {
  sub: string;        // User ID
  email: string;
  roles: string[];
  iat: number;        // Issued at
  exp: number;        // Expiration
  jti: string;        // JWT ID (for revocation)
}

class AuthenticationService {
  private readonly JWT_SECRET = process.env.JWT_SECRET;
  private readonly JWT_EXPIRY = '15m';      // Access token: 15 minutes
  private readonly REFRESH_EXPIRY = '7d';   // Refresh token: 7 days

  async login(email: string, password: string): Promise<AuthTokens> {
    // 1. Validate credentials
    const user = await this.validateCredentials(email, password);

    if (!user) {
      throw new UnauthorizedError('Invalid credentials');
    }

    // 2. Generate tokens
    const accessToken = await this.generateAccessToken(user);
    const refreshToken = await this.generateRefreshToken(user);

    // 3. Store refresh token (for revocation)
    await this.storeRefreshToken(user.id, refreshToken);

    return { accessToken, refreshToken };
  }

  private async generateAccessToken(user: User): Promise<string> {
    const payload: JWTPayload = {
      sub: user.id,
      email: user.email,
      roles: user.roles,
      iat: Math.floor(Date.now() / 1000),
      exp: Math.floor(Date.now() / 1000) + 900, // 15 minutes
      jti: uuidv4()
    };

    return jwt.sign(payload, this.JWT_SECRET, {
      algorithm: 'RS256'  // Asymmetric signing
    });
  }

  async validateToken(token: string): Promise<JWTPayload> {
    try {
      const payload = jwt.verify(token, this.JWT_PUBLIC_KEY) as JWTPayload;

      // Check if token is revoked
      const isRevoked = await this.isTokenRevoked(payload.jti);
      if (isRevoked) {
        throw new UnauthorizedError('Token has been revoked');
      }

      return payload;

    } catch (error) {
      throw new UnauthorizedError('Invalid token');
    }
  }
}
```

#### Role-Based Access Control (RBAC)

```typescript
enum Role {
  ANONYMOUS = 'anonymous',
  USER = 'user',
  PREMIUM = 'premium',
  ADMIN = 'admin',
  SUPER_ADMIN = 'super_admin'
}

enum Permission {
  READ_CONTENT = 'content:read',
  SEARCH = 'search:execute',
  SAVE_PREFERENCES = 'preferences:write',
  VIEW_HISTORY = 'history:read',
  ADMIN_CONTENT = 'admin:content',
  ADMIN_USERS = 'admin:users'
}

const rolePermissions: Record<Role, Permission[]> = {
  [Role.ANONYMOUS]: [
    Permission.READ_CONTENT,
    Permission.SEARCH
  ],
  [Role.USER]: [
    Permission.READ_CONTENT,
    Permission.SEARCH,
    Permission.SAVE_PREFERENCES,
    Permission.VIEW_HISTORY
  ],
  [Role.PREMIUM]: [
    // Inherits USER permissions
    ...rolePermissions[Role.USER]
  ],
  [Role.ADMIN]: [
    ...rolePermissions[Role.USER],
    Permission.ADMIN_CONTENT
  ],
  [Role.SUPER_ADMIN]: [
    ...rolePermissions[Role.ADMIN],
    Permission.ADMIN_USERS
  ]
};

// Authorization middleware
function requirePermission(permission: Permission) {
  return async (req: Request, res: Response, next: NextFunction) => {
    const user = req.user;

    if (!user) {
      return res.status(401).json({ error: 'Unauthorized' });
    }

    const hasPermission = user.roles.some(role =>
      rolePermissions[role]?.includes(permission)
    );

    if (!hasPermission) {
      return res.status(403).json({ error: 'Forbidden' });
    }

    next();
  };
}
```

### 2. Data Encryption

#### Encryption at Rest

```typescript
class EncryptionService {
  private readonly algorithm = 'aes-256-gcm';
  private readonly keyId = process.env.KMS_KEY_ID;

  // Encrypt sensitive data before storing
  async encrypt(plaintext: string): Promise<EncryptedData> {
    // Get data encryption key from KMS
    const dataKey = await this.kms.generateDataKey({
      KeyId: this.keyId,
      KeySpec: 'AES_256'
    });

    // Generate IV
    const iv = crypto.randomBytes(16);

    // Encrypt data
    const cipher = crypto.createCipheriv(
      this.algorithm,
      dataKey.Plaintext,
      iv
    );

    let encrypted = cipher.update(plaintext, 'utf8', 'base64');
    encrypted += cipher.final('base64');

    const authTag = cipher.getAuthTag();

    return {
      ciphertext: encrypted,
      iv: iv.toString('base64'),
      authTag: authTag.toString('base64'),
      encryptedDataKey: dataKey.CiphertextBlob.toString('base64')
    };
  }

  async decrypt(encryptedData: EncryptedData): Promise<string> {
    // Decrypt data key using KMS
    const dataKey = await this.kms.decrypt({
      CiphertextBlob: Buffer.from(encryptedData.encryptedDataKey, 'base64')
    });

    // Decrypt data
    const decipher = crypto.createDecipheriv(
      this.algorithm,
      dataKey.Plaintext,
      Buffer.from(encryptedData.iv, 'base64')
    );

    decipher.setAuthTag(Buffer.from(encryptedData.authTag, 'base64'));

    let decrypted = decipher.update(encryptedData.ciphertext, 'base64', 'utf8');
    decrypted += decipher.final('utf8');

    return decrypted;
  }
}
```

#### Encryption in Transit

```yaml
# TLS Configuration
TLS:
  MinVersion: TLS1.3
  CipherSuites:
    - TLS_AES_256_GCM_SHA384
    - TLS_AES_128_GCM_SHA256
    - TLS_CHACHA20_POLY1305_SHA256

  # Certificate Management
  Certificates:
    Provider: AWS Certificate Manager
    AutoRenewal: Enabled
    Domains:
      - api.mediadiscovery.com
      - *.mediadiscovery.com

  # HSTS Configuration
  HSTS:
    MaxAge: 31536000  # 1 year
    IncludeSubDomains: true
    Preload: true
```

### 3. Input Validation & Sanitization

```typescript
class InputValidator {
  // Query sanitization
  static sanitizeQuery(query: string): string {
    // Remove potential injection attacks
    const cleaned = query
      .trim()
      .replace(/[<>'"]/g, '')  // Remove HTML/SQL special chars
      .substring(0, 500);       // Limit length

    // Block malicious patterns
    const blockedPatterns = [
      /(\bOR\b|\bAND\b).*?=.*?/i,  // SQL injection
      /<script/i,                   // XSS
      /javascript:/i,               // XSS
      /on\w+\s*=/i                  // Event handlers
    ];

    for (const pattern of blockedPatterns) {
      if (pattern.test(cleaned)) {
        throw new ValidationError('Invalid input detected');
      }
    }

    return cleaned;
  }

  // Email validation
  static validateEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email) && email.length <= 320;
  }

  // Pagination validation
  static validatePagination(input: PaginationInput): void {
    if (input.limit < 1 || input.limit > 100) {
      throw new ValidationError('Limit must be between 1 and 100');
    }

    if (input.cursor && !this.isValidCursor(input.cursor)) {
      throw new ValidationError('Invalid cursor');
    }
  }
}
```

### 4. Rate Limiting & DDoS Protection

```typescript
class RateLimitingService {
  private redis: RedisClient;

  // Tiered rate limiting based on user type
  private limits = {
    [Role.ANONYMOUS]: {
      requests: 10,
      window: 60  // 10 requests per minute
    },
    [Role.USER]: {
      requests: 100,
      window: 60  // 100 requests per minute
    },
    [Role.PREMIUM]: {
      requests: 1000,
      window: 60  // 1000 requests per minute
    }
  };

  async checkLimit(
    userId: string,
    role: Role
  ): Promise<RateLimitResult> {
    const limit = this.limits[role];
    const key = `ratelimit:${userId}:${Math.floor(Date.now() / (limit.window * 1000))}`;

    // Increment counter
    const current = await this.redis.incr(key);

    // Set expiry on first request
    if (current === 1) {
      await this.redis.expire(key, limit.window);
    }

    const remaining = Math.max(0, limit.requests - current);
    const isAllowed = current <= limit.requests;

    return {
      isAllowed,
      limit: limit.requests,
      remaining,
      resetAt: new Date(
        Math.ceil(Date.now() / (limit.window * 1000)) * limit.window * 1000
      )
    };
  }
}
```

### 5. Privacy & Data Protection

#### GDPR Compliance

```typescript
class PrivacyService {
  // Right to Access
  async exportUserData(userId: string): Promise<UserDataExport> {
    return {
      profile: await this.getUserProfile(userId),
      preferences: await this.getUserPreferences(userId),
      history: await this.getViewingHistory(userId),
      searches: await this.getSearchHistory(userId)
    };
  }

  // Right to Erasure
  async deleteUserData(userId: string): Promise<void> {
    // Anonymize instead of delete for analytics
    await Promise.all([
      this.anonymizeUserProfile(userId),
      this.deleteUserPreferences(userId),
      this.deleteViewingHistory(userId),
      this.deleteSearchHistory(userId),
      this.revokeAllTokens(userId)
    ]);

    // Audit log
    await this.auditLog({
      action: 'USER_DATA_DELETED',
      userId,
      timestamp: new Date()
    });
  }

  // Right to Portability
  async createDataPortabilityPackage(userId: string): Promise<Buffer> {
    const userData = await this.exportUserData(userId);
    return this.createZipArchive(userData);
  }
}
```

#### PII Handling

```typescript
// Data classification
enum DataClassification {
  PUBLIC = 'public',
  INTERNAL = 'internal',
  CONFIDENTIAL = 'confidential',
  RESTRICTED = 'restricted'
}

interface UserProfile {
  id: string;                          // PUBLIC
  email: string;                       // CONFIDENTIAL
  name: string;                        // CONFIDENTIAL
  viewingHistory: ViewingRecord[];     // RESTRICTED
  paymentInfo: PaymentMethod[];        // RESTRICTED
}

// PII detection and masking
class PIIProtection {
  static maskEmail(email: string): string {
    const [local, domain] = email.split('@');
    const maskedLocal = local[0] + '*'.repeat(local.length - 2) + local[local.length - 1];
    return `${maskedLocal}@${domain}`;
  }

  static maskCreditCard(cardNumber: string): string {
    return '*'.repeat(12) + cardNumber.slice(-4);
  }

  // Log sanitization - remove PII before logging
  static sanitizeLog(data: any): any {
    const sanitized = { ...data };

    // Remove sensitive fields
    const sensitiveFields = ['password', 'ssn', 'creditCard', 'token'];
    for (const field of sensitiveFields) {
      if (field in sanitized) {
        sanitized[field] = '[REDACTED]';
      }
    }

    // Mask email
    if (sanitized.email) {
      sanitized.email = this.maskEmail(sanitized.email);
    }

    return sanitized;
  }
}
```

---

## Database Architecture

### 1. Database Selection & Rationale

#### Primary Databases

```
┌──────────────────────────────────────────────────────────────┐
│                    Database Layer                             │
│                                                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   PostgreSQL    │  │  Pinecone VDB   │  │    Redis     │ │
│  │                 │  │                 │  │              │ │
│  │ - User Data     │  │ - Embeddings    │  │ - Cache      │ │
│  │ - Content Cat.  │  │ - Semantic      │  │ - Sessions   │ │
│  │ - Relations     │  │   Search        │  │ - Queues     │ │
│  │ - Transactions  │  │ - ML Features   │  │              │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│                                                               │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │  Elasticsearch  │  │      S3         │                   │
│  │                 │  │                 │                   │
│  │ - Full-text     │  │ - Backups       │                   │
│  │   Search        │  │ - Media Files   │                   │
│  │ - Aggregations  │  │ - Logs          │                   │
│  └─────────────────┘  └─────────────────┘                   │
└──────────────────────────────────────────────────────────────┘
```

#### Database Decision Matrix

| Database | Use Case | Rationale |
|----------|----------|-----------|
| **PostgreSQL** | User data, content catalog, relations | ACID compliance, rich querying, JSON support, mature ecosystem |
| **Pinecone** | Vector embeddings, semantic search | Purpose-built for vector search, 150x faster than alternatives, managed service |
| **Redis** | Caching, sessions, rate limiting | In-memory speed, data structures, pub/sub |
| **Elasticsearch** | Full-text search, analytics | Powerful search, aggregations, real-time |
| **S3** | Object storage, backups | Durability, scalability, cost-effective |

### 2. PostgreSQL Schema Design

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(320) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    email_verified BOOLEAN DEFAULT FALSE,
    status VARCHAR(20) DEFAULT 'active',

    -- Audit fields
    last_login_at TIMESTAMP,
    last_login_ip INET
);

-- User preferences
CREATE TABLE user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    favorite_genres TEXT[] DEFAULT '{}',
    preferred_languages TEXT[] DEFAULT '{}',
    content_rating_limit VARCHAR(10),
    preferred_platforms TEXT[] DEFAULT '{}',
    notification_settings JSONB DEFAULT '{}',
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Content catalog
CREATE TABLE content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255) UNIQUE NOT NULL,
    title VARCHAR(500) NOT NULL,
    original_title VARCHAR(500),
    type VARCHAR(20) NOT NULL, -- movie, series, documentary
    description TEXT,
    release_year INT,
    runtime_minutes INT,
    content_rating VARCHAR(10),
    genres TEXT[] NOT NULL,
    languages TEXT[],
    countries TEXT[],
    imdb_id VARCHAR(20),
    tmdb_id INT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_type CHECK (type IN ('movie', 'series', 'documentary', 'short'))
);

-- Create indexes
CREATE INDEX idx_content_title_trgm ON content USING gin(title gin_trgm_ops);
CREATE INDEX idx_content_genres ON content USING gin(genres);
CREATE INDEX idx_content_type ON content(type);
CREATE INDEX idx_content_release_year ON content(release_year);

-- People (cast & crew)
CREATE TABLE people (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    imdb_id VARCHAR(20) UNIQUE,
    tmdb_id INT,
    profile_image_url TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Content credits (cast & crew relationships)
CREATE TABLE content_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- actor, director, writer, producer
    character_name VARCHAR(255), -- For actors
    credit_order INT, -- For display ordering

    UNIQUE(content_id, person_id, role, character_name)
);

CREATE INDEX idx_credits_content ON content_credits(content_id);
CREATE INDEX idx_credits_person ON content_credits(person_id);

-- Streaming platforms
CREATE TABLE platforms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    logo_url TEXT,
    website_url TEXT,
    api_endpoint TEXT,
    api_rate_limit INT DEFAULT 100,
    api_rate_window INT DEFAULT 60, -- seconds
    active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Platform availability
CREATE TABLE platform_availability (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    platform_id UUID NOT NULL REFERENCES platforms(id) ON DELETE CASCADE,
    available BOOLEAN NOT NULL DEFAULT TRUE,
    url TEXT,
    price_tier VARCHAR(20), -- free, subscription, rent, buy
    price_amount DECIMAL(10,2),
    price_currency VARCHAR(3) DEFAULT 'USD',
    region VARCHAR(2) DEFAULT 'US',
    last_verified_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(content_id, platform_id, region)
);

CREATE INDEX idx_availability_content ON platform_availability(content_id);
CREATE INDEX idx_availability_platform ON platform_availability(platform_id);
CREATE INDEX idx_availability_region ON platform_availability(region);

-- User viewing history
CREATE TABLE user_history (
    id BIGSERIAL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    watched_at TIMESTAMP NOT NULL DEFAULT NOW(),
    duration_seconds INT,
    completed BOOLEAN DEFAULT FALSE,
    platform_id UUID REFERENCES platforms(id),

    PRIMARY KEY (id, watched_at)
) PARTITION BY RANGE (watched_at);

-- Create monthly partitions (managed by pg_partman)
CREATE TABLE user_history_2025_01 PARTITION OF user_history
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

CREATE INDEX idx_history_user ON user_history(user_id, watched_at DESC);
CREATE INDEX idx_history_content ON user_history(content_id);

-- User watchlist
CREATE TABLE watchlist (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    added_at TIMESTAMP NOT NULL DEFAULT NOW(),
    priority INT DEFAULT 0,

    PRIMARY KEY (user_id, content_id)
);

CREATE INDEX idx_watchlist_user ON watchlist(user_id, added_at DESC);

-- Ratings
CREATE TABLE ratings (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    rating DECIMAL(3,1) NOT NULL CHECK (rating >= 0 AND rating <= 10),
    review TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    PRIMARY KEY (user_id, content_id)
);

CREATE INDEX idx_ratings_content ON ratings(content_id);

-- Materialized view for trending content
CREATE MATERIALIZED VIEW trending_content AS
SELECT
    c.id,
    c.title,
    c.type,
    c.genres,
    COUNT(DISTINCT uh.user_id) as unique_viewers,
    AVG(r.rating) as avg_rating,
    COUNT(DISTINCT w.user_id) as watchlist_count
FROM content c
LEFT JOIN user_history uh ON c.id = uh.content_id
    AND uh.watched_at > NOW() - INTERVAL '7 days'
LEFT JOIN ratings r ON c.id = r.content_id
LEFT JOIN watchlist w ON c.id = w.content_id
GROUP BY c.id, c.title, c.type, c.genres
ORDER BY unique_viewers DESC, avg_rating DESC
LIMIT 1000;

CREATE UNIQUE INDEX idx_trending_id ON trending_content(id);

-- Refresh function
CREATE OR REPLACE FUNCTION refresh_trending_content()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY trending_content;
END;
$$ LANGUAGE plpgsql;

-- Search history (for analytics and ML)
CREATE TABLE search_history (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    session_id VARCHAR(255),
    query TEXT NOT NULL,
    intent VARCHAR(50), -- search, browse, recommend
    results_count INT,
    clicked_content_ids UUID[],
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_history_user ON search_history(user_id, created_at DESC);
CREATE INDEX idx_search_history_query ON search_history USING gin(to_tsvector('english', query));
```

### 3. Vector Database Design (Pinecone)

```typescript
// Vector database schema
interface ContentEmbedding {
  id: string;                    // Content UUID
  values: number[];              // Embedding vector (1536 dims)
  metadata: {
    title: string;
    type: string;                // movie, series, etc.
    genres: string[];
    year: number;
    rating: number;
    platforms: string[];
    description_snippet: string;
  };
}

// Pinecone index configuration
const pineconeIndex = {
  name: 'content-embeddings-prod',
  dimension: 1536,               // text-embedding-3-large
  metric: 'cosine',              // cosine similarity
  pods: 4,
  replicas: 2,
  podType: 'p1.x2',

  metadataConfig: {
    indexed: [
      'type',
      'genres',
      'year',
      'rating',
      'platforms'
    ]
  }
};

// Embedding generation
class EmbeddingService {
  private openai: OpenAI;
  private pinecone: PineconeClient;

  async embedContent(content: Content): Promise<void> {
    // Create text representation
    const text = this.createEmbeddingText(content);

    // Generate embedding
    const response = await this.openai.embeddings.create({
      model: 'text-embedding-3-large',
      input: text
    });

    const embedding = response.data[0].embedding;

    // Store in Pinecone
    await this.pinecone.upsert({
      vectors: [{
        id: content.id,
        values: embedding,
        metadata: {
          title: content.title,
          type: content.type,
          genres: content.genres,
          year: content.releaseYear,
          rating: content.avgRating,
          platforms: content.availablePlatforms,
          description_snippet: content.description.substring(0, 500)
        }
      }]
    });
  }

  private createEmbeddingText(content: Content): string {
    // Combine relevant fields for rich embeddings
    return [
      content.title,
      content.originalTitle,
      content.description,
      `Type: ${content.type}`,
      `Genres: ${content.genres.join(', ')}`,
      `Cast: ${content.cast.slice(0, 5).join(', ')}`,
      `Director: ${content.director}`,
      `Year: ${content.releaseYear}`
    ].filter(Boolean).join('\n');
  }

  async semanticSearch(
    query: string,
    filters?: MetadataFilter
  ): Promise<Content[]> {
    // Generate query embedding
    const response = await this.openai.embeddings.create({
      model: 'text-embedding-3-large',
      input: query
    });

    const queryEmbedding = response.data[0].embedding;

    // Search Pinecone
    const results = await this.pinecone.query({
      vector: queryEmbedding,
      topK: 100,
      includeMetadata: true,
      filter: filters
    });

    // Fetch full content details from PostgreSQL
    const contentIds = results.matches.map(m => m.id);
    return this.contentService.getByIds(contentIds);
  }
}
```

### 4. Redis Data Structures

```typescript
// Redis key patterns and TTLs
const RedisKeys = {
  // Query cache
  QUERY_RESULT: (queryHash: string) => `query:${queryHash}`,
  QUERY_TTL: 3600,  // 1 hour

  // Availability cache
  AVAILABILITY: (contentId: string) => `availability:${contentId}`,
  AVAILABILITY_TTL: 3600,  // 1 hour

  // User session
  SESSION: (sessionId: string) => `session:${sessionId}`,
  SESSION_TTL: 86400,  // 24 hours

  // Rate limiting
  RATE_LIMIT: (userId: string, window: number) =>
    `ratelimit:${userId}:${window}`,

  // Trending content
  TRENDING: () => 'trending:content',
  TRENDING_TTL: 900,  // 15 minutes

  // Real-time counters
  SEARCH_COUNTER: (contentId: string) => `counter:search:${contentId}`,
  VIEW_COUNTER: (contentId: string) => `counter:view:${contentId}`,

  // Leaderboards
  POPULAR_SEARCHES: () => 'leaderboard:searches',
  POPULAR_CONTENT: (timeframe: string) => `leaderboard:content:${timeframe}`
};

// Redis usage examples
class CacheService {
  private redis: RedisClient;

  // Query result caching
  async cacheQueryResult(
    query: string,
    context: UserContext,
    result: SearchResult
  ): Promise<void> {
    const key = RedisKeys.QUERY_RESULT(this.hashQuery(query, context));
    await this.redis.setex(
      key,
      RedisKeys.QUERY_TTL,
      JSON.stringify(result)
    );
  }

  // Leaderboard (sorted sets)
  async trackPopularContent(contentId: string, score: number): Promise<void> {
    await this.redis.zadd(
      RedisKeys.POPULAR_CONTENT('week'),
      score,
      contentId
    );
  }

  async getTopContent(limit: number): Promise<string[]> {
    return this.redis.zrevrange(
      RedisKeys.POPULAR_CONTENT('week'),
      0,
      limit - 1
    );
  }

  // Pub/Sub for cache invalidation
  async publishInvalidation(contentId: string): Promise<void> {
    await this.redis.publish(
      'cache:invalidate',
      JSON.stringify({ contentId, timestamp: Date.now() })
    );
  }
}
```

---

## Real-Time Availability System

### Architecture

```
┌──────────────────────────────────────────────────────────────┐
│              Real-Time Availability System                    │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │           Availability Verification Engine               │ │
│  │                                                           │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │  Scheduled  │  │  On-Demand  │  │  Real-Time  │     │ │
│  │  │  Polling    │  │  Verification│  │  Webhooks   │     │ │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │ │
│  │         │                 │                 │            │ │
│  │         └─────────────────┼─────────────────┘            │ │
│  │                           ▼                              │ │
│  │              ┌────────────────────────┐                  │ │
│  │              │   Platform Adapters    │                  │ │
│  │              │   (Circuit Breakers)   │                  │ │
│  │              └────────────┬───────────┘                  │ │
│  │                           │                              │ │
│  │         ┌─────────────────┼─────────────────┐            │ │
│  │         ▼                 ▼                 ▼            │ │
│  │  ┌──────────┐      ┌──────────┐      ┌──────────┐      │ │
│  │  │ Netflix  │      │   Hulu   │      │ Disney+  │      │ │
│  │  │   API    │      │   API    │      │   API    │      │ │
│  │  └──────────┘      └──────────┘      └──────────┘      │ │
│  │                                                           │ │
│  │                           │                              │ │
│  │                           ▼                              │ │
│  │              ┌────────────────────────┐                  │ │
│  │              │  Result Aggregator     │                  │ │
│  │              └────────────┬───────────┘                  │ │
│  │                           │                              │ │
│  │         ┌─────────────────┼─────────────────┐            │ │
│  │         ▼                 ▼                 ▼            │ │
│  │  ┌──────────┐      ┌──────────┐      ┌──────────┐      │ │
│  │  │PostgreSQL│      │  Redis   │      │  Kafka   │      │ │
│  │  │(Source)  │      │ (Cache)  │      │ (Events) │      │ │
│  │  └──────────┘      └──────────┘      └──────────┘      │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 1. Scheduled Polling System

```typescript
class AvailabilityPoller {
  private platforms: Map<string, StreamingPlatformAdapter>;
  private scheduler: Scheduler;

  constructor() {
    this.setupSchedules();
  }

  private setupSchedules(): void {
    // High-priority content: Check every 15 minutes
    this.scheduler.schedule('*/15 * * * *', async () => {
      await this.pollHighPriority();
    });

    // Medium-priority content: Check every hour
    this.scheduler.schedule('0 * * * *', async () => {
      await this.pollMediumPriority();
    });

    // Low-priority content: Check every 6 hours
    this.scheduler.schedule('0 */6 * * *', async () => {
      await this.pollLowPriority();
    });

    // Full sync: Daily at 2 AM
    this.scheduler.schedule('0 2 * * *', async () => {
      await this.fullSync();
    });
  }

  private async pollHighPriority(): Promise<void> {
    // Get trending and popular content
    const content = await this.getHighPriorityContent();

    // Poll all platforms in parallel (with rate limiting)
    await this.pollContent(content);
  }

  private async pollContent(contentList: Content[]): Promise<void> {
    // Batch into chunks to respect rate limits
    const batches = chunk(contentList, 100);

    for (const batch of batches) {
      // Parallel execution across platforms
      const results = await Promise.allSettled(
        batch.map(content => this.checkAllPlatforms(content))
      );

      // Process results
      await this.processResults(results);
    }
  }

  private async checkAllPlatforms(
    content: Content
  ): Promise<Map<string, Availability>> {
    const platformChecks = Array.from(this.platforms.entries()).map(
      async ([platformName, adapter]) => {
        try {
          const availability = await adapter.checkAvailability(content.externalId);
          return [platformName, availability];
        } catch (error) {
          logger.error(`Failed to check ${platformName} for ${content.id}`, error);
          return [platformName, null];
        }
      }
    );

    const results = await Promise.all(platformChecks);
    return new Map(results.filter(([_, avail]) => avail !== null));
  }

  private async processResults(
    results: PromiseSettledResult<Map<string, Availability>>[]
  ): Promise<void> {
    const updates: AvailabilityUpdate[] = [];

    for (const result of results) {
      if (result.status === 'fulfilled') {
        const availability = result.value;
        updates.push(...this.createUpdates(availability));
      }
    }

    // Batch update database
    await this.batchUpdate(updates);

    // Invalidate cache
    await this.invalidateCache(updates);

    // Publish events
    await this.publishEvents(updates);
  }
}
```

### 2. On-Demand Verification

```typescript
class OnDemandVerifier {
  private circuitBreaker: CircuitBreaker;
  private cache: CacheService;

  async verifyAvailability(
    contentId: string,
    platforms?: string[]
  ): Promise<Map<string, Availability>> {
    // Check cache first
    const cached = await this.cache.getAvailability(contentId);

    if (cached && this.isFresh(cached)) {
      return cached;
    }

    // Verify with platform APIs
    const platformsToCheck = platforms || await this.getAllPlatforms();

    const results = await this.circuitBreaker.execute(async () => {
      return Promise.all(
        platformsToCheck.map(platform =>
          this.checkPlatform(contentId, platform)
        )
      );
    });

    const availability = new Map(results);

    // Update cache
    await this.cache.setAvailability(contentId, availability);

    // Update database asynchronously
    this.updateDatabase(contentId, availability).catch(error => {
      logger.error('Failed to update database', error);
    });

    return availability;
  }

  private isFresh(cached: CachedAvailability): boolean {
    const age = Date.now() - cached.timestamp;
    return age < 3600000;  // 1 hour
  }

  private async checkPlatform(
    contentId: string,
    platform: string
  ): Promise<[string, Availability]> {
    const adapter = this.platforms.get(platform);

    try {
      const availability = await adapter.checkAvailability(contentId);
      return [platform, availability];
    } catch (error) {
      // Fallback to cached data on error
      const cached = await this.cache.getPlatformAvailability(contentId, platform);
      return [platform, cached || { available: false, lastVerified: new Date() }];
    }
  }
}
```

### 3. Real-Time Webhooks

```typescript
class WebhookHandler {
  private kafka: KafkaProducer;

  // Handle platform webhook
  async handleAvailabilityChange(webhook: PlatformWebhook): Promise<void> {
    // Verify webhook signature
    if (!this.verifySignature(webhook)) {
      throw new UnauthorizedError('Invalid webhook signature');
    }

    const { platform, contentId, availability } = webhook;

    // Update database
    await this.updateAvailability(platform, contentId, availability);

    // Invalidate cache
    await this.cache.invalidateAvailability(contentId);

    // Publish event
    await this.kafka.send({
      topic: 'availability-changed',
      messages: [{
        key: contentId,
        value: JSON.stringify({
          contentId,
          platform,
          availability,
          timestamp: Date.now()
        })
      }]
    });

    // Notify users with watchlist items
    await this.notifyWatchlistUsers(contentId, availability);
  }

  private verifySignature(webhook: PlatformWebhook): boolean {
    const signature = webhook.headers['x-platform-signature'];
    const secret = this.getPlatformSecret(webhook.platform);

    const expectedSignature = crypto
      .createHmac('sha256', secret)
      .update(JSON.stringify(webhook.body))
      .digest('hex');

    return crypto.timingSafeEqual(
      Buffer.from(signature),
      Buffer.from(expectedSignature)
    );
  }
}
```

### 4. Smart Priority System

```typescript
class PriorityCalculator {
  calculatePriority(content: Content, metrics: ContentMetrics): Priority {
    // Factors influencing priority
    const factors = {
      trending: this.calculateTrendingScore(metrics),
      watchlist: this.calculateWatchlistScore(metrics),
      recency: this.calculateRecencyScore(content),
      searches: this.calculateSearchScore(metrics),
      userInterest: this.calculateUserInterestScore(metrics)
    };

    // Weighted score
    const score =
      factors.trending * 0.3 +
      factors.watchlist * 0.25 +
      factors.recency * 0.15 +
      factors.searches * 0.2 +
      factors.userInterest * 0.1;

    // Assign priority tier
    if (score > 0.7) return Priority.HIGH;
    if (score > 0.4) return Priority.MEDIUM;
    return Priority.LOW;
  }

  private calculateTrendingScore(metrics: ContentMetrics): number {
    // Views in last 7 days
    const recentViews = metrics.viewsLastWeek;
    const maxViews = 10000;  // Normalization factor

    return Math.min(recentViews / maxViews, 1);
  }

  private calculateWatchlistScore(metrics: ContentMetrics): number {
    // Number of users with content in watchlist
    const watchlistCount = metrics.watchlistCount;
    const maxWatchlist = 5000;

    return Math.min(watchlistCount / maxWatchlist, 1);
  }

  private calculateRecencyScore(content: Content): number {
    // Recently released content gets higher priority
    const daysSinceRelease =
      (Date.now() - content.releaseDate.getTime()) / (1000 * 60 * 60 * 24);

    if (daysSinceRelease < 7) return 1.0;
    if (daysSinceRelease < 30) return 0.8;
    if (daysSinceRelease < 90) return 0.5;
    return 0.2;
  }
}
```

### 5. Availability Change Detection

```typescript
class ChangeDetector {
  async detectChanges(
    contentId: string,
    newAvailability: Map<string, Availability>
  ): Promise<AvailabilityChange[]> {
    // Get previous availability
    const previous = await this.getPreviousAvailability(contentId);

    const changes: AvailabilityChange[] = [];

    // Check each platform
    for (const [platform, current] of newAvailability.entries()) {
      const prev = previous.get(platform);

      // New platform availability
      if (!prev && current.available) {
        changes.push({
          type: 'added',
          platform,
          contentId,
          timestamp: new Date()
        });
      }

      // Platform removed availability
      if (prev?.available && !current.available) {
        changes.push({
          type: 'removed',
          platform,
          contentId,
          timestamp: new Date()
        });
      }

      // Price changed
      if (prev && this.priceChanged(prev, current)) {
        changes.push({
          type: 'price_changed',
          platform,
          contentId,
          oldPrice: prev.price,
          newPrice: current.price,
          timestamp: new Date()
        });
      }
    }

    return changes;
  }

  private priceChanged(
    prev: Availability,
    current: Availability
  ): boolean {
    return prev.price !== current.price;
  }
}
```

---

## Technology Stack

### Backend Services

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **API Gateway** | Kong / AWS API Gateway | Feature-rich, performant, ecosystem |
| **Query Service** | Node.js + TypeScript + Express | Fast I/O, async, type safety |
| **User Service** | Go + gRPC | Performance, concurrency, efficiency |
| **Content Service** | Node.js + TypeScript + GraphQL | Flexible queries, type system |
| **AI/NLP Engine** | Python + AWS Lambda | ML libraries, serverless scaling |
| **Recommendation** | Python + PyTorch + Lambda | ML frameworks, serverless |
| **Availability Checker** | Node.js + Step Functions | Orchestration, parallel execution |

### Data Layer

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Primary Database** | PostgreSQL 15 | ACID, JSON support, mature |
| **Vector Database** | Pinecone | Purpose-built, managed, fast |
| **Cache** | Redis 7 | In-memory, data structures |
| **Search Engine** | Elasticsearch 8 | Full-text search, analytics |
| **Object Storage** | AWS S3 | Durability, scalability, cost |
| **Message Queue** | Apache Kafka | High throughput, durability |

### AI/ML

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **LLM** | GPT-4 / Claude 3 | Query understanding, intent |
| **Embeddings** | text-embedding-3-large | Semantic search, 1536 dims |
| **NER** | spaCy | Entity extraction |
| **Recommendations** | PyTorch + Collaborative Filter | Flexible, production-ready |

### Infrastructure

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Cloud Provider** | AWS | Comprehensive services |
| **Container Orchestration** | Kubernetes (EKS) | Industry standard, scaling |
| **Service Mesh** | Istio | Traffic management, security |
| **CI/CD** | GitHub Actions | Integration, automation |
| **IaC** | Terraform | Multi-cloud, declarative |
| **Monitoring** | Datadog / Prometheus + Grafana | APM, metrics, alerting |
| **Logging** | ELK Stack | Centralized logging |
| **Tracing** | Jaeger | Distributed tracing |

### Frontend (Reference)

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Web** | React + TypeScript + Next.js | SSR, performance, SEO |
| **Mobile** | React Native | Code sharing, native perf |
| **State Management** | Redux Toolkit | Predictable state |
| **API Client** | Apollo Client (GraphQL) | Caching, optimistic UI |

---

## Architecture Decision Records

### ADR-001: Hybrid Microservices + Serverless Architecture

**Status:** Accepted
**Date:** 2025-12-05

**Context:**
We need to build a system that can handle variable workloads, scale to millions of users, and minimize operational costs.

**Decision:**
We will use a hybrid architecture combining microservices for stateful core services and serverless functions for variable workloads.

**Rationale:**
- **Microservices** provide better control for stateful services (User, Content)
- **Serverless** offers automatic scaling for variable AI/ML workloads
- Cost optimization: Pay only for actual usage for AI processing
- Operational simplicity: Managed infrastructure for Lambda functions

**Consequences:**
- **Positive:** Cost-effective scaling, reduced operational overhead
- **Negative:** Increased architectural complexity, cold start latency
- **Mitigation:** Use provisioned concurrency for critical Lambda functions

---

### ADR-002: Pinecone for Vector Database

**Status:** Accepted
**Date:** 2025-12-05

**Context:**
We need a vector database for semantic search with high performance and low latency.

**Decision:**
We will use Pinecone as our vector database for storing and querying content embeddings.

**Rationale:**
- Purpose-built for vector search (150x faster than alternatives)
- Fully managed service (no ops overhead)
- Built-in replication and high availability
- Metadata filtering support
- Auto-scaling capabilities

**Alternatives Considered:**
- **Weaviate:** Self-hosted, more operational complexity
- **Milvus:** Good performance but requires more infrastructure
- **pgvector:** PostgreSQL extension, good for small scale but doesn't scale as well

**Consequences:**
- **Positive:** Excellent performance, minimal ops overhead
- **Negative:** Vendor lock-in, cost at large scale
- **Mitigation:** Abstract vector DB interface for potential future migration

---

### ADR-003: CQRS for Read/Write Separation

**Status:** Accepted
**Date:** 2025-12-05

**Context:**
Read operations (searches, recommendations) vastly outnumber write operations and have different scaling requirements.

**Decision:**
We will implement CQRS (Command Query Responsibility Segregation) pattern separating read and write paths.

**Rationale:**
- Read-heavy workload benefits from dedicated optimization
- Write operations can focus on consistency
- Independent scaling of read and write services
- Cache-friendly architecture

**Implementation:**
- Write path: Primary PostgreSQL + Kafka events
- Read path: Read replicas + Redis cache + Elasticsearch
- Eventual consistency acceptable for most data

**Consequences:**
- **Positive:** Better performance, independent scaling
- **Negative:** Increased complexity, eventual consistency
- **Mitigation:** Clear consistency guarantees documented per endpoint

---

### ADR-004: Event-Driven Architecture with Kafka

**Status:** Accepted
**Date:** 2025-12-05

**Context:**
Multiple services need to react to availability changes, user events, and content updates.

**Decision:**
We will use Apache Kafka as our event streaming platform for inter-service communication.

**Rationale:**
- High throughput and durability
- Event sourcing capabilities
- Replay and reprocessing support
- Multiple consumer pattern
- Strong ecosystem

**Consequences:**
- **Positive:** Decoupled services, scalable event processing
- **Negative:** Operational complexity, eventual consistency
- **Mitigation:** Managed Kafka service (AWS MSK), monitoring tools

---

### ADR-005: JWT with RS256 for Authentication

**Status:** Accepted
**Date:** 2025-12-05

**Context:**
We need stateless authentication that scales horizontally without shared session storage.

**Decision:**
We will use JWT tokens with RS256 (asymmetric signing) for authentication.

**Rationale:**
- Stateless: No server-side session storage needed
- RS256: Public key verification allows any service to validate
- Standard: Well-supported across platforms
- Short-lived access tokens (15 min) + refresh tokens (7 days)

**Security Measures:**
- Token revocation list in Redis
- Refresh token rotation
- JTI (JWT ID) for tracking
- Secure key storage in AWS KMS

**Consequences:**
- **Positive:** Scalability, no session storage
- **Negative:** Token size, revocation complexity
- **Mitigation:** Keep payload minimal, implement revocation list

---

## Deployment Architecture

### Multi-Region Deployment

```
                    ┌─────────────────┐
                    │   Global CDN    │
                    │   (CloudFront)  │
                    └────────┬────────┘
                             │
                    ┌────────┴────────┐
                    │   Route 53      │
                    │  (DNS + Health) │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
              ▼                             ▼
    ┌──────────────────┐          ┌──────────────────┐
    │   US-WEST-2      │          │   US-EAST-1      │
    │   (Primary)      │          │   (Secondary)    │
    │                  │          │                  │
    │  ┌────────────┐  │          │  ┌────────────┐  │
    │  │    EKS     │  │          │  │    EKS     │  │
    │  │  Cluster   │  │          │  │  Cluster   │  │
    │  └────────────┘  │          │  └────────────┘  │
    │  ┌────────────┐  │          │  ┌────────────┐  │
    │  │   RDS      │──┼──────────┼─▶│    RDS     │  │
    │  │  Primary   │  │  Repl.   │  │  Replica   │  │
    │  └────────────┘  │          │  └────────────┘  │
    │  ┌────────────┐  │          │  ┌────────────┐  │
    │  │   Redis    │  │          │  │   Redis    │  │
    │  │  Cluster   │  │          │  │  Cluster   │  │
    │  └────────────┘  │          │  └────────────┘  │
    └──────────────────┘          └──────────────────┘
```

### Kubernetes Deployment

```yaml
# Query Service Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: query-service
  namespace: production
spec:
  replicas: 5
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 2
      maxUnavailable: 1
  selector:
    matchLabels:
      app: query-service
  template:
    metadata:
      labels:
        app: query-service
        version: v1.0.0
    spec:
      containers:
      - name: query-service
        image: query-service:1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: NODE_ENV
          value: production
        - name: DB_HOST
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: host
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
---
# Service
apiVersion: v1
kind: Service
metadata:
  name: query-service
  namespace: production
spec:
  selector:
    app: query-service
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
---
# HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: query-service-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: query-service
  minReplicas: 5
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## Monitoring & Observability

### Key Metrics

```typescript
// Application metrics
interface ApplicationMetrics {
  // Request metrics
  requestRate: number;              // Requests per second
  errorRate: number;                // Error rate percentage
  latencyP50: number;               // 50th percentile latency
  latencyP95: number;               // 95th percentile latency
  latencyP99: number;               // 99th percentile latency

  // Business metrics
  searchesPerMinute: number;
  activeUsers: number;
  cacheHitRate: number;

  // Infrastructure metrics
  cpuUtilization: number;
  memoryUtilization: number;
  diskIO: number;
  networkIO: number;

  // Database metrics
  dbConnectionPoolUtilization: number;
  dbQueryDuration: LatencyStats;
  dbSlowQueries: number;

  // External API metrics
  platformAPILatency: Map<string, LatencyStats>;
  platformAPIErrors: Map<string, number>;
  platformAPIRateLimit: Map<string, RateLimitStatus>;
}
```

### Alerting Rules

```yaml
# Prometheus alerting rules
groups:
  - name: availability
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }}%"

      - alert: HighLatency
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "P95 latency is {{ $value }}s"

      - alert: LowCacheHitRate
        expr: cache_hit_rate < 0.7
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Cache hit rate below threshold"
          description: "Cache hit rate is {{ $value }}"

  - name: resources
    interval: 30s
    rules:
      - alert: HighCPUUsage
        expr: avg(rate(container_cpu_usage_seconds_total[5m])) > 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}%"

      - alert: HighMemoryUsage
        expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}%"
```

### Distributed Tracing

```typescript
// OpenTelemetry tracing
import { trace, context } from '@opentelemetry/api';

class QueryService {
  private tracer = trace.getTracer('query-service');

  async handleSearch(query: string): Promise<SearchResult> {
    // Create span
    const span = this.tracer.startSpan('search.execute');

    try {
      // Add attributes
      span.setAttribute('query', query);
      span.setAttribute('user.id', context.userId);

      // NLP processing
      const nlpSpan = this.tracer.startSpan('nlp.process', {
        parent: span
      });
      const enriched = await this.nlpEngine.process(query);
      nlpSpan.end();

      // Vector search
      const searchSpan = this.tracer.startSpan('vector.search', {
        parent: span
      });
      const results = await this.vectorDB.search(enriched.embedding);
      searchSpan.end();

      // Rank results
      const rankSpan = this.tracer.startSpan('results.rank', {
        parent: span
      });
      const ranked = await this.ranker.rank(results);
      rankSpan.end();

      span.setStatus({ code: 0 });  // Success
      return ranked;

    } catch (error) {
      span.setStatus({ code: 2, message: error.message });  // Error
      throw error;
    } finally {
      span.end();
    }
  }
}
```

---

## Conclusion

This architecture provides a scalable, performant, and maintainable foundation for an AI-native media discovery platform. Key design decisions prioritize:

1. **Performance**: Sub-200ms response times through multi-layer caching and optimized data access patterns
2. **Scalability**: Horizontal scaling at every layer, supporting millions of concurrent users
3. **Reliability**: Fault tolerance through circuit breakers, retries, and multi-region deployment
4. **Security**: Zero-trust architecture with encryption at rest and in transit
5. **Maintainability**: Clear separation of concerns, comprehensive monitoring, and documentation

**Next Steps:**
1. Prototype core services (Query, User, Content)
2. Set up infrastructure with Terraform
3. Implement CI/CD pipelines
4. Load testing and performance tuning
5. Security audit and penetration testing
6. Gradual rollout with feature flags

**Estimated Timeline:**
- Infrastructure setup: 2 weeks
- Core services development: 8 weeks
- AI/ML integration: 4 weeks
- Testing and optimization: 4 weeks
- Security and compliance: 2 weeks
- **Total: 20 weeks (5 months)**

**Team Requirements:**
- 2 Backend Engineers (Node.js/Go)
- 1 ML Engineer (Python)
- 1 DevOps Engineer
- 1 QA Engineer
- 1 Security Engineer
