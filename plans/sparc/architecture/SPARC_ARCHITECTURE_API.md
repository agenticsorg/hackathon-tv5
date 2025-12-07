# SPARC Architecture Phase — Part 3: API Architecture

**Version:** 1.0.0
**Phase:** SPARC Architecture
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [API Gateway Design](#2-api-gateway-design)
3. [REST API Design](#3-rest-api-design)
4. [MCP Protocol Design](#4-mcp-protocol-design)
5. [GraphQL Considerations](#5-graphql-considerations)
6. [Real-time API Design](#6-real-time-api-design)
7. [API Security Architecture](#7-api-security-architecture)
8. [API Documentation Strategy](#8-api-documentation-strategy)
9. [API Contracts and Standards](#9-api-contracts-and-standards)
10. [Version Management and Deprecation](#10-version-management-and-deprecation)

---

## 1. Executive Summary

### 1.1 API Design Principles

The Media Gateway API architecture is designed around three core principles:

1. **Protocol Diversity**: Support REST, MCP, GraphQL, and WebSocket for different use cases
2. **AI-First Design**: Optimized for AI agent consumption via ARW protocol (85% token reduction)
3. **Production-Grade**: Circuit breakers, rate limiting, observability, and security built-in

### 1.2 API Layers

```
┌──────────────────────────────────────────────────────────────────┐
│                     API GATEWAY LAYER                            │
│  - Kong/Nginx API Gateway                                        │
│  - Authentication & Authorization                                │
│  - Rate Limiting & Circuit Breaking                              │
│  - Request/Response Transformation                               │
│  - AI-* Header Injection                                         │
└──────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   REST API    │    │   MCP Server  │    │   GraphQL     │
│   (Express)   │    │   (STDIO/SSE) │    │   (Apollo)    │
└───────────────┘    └───────────────┘    └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│                     SERVICE LAYER                                │
│  - Content Service                                               │
│  - Search Service                                                │
│  - Recommendation Service                                        │
│  - User State Service                                            │
└──────────────────────────────────────────────────────────────────┘
```

### 1.3 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **API Gateway: Kong** | Enterprise-grade, plugin ecosystem, OpenResty/Lua extensibility |
| **REST: Express.js** | TypeScript support, middleware ecosystem, performance |
| **MCP: Native Implementation** | Direct control, minimal overhead, ARW optimization |
| **GraphQL: Apollo Server** | Schema federation ready, excellent TypeScript support |
| **WebSocket: Socket.IO** | Fallback support, room-based broadcasting, reconnection |
| **Versioning: URL-based** | Clear visibility, easier routing, simpler deprecation |
| **Rate Limiting: Tiered** | User tier-based limits, per-endpoint overrides |

---

## 2. API Gateway Design

### 2.1 Gateway Architecture

```yaml
api_gateway:
  platform: kong
  version: "3.5.x"
  deployment:
    mode: "db-less"
    config_source: "declarative"
    high_availability: true
    instances: 3

  plugins:
    global:
      - name: "request-id"
        config:
          header_name: "X-Request-ID"
          echo_downstream: true

      - name: "correlation-id"
        config:
          header_name: "X-Correlation-ID"
          generator: "uuid#counter"

      - name: "ai-agent-detection"
        config:
          header_injection:
            - "AI-Request: true"
            - "AI-Agent: {detected_agent}"
            - "AI-Model: {detected_model}"

      - name: "rate-limiting"
        config:
          policy: "redis"
          redis_host: "valkey.cache.svc.cluster.local"
          limits:
            second: 100
            minute: 1000
            hour: 10000

      - name: "response-transformer"
        config:
          add:
            headers:
              - "X-Response-Time: {latency}ms"
              - "X-RateLimit-Remaining: {remaining}"
              - "X-Content-Version: 1.0"
```

### 2.2 Routing Strategy

```typescript
// Gateway Route Configuration
interface RouteConfig {
  service: string;
  protocol: 'http' | 'https' | 'grpc';
  paths: string[];
  methods: string[];
  stripPath: boolean;
  preserveHost: boolean;
}

const routes: RouteConfig[] = [
  {
    service: 'content-service',
    protocol: 'http',
    paths: ['/api/v1/content/*', '/api/v1/movies/*', '/api/v1/tv/*'],
    methods: ['GET', 'POST'],
    stripPath: true,
    preserveHost: false,
  },
  {
    service: 'search-service',
    protocol: 'http',
    paths: ['/api/v1/search', '/api/v1/discover'],
    methods: ['GET', 'POST'],
    stripPath: true,
    preserveHost: false,
  },
  {
    service: 'recommendation-service',
    protocol: 'http',
    paths: ['/api/v1/recommendations', '/api/v1/personalize'],
    methods: ['POST'],
    stripPath: true,
    preserveHost: false,
  },
  {
    service: 'mcp-server',
    protocol: 'https',
    paths: ['/mcp/*'],
    methods: ['GET', 'POST'],
    stripPath: false,
    preserveHost: true,
  },
  {
    service: 'graphql-server',
    protocol: 'http',
    paths: ['/graphql'],
    methods: ['POST', 'GET'],
    stripPath: false,
    preserveHost: false,
  },
];
```

### 2.3 Load Balancing

```yaml
load_balancing:
  algorithm: "consistent-hashing"
  hash_on: "header"
  hash_on_header: "X-User-ID"
  hash_fallback: "ip"

  health_checks:
    active:
      type: "http"
      http_path: "/health"
      healthy:
        interval: 5
        successes: 2
      unhealthy:
        interval: 5
        tcp_failures: 3
        http_failures: 3
        timeout: 3

    passive:
      healthy:
        successes: 5
      unhealthy:
        tcp_failures: 3
        http_failures: 5
        timeout: 5

  upstream_targets:
    content_service:
      - target: "content-service-0.svc:3000"
        weight: 100
      - target: "content-service-1.svc:3000"
        weight: 100
      - target: "content-service-2.svc:3000"
        weight: 100
```

### 2.4 Circuit Breaker Integration

```typescript
// Circuit Breaker Configuration (using Hystrix pattern)
interface CircuitBreakerConfig {
  requestVolumeThreshold: number;
  sleepWindowInMilliseconds: number;
  errorThresholdPercentage: number;
  timeout: number;
}

const circuitBreakerConfigs: Record<string, CircuitBreakerConfig> = {
  'content-service': {
    requestVolumeThreshold: 20,
    sleepWindowInMilliseconds: 5000,
    errorThresholdPercentage: 50,
    timeout: 3000,
  },
  'search-service': {
    requestVolumeThreshold: 10,
    sleepWindowInMilliseconds: 3000,
    errorThresholdPercentage: 40,
    timeout: 2000,
  },
  'recommendation-service': {
    requestVolumeThreshold: 15,
    sleepWindowInMilliseconds: 5000,
    errorThresholdPercentage: 50,
    timeout: 5000, // AI inference can be slower
  },
};

// Kong plugin configuration
const circuitBreakerPlugin = {
  name: 'circuit-breaker',
  config: {
    max_failures: 5,
    window_size: 60, // seconds
    min_throughput: 10,
    break_duration: 30, // seconds
    fallback_response: {
      status: 503,
      body: {
        error: 'SERVICE_UNAVAILABLE',
        message: 'Service temporarily unavailable. Please try again later.',
        retry_after: 30,
      },
    },
  },
};
```

---

## 3. REST API Design

### 3.1 Resource Hierarchy

```
/api/v1/
├── /content                    # Content catalog
│   ├── /movies
│   │   ├── /{id}
│   │   ├── /{id}/credits
│   │   ├── /{id}/images
│   │   ├── /{id}/availability
│   │   └── /{id}/similar
│   ├── /tv
│   │   ├── /{id}
│   │   ├── /{id}/seasons
│   │   ├── /{id}/seasons/{season_number}
│   │   ├── /{id}/seasons/{season_number}/episodes/{episode_number}
│   │   └── /{id}/credits
│   └── /trending
│
├── /search                     # Search operations
│   ├── /semantic              # AI-powered semantic search
│   ├── /faceted               # Traditional faceted search
│   └── /autocomplete          # Search suggestions
│
├── /discover                   # Content discovery
│   ├── /movies
│   ├── /tv
│   └── /popular
│
├── /recommendations            # Personalization
│   ├── /for-you              # User-specific
│   ├── /similar              # Content-based
│   └── /trending             # Popularity-based
│
├── /platforms                  # Streaming platforms
│   ├── /{platform_id}
│   ├── /{platform_id}/catalog
│   └── /{platform_id}/pricing
│
├── /user                       # User management
│   ├── /profile
│   ├── /watchlist
│   ├── /history
│   ├── /preferences
│   └── /state                 # CRDT-synced state
│
└── /genres                     # Metadata
    ├── /movies
    └── /tv
```

### 3.2 Core Endpoints

#### 3.2.1 Content Endpoints

| Endpoint | Method | Description | Rate Limit |
|----------|--------|-------------|------------|
| `/api/v1/content/movies/{id}` | GET | Get movie details | 1000/min |
| `/api/v1/content/movies/{id}/credits` | GET | Get cast and crew | 1000/min |
| `/api/v1/content/movies/{id}/availability` | GET | Where to watch | 500/min |
| `/api/v1/content/tv/{id}` | GET | Get TV show details | 1000/min |
| `/api/v1/content/tv/{id}/seasons/{season}` | GET | Get season details | 1000/min |
| `/api/v1/content/trending` | GET | Get trending content | 100/min |

**Example: Get Movie Details**

```http
GET /api/v1/content/movies/550 HTTP/1.1
Host: api.media-gateway.com
Authorization: Bearer {jwt_token}
Accept: application/json
X-Request-ID: 550e8400-e29b-41d4-a716-446655440000

Response 200 OK:
{
  "id": "550",
  "content_type": "movie",
  "title": "Fight Club",
  "original_title": "Fight Club",
  "overview": "A ticking-time-bomb insomniac...",
  "tagline": "Mischief. Mayhem. Soap.",
  "release_date": "1999-10-15",
  "runtime_minutes": 139,
  "genres": [
    { "id": 18, "name": "Drama" },
    { "id": 53, "name": "Thriller" }
  ],
  "external_ids": {
    "imdb_id": "tt0137523",
    "tmdb_id": 550,
    "eidr": "10.5240/FD9A-9A0E-5F1C-9E4D-7C6A-H"
  },
  "popularity_score": 0.87,
  "average_rating": 8.4,
  "vote_count": 28453,
  "availability": [
    {
      "platform_id": "netflix",
      "platform_name": "Netflix",
      "region": "US",
      "type": "subscription",
      "deep_link": "netflix://title/60011236",
      "updated_at": "2025-12-05T12:00:00Z"
    },
    {
      "platform_id": "prime_video",
      "platform_name": "Prime Video",
      "region": "US",
      "type": "rent",
      "price": 3.99,
      "currency": "USD",
      "deep_link": "https://www.amazon.com/dp/B0012Y4DXO",
      "updated_at": "2025-12-05T12:00:00Z"
    }
  ],
  "images": {
    "poster": "https://cdn.media-gateway.com/posters/550.jpg",
    "backdrop": "https://cdn.media-gateway.com/backdrops/550.jpg"
  },
  "_links": {
    "self": "/api/v1/content/movies/550",
    "credits": "/api/v1/content/movies/550/credits",
    "similar": "/api/v1/content/movies/550/similar",
    "availability": "/api/v1/content/movies/550/availability"
  }
}
```

#### 3.2.2 Search Endpoints

| Endpoint | Method | Description | Rate Limit |
|----------|--------|-------------|------------|
| `/api/v1/search/semantic` | POST | AI-powered search | 100/min |
| `/api/v1/search/faceted` | GET | Traditional search | 500/min |
| `/api/v1/search/autocomplete` | GET | Search suggestions | 1000/min |

**Example: Semantic Search**

```http
POST /api/v1/search/semantic HTTP/1.1
Host: api.media-gateway.com
Authorization: Bearer {jwt_token}
Content-Type: application/json
X-Request-ID: 550e8400-e29b-41d4-a716-446655440001

Request Body:
{
  "query": "mind-bending thrillers with unreliable narrators like Fight Club",
  "filters": {
    "media_type": "movie",
    "rating_min": 7.0,
    "release_year_min": 1990
  },
  "explain": true,
  "limit": 10
}

Response 200 OK:
{
  "query": "mind-bending thrillers with unreliable narrators like Fight Club",
  "results": [
    {
      "id": "155",
      "title": "The Sixth Sense",
      "content_type": "movie",
      "score": 0.92,
      "explanation": "Psychological thriller with a twist ending and unreliable narrative perspective",
      "availability_count": 3,
      "thumbnail": "https://cdn.media-gateway.com/posters/155_small.jpg"
    },
    {
      "id": "13",
      "title": "Memento",
      "content_type": "movie",
      "score": 0.89,
      "explanation": "Non-linear narrative with protagonist who cannot form new memories",
      "availability_count": 5,
      "thumbnail": "https://cdn.media-gateway.com/posters/13_small.jpg"
    }
  ],
  "total": 47,
  "processing_time_ms": 124,
  "embedding_dimensions": 1536,
  "_links": {
    "self": "/api/v1/search/semantic",
    "next": "/api/v1/search/semantic?cursor=eyJ0eXAiOiJKV1QiLCJhbGc..."
  }
}
```

#### 3.2.3 Recommendation Endpoints

| Endpoint | Method | Description | Rate Limit |
|----------|--------|-------------|------------|
| `/api/v1/recommendations/for-you` | POST | Personalized recommendations | 50/min |
| `/api/v1/recommendations/similar` | POST | Similar content | 200/min |
| `/api/v1/recommendations/trending` | GET | Trending content | 100/min |

**Example: Personalized Recommendations**

```http
POST /api/v1/recommendations/for-you HTTP/1.1
Host: api.media-gateway.com
Authorization: Bearer {jwt_token}
Content-Type: application/json

Request Body:
{
  "user_context": {
    "mood": "relaxed",
    "time_of_day": "evening",
    "device": "smart_tv"
  },
  "preferences": {
    "genres": [18, 35, 10749],  // Drama, Comedy, Romance
    "exclude_content_ids": ["550", "13", "155"]  // Already watched
  },
  "limit": 20
}

Response 200 OK:
{
  "recommendations": [
    {
      "id": "680",
      "title": "Pulp Fiction",
      "content_type": "movie",
      "score": 0.94,
      "reasoning": "Matches your preference for dialogue-driven dramas with dark humor",
      "sona_confidence": 0.89,
      "availability": [
        {
          "platform_id": "netflix",
          "region": "US",
          "type": "subscription"
        }
      ]
    }
  ],
  "total": 20,
  "model_version": "sona-v2.1",
  "generated_at": "2025-12-06T14:32:15Z"
}
```

### 3.3 HTTP Methods and Status Codes

#### 3.3.1 Method Usage

| Method | Usage | Idempotent | Safe |
|--------|-------|------------|------|
| GET | Retrieve resources | Yes | Yes |
| POST | Create resources, complex queries | No | No |
| PUT | Full resource replacement | Yes | No |
| PATCH | Partial resource update | No | No |
| DELETE | Remove resources | Yes | No |
| OPTIONS | CORS preflight | Yes | Yes |

#### 3.3.2 Status Code Standards

| Code | Usage | Example |
|------|-------|---------|
| 200 OK | Successful GET/PUT/PATCH | Content retrieved |
| 201 Created | Successful POST | Watchlist item added |
| 202 Accepted | Async operation queued | Background sync started |
| 204 No Content | Successful DELETE | Watchlist item removed |
| 304 Not Modified | Resource unchanged (ETag) | Content not modified |
| 400 Bad Request | Invalid request format | Missing required field |
| 401 Unauthorized | Invalid/missing auth | JWT expired |
| 403 Forbidden | Insufficient permissions | User tier limit reached |
| 404 Not Found | Resource doesn't exist | Content ID not found |
| 409 Conflict | Resource state conflict | CRDT merge conflict |
| 422 Unprocessable Entity | Validation error | Invalid date format |
| 429 Too Many Requests | Rate limit exceeded | 1000 req/min exceeded |
| 500 Internal Server Error | Server error | Database connection failed |
| 503 Service Unavailable | Circuit breaker open | Service degraded |

### 3.4 Pagination Strategy

```typescript
// Cursor-based pagination for large datasets
interface PaginationRequest {
  limit?: number;          // Default: 20, Max: 100
  cursor?: string;         // Opaque cursor token
}

interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    total?: number;        // Optional (expensive to compute)
    limit: number;
    has_more: boolean;
    next_cursor?: string;
    prev_cursor?: string;
  };
  _links: {
    self: string;
    next?: string;
    prev?: string;
  };
}

// Example: Cursor encoding (JWT-based)
interface CursorPayload {
  last_id: string;
  last_score?: number;     // For relevance-sorted results
  timestamp: number;
  direction: 'forward' | 'backward';
}
```

**Example: Paginated Request**

```http
GET /api/v1/discover/movies?limit=20&cursor=eyJhbGc... HTTP/1.1

Response 200 OK:
{
  "data": [...],
  "pagination": {
    "limit": 20,
    "has_more": true,
    "next_cursor": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  },
  "_links": {
    "self": "/api/v1/discover/movies?limit=20&cursor=eyJhbGc...",
    "next": "/api/v1/discover/movies?limit=20&cursor=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

### 3.5 Versioning Strategy

**URL-based versioning** (chosen for clarity and routing simplicity):

```
/api/v1/content/movies/{id}
/api/v2/content/movies/{id}
```

**Version Support Policy:**
- Latest version (v1): Full support
- Previous version (N-1): Maintenance mode (bug fixes only)
- Older versions (N-2): Deprecated (12-month sunset period)
- Sunset: Version removed

**Version Headers (for clients):**

```http
X-API-Version: 1.0
X-Supported-Versions: 1.0,2.0
X-Deprecated-Versions: 0.9
X-Sunset-Date: 2026-12-06
```

---

## 4. MCP Protocol Design

### 4.1 MCP Architecture Overview

The Model Context Protocol (MCP) provides AI-optimized access to Media Gateway functionality.

```
┌─────────────────────────────────────────────────────────┐
│                   MCP SERVER ARCHITECTURE                │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐           ┌─────────────┐             │
│  │   STDIO     │           │     SSE     │             │
│  │  Transport  │           │  Transport  │             │
│  └──────┬──────┘           └──────┬──────┘             │
│         │                         │                     │
│         └────────┬────────────────┘                     │
│                  │                                      │
│         ┌────────▼────────┐                             │
│         │   MCP Router    │                             │
│         │  - Tool calls   │                             │
│         │  - Resources    │                             │
│         │  - Prompts      │                             │
│         └────────┬────────┘                             │
│                  │                                      │
│    ┌─────────────┼─────────────┐                        │
│    │             │             │                        │
│    ▼             ▼             ▼                        │
│ ┌──────┐    ┌──────┐      ┌──────┐                     │
│ │Tools │    │Resources│    │Prompts│                    │
│ └──────┘    └──────┘      └──────┘                     │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 4.2 MCP Tool Definitions

```typescript
// MCP Tool: semantic_search
const semanticSearchTool: MCPTool = {
  name: 'semantic_search',
  description: 'Search for movies and TV shows using natural language queries',
  inputSchema: {
    type: 'object',
    properties: {
      query: {
        type: 'string',
        description: 'Natural language search query describing the desired content',
      },
      filters: {
        type: 'object',
        properties: {
          mediaType: {
            type: 'string',
            enum: ['movie', 'tv', 'all'],
            default: 'all',
          },
          ratingMin: {
            type: 'number',
            minimum: 0,
            maximum: 10,
            description: 'Minimum average rating (0-10)',
          },
          releaseYearMin: {
            type: 'number',
            description: 'Minimum release year',
          },
          releaseYearMax: {
            type: 'number',
            description: 'Maximum release year',
          },
        },
      },
      limit: {
        type: 'number',
        minimum: 1,
        maximum: 50,
        default: 10,
      },
      explain: {
        type: 'boolean',
        description: 'Include AI-generated explanations for each result',
        default: false,
      },
    },
    required: ['query'],
  },
};

// MCP Tool: get_recommendations
const getRecommendationsTool: MCPTool = {
  name: 'get_recommendations',
  description: 'Get personalized content recommendations based on user preferences or similar content',
  inputSchema: {
    type: 'object',
    properties: {
      basedOn: {
        type: 'object',
        properties: {
          contentId: {
            type: 'string',
            description: 'Content ID to find similar items for',
          },
          mediaType: {
            type: 'string',
            enum: ['movie', 'tv'],
          },
        },
      },
      preferences: {
        type: 'object',
        properties: {
          genres: {
            type: 'array',
            items: { type: 'number' },
            description: 'Preferred genre IDs',
          },
          mood: {
            type: 'string',
            description: 'Current mood (e.g., "relaxed", "excited", "thoughtful")',
          },
        },
      },
      limit: {
        type: 'number',
        minimum: 1,
        maximum: 50,
        default: 10,
      },
    },
  },
};

// MCP Tool: check_availability
const checkAvailabilityTool: MCPTool = {
  name: 'check_availability',
  description: 'Check where specific content is available to watch',
  inputSchema: {
    type: 'object',
    properties: {
      contentId: {
        type: 'string',
        description: 'Content ID to check availability for',
      },
      region: {
        type: 'string',
        description: 'Region code (e.g., "US", "UK", "CA")',
        default: 'US',
      },
      platforms: {
        type: 'array',
        items: { type: 'string' },
        description: 'Specific platforms to check (optional)',
      },
    },
    required: ['contentId'],
  },
};

// MCP Tool: get_content_details
const getContentDetailsTool: MCPTool = {
  name: 'get_content_details',
  description: 'Get detailed information about a specific movie or TV show',
  inputSchema: {
    type: 'object',
    properties: {
      contentId: {
        type: 'string',
        description: 'Content ID',
      },
      include: {
        type: 'array',
        items: {
          type: 'string',
          enum: ['credits', 'images', 'availability', 'similar'],
        },
        description: 'Additional information to include',
      },
    },
    required: ['contentId'],
  },
};

// Complete tool catalog
const mcpTools: MCPTool[] = [
  semanticSearchTool,
  getRecommendationsTool,
  checkAvailabilityTool,
  getContentDetailsTool,
];
```

### 4.3 MCP Resource Definitions

```typescript
// MCP Resource: Content metadata
interface MCPResource {
  uri: string;
  name: string;
  description: string;
  mimeType?: string;
}

const mcpResources: MCPResource[] = [
  {
    uri: 'content://movies/{id}',
    name: 'Movie Details',
    description: 'Complete movie metadata including credits, images, and availability',
    mimeType: 'application/json',
  },
  {
    uri: 'content://tv/{id}',
    name: 'TV Show Details',
    description: 'Complete TV show metadata including seasons, episodes, and availability',
    mimeType: 'application/json',
  },
  {
    uri: 'content://genres',
    name: 'Genre List',
    description: 'List of all available content genres',
    mimeType: 'application/json',
  },
  {
    uri: 'content://platforms',
    name: 'Streaming Platforms',
    description: 'List of supported streaming platforms',
    mimeType: 'application/json',
  },
  {
    uri: 'llm://home',
    name: 'Homepage Machine View',
    description: 'ARW-optimized homepage content for LLM consumption',
    mimeType: 'text/markdown',
  },
  {
    uri: 'llm://search',
    name: 'Search Machine View',
    description: 'ARW-optimized search interface for LLM consumption',
    mimeType: 'text/markdown',
  },
];
```

### 4.4 Transport Layer Design

#### 4.4.1 STDIO Transport (for Claude Desktop, etc.)

```typescript
// STDIO transport implementation
class MCPStdioTransport {
  private readline: readline.Interface;

  constructor() {
    this.readline = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
  }

  async start(): Promise<void> {
    this.readline.on('line', async (line) => {
      try {
        const request = JSON.parse(line);
        const response = await this.handleRequest(request);
        console.log(JSON.stringify(response));
      } catch (error) {
        console.log(JSON.stringify({
          jsonrpc: '2.0',
          error: {
            code: -32700,
            message: 'Parse error',
            data: error.message,
          },
          id: null,
        }));
      }
    });
  }

  private async handleRequest(request: MCPRequest): Promise<MCPResponse> {
    // Handle tool calls, resource requests, etc.
    const { method, params, id } = request;

    switch (method) {
      case 'tools/call':
        return this.handleToolCall(params, id);
      case 'resources/read':
        return this.handleResourceRead(params, id);
      case 'prompts/get':
        return this.handlePromptGet(params, id);
      default:
        return {
          jsonrpc: '2.0',
          error: {
            code: -32601,
            message: 'Method not found',
          },
          id,
        };
    }
  }
}
```

#### 4.4.2 SSE Transport (for web integrations)

```typescript
// SSE transport implementation (Express)
import express from 'express';
import { EventEmitter } from 'events';

class MCPSSETransport {
  private app: express.Application;
  private clients: Map<string, express.Response>;
  private events: EventEmitter;

  constructor() {
    this.app = express();
    this.clients = new Map();
    this.events = new EventEmitter();
    this.setupRoutes();
  }

  private setupRoutes(): void {
    // SSE endpoint for event streaming
    this.app.get('/mcp/events', (req, res) => {
      const clientId = req.query.clientId as string;

      res.setHeader('Content-Type', 'text/event-stream');
      res.setHeader('Cache-Control', 'no-cache');
      res.setHeader('Connection', 'keep-alive');

      this.clients.set(clientId, res);

      // Send initial connection event
      res.write('event: connected\n');
      res.write(`data: ${JSON.stringify({ clientId })}\n\n`);

      req.on('close', () => {
        this.clients.delete(clientId);
      });
    });

    // POST endpoint for tool calls
    this.app.post('/mcp/tools/call', async (req, res) => {
      const { toolName, arguments: args } = req.body;
      try {
        const result = await this.handleToolCall(toolName, args);
        res.json(result);
      } catch (error) {
        res.status(500).json({
          error: {
            code: 'TOOL_EXECUTION_ERROR',
            message: error.message,
          },
        });
      }
    });

    // GET endpoint for resources
    this.app.get('/mcp/resources/:resourceUri(*)', async (req, res) => {
      const { resourceUri } = req.params;
      try {
        const resource = await this.handleResourceRead(resourceUri);
        res.json(resource);
      } catch (error) {
        res.status(404).json({
          error: {
            code: 'RESOURCE_NOT_FOUND',
            message: error.message,
          },
        });
      }
    });
  }

  private async handleToolCall(toolName: string, args: any): Promise<any> {
    // Tool execution logic
  }

  private async handleResourceRead(uri: string): Promise<any> {
    // Resource retrieval logic
  }

  public sendEvent(clientId: string, event: string, data: any): void {
    const client = this.clients.get(clientId);
    if (client) {
      client.write(`event: ${event}\n`);
      client.write(`data: ${JSON.stringify(data)}\n\n`);
    }
  }
}
```

### 4.5 Error Handling

```typescript
// MCP error codes (JSON-RPC 2.0 compliant)
enum MCPErrorCode {
  PARSE_ERROR = -32700,
  INVALID_REQUEST = -32600,
  METHOD_NOT_FOUND = -32601,
  INVALID_PARAMS = -32602,
  INTERNAL_ERROR = -32603,

  // Custom error codes
  TOOL_NOT_FOUND = -32000,
  TOOL_EXECUTION_ERROR = -32001,
  RESOURCE_NOT_FOUND = -32002,
  RATE_LIMIT_EXCEEDED = -32003,
  AUTHORIZATION_ERROR = -32004,
}

interface MCPError {
  code: MCPErrorCode;
  message: string;
  data?: any;
}

// Error response format
interface MCPErrorResponse {
  jsonrpc: '2.0';
  error: MCPError;
  id: string | number | null;
}
```

---

## 5. GraphQL Considerations

### 5.1 Schema Design Approach

```graphql
# Core types
interface Content {
  id: ID!
  contentType: ContentType!
  title: String!
  originalTitle: String
  overview: String
  releaseDate: Date
  runtime: Int
  genres: [Genre!]!
  credits: Credits
  images: ContentImages
  availability(region: String = "US"): [PlatformAvailability!]!
  popularityScore: Float
  averageRating: Float
  voteCount: Int
}

type Movie implements Content {
  id: ID!
  contentType: ContentType!
  title: String!
  originalTitle: String
  overview: String
  releaseDate: Date
  runtime: Int!
  genres: [Genre!]!
  credits: Credits
  images: ContentImages
  availability(region: String = "US"): [PlatformAvailability!]!
  popularityScore: Float
  averageRating: Float
  voteCount: Int

  # Movie-specific fields
  budget: Int
  revenue: Int
  tagline: String
  similar(limit: Int = 10): [Movie!]!
}

type TVShow implements Content {
  id: ID!
  contentType: ContentType!
  title: String!
  originalTitle: String
  overview: String
  releaseDate: Date
  runtime: Int
  genres: [Genre!]!
  credits: Credits
  images: ContentImages
  availability(region: String = "US"): [PlatformAvailability!]!
  popularityScore: Float
  averageRating: Float
  voteCount: Int

  # TV-specific fields
  numberOfSeasons: Int!
  numberOfEpisodes: Int!
  status: TVShowStatus!
  seasons: [Season!]!
  nextEpisodeToAir: Episode
}

# Query root
type Query {
  # Content retrieval
  movie(id: ID!): Movie
  tvShow(id: ID!): TVShow
  content(id: ID!): Content

  # Search
  search(
    query: String!
    filters: SearchFilters
    limit: Int = 10
  ): SearchResults!

  # Discovery
  discover(
    category: DiscoverCategory!
    mediaType: MediaType = ALL
    limit: Int = 20
  ): DiscoverResults!

  # Recommendations
  recommendations(
    input: RecommendationInput!
  ): RecommendationResults!

  # Metadata
  genres(mediaType: MediaType): [Genre!]!
  platforms(region: String): [Platform!]!
}

# Mutation root
type Mutation {
  # User state
  addToWatchlist(contentId: ID!): Watchlist!
  removeFromWatchlist(contentId: ID!): Watchlist!
  updateUserPreferences(preferences: UserPreferencesInput!): UserPreferences!
}

# Subscription root
type Subscription {
  # Real-time updates
  contentUpdated(contentId: ID!): Content!
  availabilityChanged(contentId: ID!, region: String!): [PlatformAvailability!]!
  watchlistUpdated(userId: ID!): Watchlist!
}
```

### 5.2 Query Complexity Limits

```typescript
// Query complexity calculation
interface ComplexityConfig {
  maxDepth: number;
  maxComplexity: number;
  scalarCost: number;
  objectCost: number;
  listMultiplier: number;
}

const complexityConfig: ComplexityConfig = {
  maxDepth: 7,
  maxComplexity: 1000,
  scalarCost: 1,
  objectCost: 2,
  listMultiplier: 10,
};

// Field complexity estimators
const fieldComplexityEstimators = {
  Query: {
    search: ({ args }) => args.limit * 50,
    discover: ({ args }) => args.limit * 30,
    recommendations: ({ args }) => args.limit * 100, // AI inference is expensive
  },
  Movie: {
    similar: ({ args }) => args.limit * 40,
    availability: () => 20, // External API call
  },
  TVShow: {
    seasons: () => 50,
    episodes: ({ args }) => args.limit * 10,
  },
};
```

### 5.3 Subscription Support

```typescript
// GraphQL subscription implementation (Apollo Server)
import { PubSub } from 'graphql-subscriptions';

const pubsub = new PubSub();

const resolvers = {
  Subscription: {
    contentUpdated: {
      subscribe: (_, { contentId }) => {
        return pubsub.asyncIterator([`CONTENT_UPDATED_${contentId}`]);
      },
    },
    availabilityChanged: {
      subscribe: (_, { contentId, region }) => {
        return pubsub.asyncIterator([`AVAILABILITY_${contentId}_${region}`]);
      },
    },
  },
};
```

### 5.4 Federation Potential

```graphql
# User service (federated)
extend type User @key(fields: "id") {
  id: ID! @external
  watchlist: [Content!]!
  preferences: UserPreferences
}

# Content service (federated)
type Content @key(fields: "id") {
  id: ID!
  title: String!
}
```

---

## 6. Real-time API Design

### 6.1 WebSocket Architecture

```typescript
// Socket.IO implementation
import { Server as SocketIOServer } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { createClient } from 'redis';

class RealtimeServer {
  private io: SocketIOServer;
  private redisClient: ReturnType<typeof createClient>;
  private redisPub: ReturnType<typeof createClient>;
  private redisSub: ReturnType<typeof createClient>;

  constructor(httpServer: any) {
    this.io = new SocketIOServer(httpServer, {
      cors: {
        origin: process.env.ALLOWED_ORIGINS?.split(',') || '*',
        credentials: true,
      },
      transports: ['websocket', 'polling'],
      pingTimeout: 60000,
      pingInterval: 25000,
    });

    this.setupRedisAdapter();
    this.setupMiddleware();
    this.setupEventHandlers();
  }

  private async setupRedisAdapter() {
    this.redisPub = createClient({ host: 'valkey.cache.svc' });
    this.redisSub = this.redisPub.duplicate();

    await Promise.all([this.redisPub.connect(), this.redisSub.connect()]);

    this.io.adapter(createAdapter(this.redisPub, this.redisSub));
  }

  private setupMiddleware() {
    // Authentication middleware
    this.io.use(async (socket, next) => {
      const token = socket.handshake.auth.token;
      try {
        const user = await this.verifyToken(token);
        socket.data.user = user;
        next();
      } catch (error) {
        next(new Error('Authentication failed'));
      }
    });

    // Rate limiting middleware
    this.io.use(async (socket, next) => {
      const userId = socket.data.user.id;
      const key = `ratelimit:ws:${userId}`;
      const count = await this.redisClient.incr(key);
      if (count === 1) {
        await this.redisClient.expire(key, 60);
      }
      if (count > 100) {
        next(new Error('Rate limit exceeded'));
      } else {
        next();
      }
    });
  }

  private setupEventHandlers() {
    this.io.on('connection', (socket) => {
      const userId = socket.data.user.id;

      // Join user-specific room
      socket.join(`user:${userId}`);

      // Handle watchlist subscriptions
      socket.on('subscribe:watchlist', () => {
        socket.join(`watchlist:${userId}`);
      });

      // Handle content subscriptions
      socket.on('subscribe:content', ({ contentId }) => {
        socket.join(`content:${contentId}`);
      });

      // Handle user state sync
      socket.on('sync:state', async (state) => {
        await this.handleStateSync(userId, state);
      });

      // Handle disconnection
      socket.on('disconnect', () => {
        console.log(`User ${userId} disconnected`);
      });
    });
  }

  private async handleStateSync(userId: string, state: any) {
    // CRDT merge logic
    // Broadcast to other user sessions
    this.io.to(`user:${userId}`).emit('state:updated', state);
  }
}
```

### 6.2 Event Types

```typescript
// Real-time event definitions
enum RealtimeEvent {
  // Content events
  CONTENT_UPDATED = 'content:updated',
  AVAILABILITY_CHANGED = 'availability:changed',
  NEW_CONTENT = 'content:new',

  // User state events
  WATCHLIST_UPDATED = 'watchlist:updated',
  HISTORY_UPDATED = 'history:updated',
  PREFERENCES_UPDATED = 'preferences:updated',
  STATE_SYNCED = 'state:synced',

  // Recommendation events
  RECOMMENDATIONS_READY = 'recommendations:ready',
  RECOMMENDATIONS_UPDATED = 'recommendations:updated',

  // System events
  SYSTEM_NOTIFICATION = 'system:notification',
  RATE_LIMIT_WARNING = 'ratelimit:warning',
}
```

### 6.3 Channel Structure

```typescript
// Channel naming convention
const channels = {
  user: (userId: string) => `user:${userId}`,
  watchlist: (userId: string) => `watchlist:${userId}`,
  recommendations: (userId: string) => `recommendations:${userId}`,
  content: (contentId: string) => `content:${contentId}`,
  availability: (contentId: string, region: string) => `availability:${contentId}:${region}`,
  trending: () => 'global:trending',
  newContent: () => 'global:new-content',
};
```

### 6.4 Authentication for Connections

```typescript
// WebSocket authentication flow
class WebSocketAuth {
  static async authenticate(socket: Socket, next: (err?: Error) => void) {
    const token = socket.handshake.auth.token;

    if (!token) {
      return next(new Error('Authentication token required'));
    }

    try {
      const decoded = await this.verifyJWT(token);
      socket.data.user = decoded;
      next();
    } catch (error) {
      if (error.name === 'TokenExpiredError') {
        return next(new Error('TOKEN_EXPIRED'));
      }
      return next(new Error('INVALID_TOKEN'));
    }
  }

  static setupTokenRefresh(io: SocketIOServer) {
    io.on('connection', (socket) => {
      socket.on('auth:refresh', async ({ refreshToken }) => {
        try {
          const newTokens = await this.refreshTokens(refreshToken);
          socket.emit('auth:refreshed', newTokens);
        } catch (error) {
          socket.emit('auth:refresh:failed', { error: 'REFRESH_FAILED' });
          socket.disconnect(true);
        }
      });
    });
  }
}
```

### 6.5 Reconnection Strategy

```typescript
// Client-side reconnection strategy
interface ReconnectionConfig {
  attempts: number;
  delay: number;
  backoffMultiplier: number;
  maxDelay: number;
}

const reconnectionConfig: ReconnectionConfig = {
  attempts: 10,
  delay: 1000,
  backoffMultiplier: 1.5,
  maxDelay: 30000,
};

class RealtimeClient {
  private socket: Socket;
  private reconnectAttempts: number = 0;

  constructor(url: string, auth: { token: string }) {
    this.socket = io(url, {
      auth,
      reconnection: true,
      reconnectionAttempts: reconnectionConfig.attempts,
      reconnectionDelay: reconnectionConfig.delay,
      reconnectionDelayMax: reconnectionConfig.maxDelay,
      randomizationFactor: 0.5,
      transports: ['websocket', 'polling'],
    });

    this.setupReconnectionHandlers();
  }

  private setupReconnectionHandlers() {
    this.socket.on('connect', () => {
      console.log('Connected to real-time server');
      this.reconnectAttempts = 0;
      this.resubscribeToChannels();
    });

    this.socket.on('disconnect', (reason) => {
      console.log('Disconnected:', reason);
      if (reason === 'io server disconnect') {
        this.socket.connect();
      }
    });

    this.socket.on('reconnect_attempt', (attempt) => {
      console.log(`Reconnection attempt ${attempt}`);
      this.reconnectAttempts = attempt;
    });
  }

  private resubscribeToChannels() {
    // Resubscribe to all channels after reconnection
    const subscriptions = this.getStoredSubscriptions();
    subscriptions.forEach(sub => {
      this.socket.emit(`subscribe:${sub.type}`, sub.params);
    });
  }
}
```

---

## 7. API Security Architecture

### 7.1 Authentication Methods

```typescript
// Multi-method authentication system
enum AuthMethod {
  JWT_BEARER = 'jwt_bearer',
  API_KEY = 'api_key',
  OAUTH2 = 'oauth2',
}

const authConfig = {
  jwt: {
    algorithm: 'RS256' as const,
    issuer: 'media-gateway.com',
    audience: 'api.media-gateway.com',
    accessTokenExpiry: '15m',
    refreshTokenExpiry: '7d',
    publicKeyUrl: 'https://auth.media-gateway.com/.well-known/jwks.json',
  },
  apiKey: {
    headerName: 'X-API-Key',
    rateLimit: {
      free: '100/hour',
      pro: '1000/hour',
      enterprise: '10000/hour',
    },
  },
  oauth2: {
    providers: ['google', 'github'],
    scopes: ['profile', 'email'],
    callbackUrl: 'https://media-gateway.com/auth/callback',
  },
};
```

### 7.2 Authorization (RBAC and Scopes)

```typescript
// Role-Based Access Control
enum Role {
  ANONYMOUS = 'anonymous',
  USER = 'user',
  PRO_USER = 'pro_user',
  ADMIN = 'admin',
  SERVICE_ACCOUNT = 'service_account',
}

enum Permission {
  // Content permissions
  READ_CONTENT = 'content:read',
  SEARCH_CONTENT = 'content:search',

  // User permissions
  READ_PROFILE = 'profile:read',
  WRITE_PROFILE = 'profile:write',
  READ_WATCHLIST = 'watchlist:read',
  WRITE_WATCHLIST = 'watchlist:write',

  // Recommendations
  GET_RECOMMENDATIONS = 'recommendations:read',

  // Admin permissions
  MANAGE_CONTENT = 'content:manage',
  MANAGE_USERS = 'users:manage',
  VIEW_ANALYTICS = 'analytics:read',
}

const rolePermissions: Record<Role, Permission[]> = {
  [Role.ANONYMOUS]: [
    Permission.READ_CONTENT,
    Permission.SEARCH_CONTENT,
  ],
  [Role.USER]: [
    Permission.READ_CONTENT,
    Permission.SEARCH_CONTENT,
    Permission.READ_PROFILE,
    Permission.WRITE_PROFILE,
    Permission.READ_WATCHLIST,
    Permission.WRITE_WATCHLIST,
    Permission.GET_RECOMMENDATIONS,
  ],
  [Role.PRO_USER]: [
    ...rolePermissions[Role.USER],
  ],
  [Role.ADMIN]: [
    ...Object.values(Permission),
  ],
  [Role.SERVICE_ACCOUNT]: [
    Permission.READ_CONTENT,
    Permission.SEARCH_CONTENT,
  ],
};
```

### 7.3 Rate Limiting Tiers

```typescript
// Tiered rate limiting
interface RateLimitTier {
  name: string;
  limits: {
    perSecond: number;
    perMinute: number;
    perHour: number;
    perDay: number;
  };
  burstAllowance: number;
}

const rateLimitTiers: Record<string, RateLimitTier> = {
  anonymous: {
    name: 'Anonymous',
    limits: {
      perSecond: 5,
      perMinute: 100,
      perHour: 1000,
      perDay: 5000,
    },
    burstAllowance: 10,
  },
  free: {
    name: 'Free User',
    limits: {
      perSecond: 10,
      perMinute: 200,
      perHour: 2000,
      perDay: 10000,
    },
    burstAllowance: 20,
  },
  pro: {
    name: 'Pro User',
    limits: {
      perSecond: 50,
      perMinute: 1000,
      perHour: 10000,
      perDay: 100000,
    },
    burstAllowance: 100,
  },
  enterprise: {
    name: 'Enterprise',
    limits: {
      perSecond: 200,
      perMinute: 5000,
      perHour: 50000,
      perDay: 500000,
    },
    burstAllowance: 500,
  },
};

// Per-endpoint rate limit overrides
const endpointRateLimits: Record<string, Partial<RateLimitTier['limits']>> = {
  '/api/v1/search/semantic': {
    perMinute: 50,
    perHour: 500,
  },
  '/api/v1/recommendations/for-you': {
    perMinute: 20,
    perHour: 200,
  },
};
```

### 7.4 Input Validation

```typescript
// Request validation middleware (using Zod)
import { z } from 'zod';

// Common schemas
const commonSchemas = {
  id: z.string().uuid(),
  pagination: z.object({
    limit: z.number().int().min(1).max(100).default(20),
    cursor: z.string().optional(),
  }),
  region: z.string().regex(/^[A-Z]{2}$/).default('US'),
};

// Endpoint-specific validation schemas
const validationSchemas = {
  getMovie: z.object({
    params: z.object({
      id: commonSchemas.id,
    }),
    query: z.object({
      include: z.array(z.enum(['credits', 'images', 'availability', 'similar'])).optional(),
    }),
  }),

  semanticSearch: z.object({
    body: z.object({
      query: z.string().min(1).max(500),
      filters: z.object({
        mediaType: z.enum(['movie', 'tv', 'all']).default('all'),
        ratingMin: z.number().min(0).max(10).optional(),
        releaseYearMin: z.number().int().min(1900).max(2100).optional(),
        releaseYearMax: z.number().int().min(1900).max(2100).optional(),
      }).optional(),
      limit: z.number().int().min(1).max(50).default(10),
      explain: z.boolean().default(false),
    }),
  }),
};

// Validation middleware factory
function validate(schema: z.ZodSchema) {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      await schema.parseAsync({
        body: req.body,
        query: req.query,
        params: req.params,
      });
      next();
    } catch (error) {
      if (error instanceof z.ZodError) {
        res.status(422).json({
          error: {
            code: 'VALIDATION_ERROR',
            message: 'Invalid request parameters',
            details: error.errors,
          },
        });
      } else {
        next(error);
      }
    }
  };
}
```

---

## 8. API Documentation Strategy

### 8.1 OpenAPI 3.0 Specification

Complete OpenAPI spec available at: `/openapi.yaml`

Key sections:
- **Info**: API metadata, contact, license
- **Servers**: Production, staging, local URLs
- **Security**: Authentication schemes
- **Components**: Reusable schemas, responses, parameters
- **Paths**: All endpoints with examples
- **Tags**: Logical grouping of endpoints

### 8.2 ARW Manifest Structure

Located at `/.well-known/arw-manifest.json`

Key sections:
- **Site metadata**: Name, description, contact
- **Content**: Machine views for AI agents (85% token reduction)
- **Actions**: API endpoints with schemas
- **Protocols**: REST, MCP, GraphQL
- **Policies**: Training, inference, attribution, rate limits

### 8.3 SDK Generation

Auto-generated SDKs for:
- **TypeScript**: `@media-gateway/sdk`
- **Python**: `media-gateway-sdk`
- **Go**: `github.com/media-gateway/go-sdk`
- **Rust**: `media-gateway`

---

## 9. API Contracts and Standards

### 9.1 Request/Response Schemas

```typescript
// Standard response envelope
interface APIResponse<T = any> {
  data: T;
  meta: {
    request_id: string;
    response_time_ms: number;
    api_version: string;
    timestamp: string;
  };
  pagination?: {
    total?: number;
    limit: number;
    has_more: boolean;
    next_cursor?: string;
  };
  _links: {
    self: string;
    [key: string]: string;
  };
}

// Standard error response
interface APIErrorResponse {
  error: {
    code: string;
    message: string;
    details?: any;
    request_id: string;
    timestamp: string;
    documentation_url?: string;
  };
}
```

### 9.2 Error Response Format

```typescript
// Standardized error codes
enum ErrorCode {
  // Client errors (4xx)
  BAD_REQUEST = 'BAD_REQUEST',
  UNAUTHORIZED = 'UNAUTHORIZED',
  FORBIDDEN = 'FORBIDDEN',
  NOT_FOUND = 'NOT_FOUND',
  CONFLICT = 'CONFLICT',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',

  // Server errors (5xx)
  INTERNAL_SERVER_ERROR = 'INTERNAL_SERVER_ERROR',
  SERVICE_UNAVAILABLE = 'SERVICE_UNAVAILABLE',
  GATEWAY_TIMEOUT = 'GATEWAY_TIMEOUT',

  // Domain-specific errors
  CONTENT_NOT_FOUND = 'CONTENT_NOT_FOUND',
  PLATFORM_UNAVAILABLE = 'PLATFORM_UNAVAILABLE',
  SEARCH_FAILED = 'SEARCH_FAILED',
  RECOMMENDATION_FAILED = 'RECOMMENDATION_FAILED',
}
```

### 9.3 Deprecation Policy

```typescript
// API deprecation lifecycle
enum DeprecationPhase {
  ACTIVE = 'active',              // Fully supported
  MAINTENANCE = 'maintenance',    // Bug fixes only
  DEPRECATED = 'deprecated',      // Sunset announced
  SUNSET = 'sunset',              // No longer available
}

// Deprecation timeline
const deprecationTimeline = {
  announcement: 'T+0 months',
  migration_docs: 'T+0 months',
  warnings: 'T+3 months',
  maintenance_only: 'T+6 months',
  sunset: 'T+12 months',
};
```

---

## 10. Version Management and Deprecation

### 10.1 API Versioning Strategy

```yaml
versioning:
  strategy: url_based
  format: "/api/v{major}"
  current_version: "v1"
  supported_versions: ["v1"]
  deprecated_versions: []

  version_selection:
    explicit: "/api/v1/content/movies/123"
    default: "/api/content/movies/123"  # Defaults to v1
    header: "X-API-Version: 1"

  backward_compatibility:
    semantic_versioning: true
    support_policy: "n_minus_1"
```

### 10.2 Deprecation Workflow

```typescript
interface DeprecationAnnouncement {
  affected_endpoints: string[];
  reason: string;
  announcement_date: string;
  sunset_date: string;
  replacement: {
    version: string;
    endpoints: Record<string, string>;
  };
  migration_guide_url: string;
  support_contact: string;
}

// Deprecation notification in responses
function addDeprecationHeaders(res: Response, deprecation: DeprecationAnnouncement) {
  res.setHeader('Deprecation', 'true');
  res.setHeader('Sunset', new Date(deprecation.sunset_date).toUTCString());
  res.setHeader('Link', `<${deprecation.migration_guide_url}>; rel="deprecation"`);
  res.setHeader('X-API-Warn',
    `This endpoint will be removed on ${deprecation.sunset_date}. ` +
    `See ${deprecation.migration_guide_url} for migration guide.`
  );
}
```

---

## Summary

This API Architecture document provides comprehensive planning for the Media Gateway platform's API layer:

1. **API Gateway**: Kong-based gateway with circuit breakers, rate limiting, and AI agent detection
2. **REST API**: Well-structured resource hierarchy with cursor-based pagination and URL versioning
3. **MCP Protocol**: Native MCP implementation with STDIO/SSE transports for AI agent integration
4. **GraphQL**: Apollo Server with federation support and query complexity limits
5. **Real-time API**: Socket.IO-based WebSocket with Redis adapter for horizontal scaling
6. **Security**: Multi-method authentication (JWT, API keys, OAuth2) with RBAC and tiered rate limiting
7. **Documentation**: OpenAPI 3.0 spec with ARW manifest and auto-generated SDKs
8. **Contracts**: Standardized request/response formats with comprehensive error handling
9. **Versioning**: URL-based versioning with clear deprecation policy and migration support

**Key Design Decisions:**
- **Protocol diversity** supports different client types (web, mobile, AI agents)
- **ARW protocol** reduces AI agent token usage by 85%
- **Tiered rate limiting** balances access control with user experience
- **Circuit breakers** ensure resilience under failure
- **Comprehensive documentation** enables easy integration

This architecture is production-ready, scalable, and optimized for both human and AI agent consumption.

**Next Steps:**
- Proceed to SPARC Refinement phase (TDD implementation)
- Begin with API Gateway setup and routing
- Implement core REST endpoints
- Add MCP server integration
- Develop comprehensive test suite
