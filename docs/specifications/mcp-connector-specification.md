# MCP Connector Specification for Media Gateway System

## Document Information

**Document Type:** Technical Specification - SPARC Phase 1 (Specification)
**Version:** 0.1.0
**Date:** 2025-12-06
**Status:** Draft
**Author:** MCP Specialist Agent (Research & Analysis)

---

## Executive Summary

This specification defines the Model Context Protocol (MCP) connector role within a Media Gateway system. The MCP server acts as a standardized interface layer that exposes media discovery, streaming control, and content management functionality to AI assistants and agentic systems. Based on analysis of the reference implementation at [hackathon-tv5](https://github.com/agenticsorg/hackathon-tv5) and the official MCP specification, this document establishes requirements for tool definitions, transport mechanisms, resource management, and integration patterns.

---

## 1. MCP Server Role and Responsibilities

### 1.1 Core Purpose

The MCP server serves as a **protocol bridge** between AI assistants (clients) and media gateway functionality, providing:

- **Standardized Tool Interface**: Exposes media operations as discoverable, invokable tools following JSON-RPC 2.0 protocol
- **Resource Management**: Provides access to media catalogs, content metadata, and configuration data
- **Prompt Templates**: Delivers context-aware workflow starters for common media discovery tasks
- **State Coordination**: Maintains session state and coordinates between multiple concurrent client connections

### 1.2 Architectural Position

```
┌─────────────────┐
│  AI Assistant   │
│  (Claude, etc)  │
└────────┬────────┘
         │ MCP Protocol (JSON-RPC 2.0)
         │ Transport: STDIO or SSE
         ▼
┌─────────────────┐
│   MCP Server    │ ◄── This Component
│  (Connector)    │
└────────┬────────┘
         │ Internal APIs
         ▼
┌─────────────────┐
│ Media Gateway   │
│  Core Services  │
│ • Discovery     │
│ • Streaming     │
│ • Metadata      │
└─────────────────┘
```

### 1.3 Capabilities Declaration

The MCP server MUST declare its capabilities during the initialization handshake:

```typescript
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "tools": {},          // Supports tool invocation
    "resources": {},      // Supports resource reading
    "prompts": {}         // Supports prompt templates
  },
  "serverInfo": {
    "name": "media-gateway-mcp",
    "version": "1.0.0"
  }
}
```

### 1.4 Responsibility Matrix

| Responsibility | Description | Implementation Requirement |
|---|---|---|
| **Tool Discovery** | List all available media gateway operations | REQUIRED |
| **Tool Invocation** | Execute media operations with parameter validation | REQUIRED |
| **Resource Provisioning** | Serve content catalogs and metadata | REQUIRED |
| **Prompt Management** | Provide workflow templates | RECOMMENDED |
| **Error Handling** | Return standardized JSON-RPC error codes | REQUIRED |
| **Session Management** | Track client connections and state | RECOMMENDED |
| **Rate Limiting** | Enforce usage quotas per client | RECOMMENDED |
| **Authentication** | Validate client authorization (OAuth 2.0) | REQUIRED (Production) |

---

## 2. MCP Transport Mechanisms

### 2.1 Transport Layer Architecture

The MCP specification defines two primary transport mechanisms, both operating over **JSON-RPC 2.0**:

#### 2.1.1 STDIO Transport

**Purpose**: Direct process communication for desktop AI assistants (e.g., Claude Desktop)

**Characteristics**:
- Synchronous, line-delimited JSON messages
- Single client per server process
- Lifetime tied to parent process
- Suitable for local development and desktop integration

**Implementation Requirements**:

```typescript
interface StdioTransport {
  // Read JSON-RPC requests from stdin
  input: readline.Interface;

  // Write JSON-RPC responses to stdout
  output: process.stdout;

  // Error logging to stderr (not stdout)
  errorLog: process.stderr;

  // Buffer for incomplete JSON messages
  messageBuffer: string;
}
```

**Connection Lifecycle**:

1. **Startup**: Server process launched via `npx media-gateway-mcp mcp stdio`
2. **Initialization**: Client sends `initialize` request
3. **Capability Negotiation**: Server responds with supported features
4. **Active Session**: Request/response message exchange
5. **Shutdown**: STDIN close triggers graceful cleanup
6. **Termination**: Process exit on uncaught exception or SIGTERM

**Example Configuration** (Claude Desktop):

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "npx",
      "args": ["media-gateway-mcp", "mcp", "stdio"]
    }
  }
}
```

#### 2.1.2 SSE (Server-Sent Events) Transport

**Purpose**: Web-based integrations supporting multiple concurrent clients

**Characteristics**:
- HTTP-based stateful connections
- Multiple simultaneous clients
- Persistent server process
- Suitable for web applications and cloud deployments

**Implementation Requirements**:

```typescript
interface SseTransport {
  // HTTP server instance
  httpServer: express.Application;

  // SSE endpoint for event streaming
  sseEndpoint: '/sse';  // GET

  // JSON-RPC endpoint for tool calls
  rpcEndpoint: '/rpc';  // POST

  // Health check endpoint
  healthEndpoint: '/health';  // GET

  // Security middleware
  security: {
    helmet: SecurityHeaders;
    rateLimiter: RateLimitConfig;
    cors: CorsConfig;
  };
}
```

**Connection Lifecycle**:

1. **Server Startup**: HTTP server binds to configured port
2. **Client Connection**: Client establishes SSE connection via GET /sse
3. **Keepalive**: Server sends periodic heartbeat events (30s interval)
4. **Tool Invocation**: Client POSTs JSON-RPC requests to /rpc
5. **Response Delivery**: Server returns JSON-RPC responses
6. **Disconnection**: Client closes connection or timeout expires
7. **Cleanup**: Server releases resources for that connection

**Security Requirements** (SSE Transport):

```typescript
interface SecurityConfig {
  // Helmet security headers
  contentSecurityPolicy: {
    defaultSrc: ["'self'"],
    scriptSrc: ["'self'"],
    styleSrc: ["'self'"]
  };

  // Rate limiting
  rateLimits: {
    windowMs: 900000,        // 15 minutes
    maxRequests: 100,        // Per IP address
    message: string;
  };

  // CORS restrictions
  allowedOrigins: string[];  // Localhost only for dev

  // Request timeout
  requestTimeout: 30000;     // 30 seconds
}
```

### 2.2 Transport Selection Criteria

| Criterion | STDIO | SSE |
|---|---|---|
| **Client Type** | Desktop AI assistants | Web applications, cloud services |
| **Concurrency** | Single client | Multiple clients |
| **Deployment** | Local development, CLI tools | Production servers, cloud platforms |
| **State Management** | Process-scoped | Server-scoped with session tracking |
| **Performance** | Low latency, direct IPC | Network overhead, connection pooling |
| **Security** | Process isolation | TLS, authentication, rate limiting |

### 2.3 Protocol Message Format

All transports use **JSON-RPC 2.0** message format:

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "semantic_search",
    "arguments": {
      "query": "exciting sci-fi movies like Inception"
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"results\": [...], \"count\": 15}"
    }]
  }
}
```

**Error Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Media service unavailable",
    "data": {
      "service": "tmdb-api",
      "reason": "rate_limit_exceeded"
    }
  }
}
```

---

## 3. MCP Tool Categories

### 3.1 Tool Definition Schema

Each MCP tool MUST conform to the following structure:

```typescript
interface McpTool {
  name: string;              // Unique identifier (snake_case)
  description: string;       // Human-readable purpose
  inputSchema: {
    type: 'object';
    properties: Record<string, JSONSchema>;
    required?: string[];
  };
}
```

### 3.2 Media Ingestion Tools

**Purpose**: Enable AI agents to search, discover, and retrieve media content metadata.

#### 3.2.1 Tool: `semantic_search`

**Description**: Search for movies and TV shows using natural language queries with intent understanding.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    query: {
      type: 'string',
      description: 'Natural language search query',
      examples: [
        'exciting sci-fi movies like Inception',
        'heartwarming comedy for family movie night',
        'dark thriller with plot twists'
      ]
    },
    filters: {
      type: 'object',
      properties: {
        mediaType: {
          type: 'string',
          enum: ['movie', 'tv', 'all'],
          default: 'all'
        },
        ratingMin: {
          type: 'number',
          minimum: 0,
          maximum: 10
        },
        yearRange: {
          type: 'object',
          properties: {
            min: { type: 'number' },
            max: { type: 'number' }
          }
        }
      }
    },
    explain: {
      type: 'boolean',
      description: 'Include AI-generated explanations for recommendations'
    }
  },
  required: ['query']
}
```

**Output Format**:
```typescript
{
  content: [{
    type: 'text',
    text: JSON.stringify({
      results: SearchResult[],
      totalCount: number,
      queryIntent: SearchIntent,
      executionTime: number
    })
  }]
}
```

#### 3.2.2 Tool: `get_content_details`

**Description**: Retrieve detailed metadata for a specific movie or TV show.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    contentId: {
      type: 'number',
      description: 'TMDB content identifier'
    },
    mediaType: {
      type: 'string',
      enum: ['movie', 'tv'],
      description: 'Content type'
    },
    includeRelated: {
      type: 'boolean',
      default: false,
      description: 'Include similar content recommendations'
    }
  },
  required: ['contentId', 'mediaType']
}
```

#### 3.2.3 Tool: `discover_content`

**Description**: Browse trending, popular, or genre-filtered content.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    category: {
      type: 'string',
      enum: ['trending', 'popular', 'top_rated', 'upcoming'],
      default: 'trending'
    },
    type: {
      type: 'string',
      enum: ['movie', 'tv', 'all'],
      default: 'all'
    },
    genres: {
      type: 'array',
      items: { type: 'number' },
      description: 'Genre IDs to filter by'
    },
    page: {
      type: 'number',
      default: 1,
      minimum: 1,
      maximum: 1000
    }
  }
}
```

### 3.3 Streaming Control Tools

**Purpose**: Manage playback sessions, device control, and streaming quality.

#### 3.3.1 Tool: `initiate_playback`

**Description**: Start a playback session for specified content.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    contentId: { type: 'number' },
    mediaType: { type: 'string', enum: ['movie', 'tv'] },
    deviceId: { type: 'string', description: 'Target playback device' },
    startPosition: { type: 'number', default: 0, description: 'Timestamp in seconds' },
    quality: { type: 'string', enum: ['auto', '720p', '1080p', '4k'], default: 'auto' }
  },
  required: ['contentId', 'mediaType']
}
```

#### 3.3.2 Tool: `control_playback`

**Description**: Pause, resume, seek, or adjust playback.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    sessionId: { type: 'string' },
    action: { type: 'string', enum: ['play', 'pause', 'seek', 'stop'] },
    position: { type: 'number', description: 'Seek position in seconds (for seek action)' }
  },
  required: ['sessionId', 'action']
}
```

### 3.4 Metadata Management Tools

**Purpose**: Access and update content metadata, genres, keywords, and user preferences.

#### 3.4.1 Tool: `get_genres`

**Description**: Retrieve available content genres.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    mediaType: { type: 'string', enum: ['movie', 'tv', 'all'], default: 'all' }
  }
}
```

#### 3.4.2 Tool: `update_user_preferences`

**Description**: Update user preference profile for personalized recommendations.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    userId: { type: 'string' },
    favoriteGenres: { type: 'array', items: { type: 'number' } },
    likedContent: { type: 'array', items: { type: 'number' } },
    dislikedContent: { type: 'array', items: { type: 'number' } }
  },
  required: ['userId']
}
```

### 3.5 Device Interaction Tools

**Purpose**: Discover and manage playback devices (smart TVs, streaming sticks, browsers).

#### 3.5.1 Tool: `list_devices`

**Description**: Discover available playback devices on the network.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    includeOffline: { type: 'boolean', default: false }
  }
}
```

#### 3.5.2 Tool: `get_device_status`

**Description**: Retrieve current status of a specific device.

**Input Schema**:
```typescript
{
  type: 'object',
  properties: {
    deviceId: { type: 'string' }
  },
  required: ['deviceId']
}
```

---

## 4. MCP Integration Patterns

### 4.1 Client-Server Communication Flows

#### 4.1.1 Initialization Flow

```
Client                          MCP Server                    Media Gateway
  │                                 │                              │
  ├─ initialize ────────────────────>│                              │
  │                                 ├─ Load config ────────────────>│
  │                                 │<── Config data ───────────────┤
  │<── capabilities ─────────────────┤                              │
  │                                 │                              │
  ├─ tools/list ────────────────────>│                              │
  │<── [tool definitions] ───────────┤                              │
  │                                 │                              │
```

**Requirements**:
- Server MUST respond to `initialize` within 5 seconds
- Server MUST declare all capabilities upfront
- Server SHOULD validate Media Gateway connectivity before confirming initialization

#### 4.1.2 Tool Invocation Flow

```
Client                          MCP Server                    Media Gateway
  │                                 │                              │
  ├─ tools/call: semantic_search ──>│                              │
  │                                 ├─ Validate params             │
  │                                 ├─ Parse query intent          │
  │                                 ├─ POST /api/search ──────────>│
  │                                 │                              ├─ Execute search
  │                                 │                              ├─ Rank results
  │                                 │<── Results [JSON] ────────────┤
  │                                 ├─ Format MCP response         │
  │<── result ───────────────────────┤                              │
  │                                 │                              │
```

**Requirements**:
- Server MUST validate all tool parameters against declared schema
- Server MUST return errors in JSON-RPC 2.0 format
- Server SHOULD implement request timeouts (default: 30s)
- Server SHOULD log all tool invocations for debugging

#### 4.1.3 Resource Access Flow

```
Client                          MCP Server                    Media Gateway
  │                                 │                              │
  ├─ resources/list ────────────────>│                              │
  │<── [resource URIs] ──────────────┤                              │
  │                                 │                              │
  ├─ resources/read ────────────────>│                              │
  │   uri: "media://config"         │                              │
  │                                 ├─ GET /api/config ───────────>│
  │                                 │<── Config data ───────────────┤
  │<── resource contents ────────────┤                              │
  │                                 │                              │
```

**Resource URI Scheme**:
- `media://config` - Gateway configuration
- `media://genres` - Available content genres
- `media://devices` - Connected playback devices
- `media://catalog/{type}` - Content catalogs (movies/tv)

### 4.2 Error Propagation Patterns

#### 4.2.1 Error Code Mapping

| Media Gateway Error | HTTP Status | JSON-RPC Code | MCP Error Message |
|---|---|---|---|
| Invalid parameters | 400 | -32602 | Invalid params: {details} |
| Not authenticated | 401 | -32000 | Authentication required |
| Rate limit exceeded | 429 | -32000 | Rate limit exceeded |
| Service unavailable | 503 | -32603 | Media service unavailable |
| Content not found | 404 | -32000 | Content not found: {id} |
| Network timeout | 504 | -32000 | Request timeout |

#### 4.2.2 Error Response Format

```typescript
interface McpError {
  jsonrpc: '2.0';
  id: string | number;
  error: {
    code: number;              // JSON-RPC error code
    message: string;           // Human-readable error
    data?: {
      service?: string;        // Which backend service failed
      reason?: string;         // Machine-readable error type
      retryable?: boolean;     // Can client retry?
      retryAfter?: number;     // Seconds to wait before retry
    };
  };
}
```

**Error Handling Requirements**:
- Server MUST catch all exceptions and convert to JSON-RPC errors
- Server MUST NOT leak internal implementation details in error messages
- Server SHOULD distinguish between client errors (4xx) and server errors (5xx)
- Server SHOULD provide actionable error messages

### 4.3 State Synchronization via MCP

#### 4.3.1 Session State Management

For **SSE transport**, the server maintains per-connection state:

```typescript
interface ClientSession {
  sessionId: string;
  clientId: string;
  connectedAt: Date;
  lastActivity: Date;
  state: {
    currentPlayback?: PlaybackSession;
    userPreferences?: UserPreferences;
    searchHistory: SearchQuery[];
  };
}
```

**State Synchronization Requirements**:
- Server MUST generate unique session IDs per connection
- Server SHOULD expire inactive sessions after 30 minutes
- Server MUST cleanup resources on session termination
- Server MAY persist critical state to database

#### 4.3.2 Playback Session Coordination

When multiple clients control the same playback device:

```typescript
interface PlaybackSession {
  sessionId: string;
  deviceId: string;
  contentId: number;
  mediaType: 'movie' | 'tv';
  position: number;          // Current playback position (seconds)
  state: 'playing' | 'paused' | 'stopped';
  controllers: string[];     // Client IDs with control access
  lastUpdate: Date;
}
```

**Coordination Pattern**:
1. Client A initiates playback via `initiate_playback`
2. Server creates `PlaybackSession` and returns `sessionId`
3. Client B requests device status via `get_device_status`
4. Server includes active `sessionId` in response
5. Client B can control playback using `sessionId`
6. Server broadcasts state changes to all controlling clients (SSE only)

#### 4.3.3 Preference Synchronization

User preferences sync across multiple client sessions:

```typescript
// Client A updates preferences
tools/call: update_user_preferences
  userId: "user-123"
  favoriteGenres: [28, 12, 878]  // Action, Adventure, Sci-Fi

// Server persists to Media Gateway
POST /api/users/user-123/preferences
  { favoriteGenres: [28, 12, 878] }

// Client B queries recommendations
tools/call: get_recommendations
  userId: "user-123"

// Server uses synced preferences
GET /api/recommendations?userId=user-123
  // Returns content matching [Action, Adventure, Sci-Fi]
```

---

## 5. Implementation Requirements Summary

### 5.1 Mandatory Requirements (MUST)

1. **Protocol Compliance**
   - Implement JSON-RPC 2.0 message format
   - Support MCP protocol version 2024-11-05 or later
   - Declare capabilities in `initialize` response

2. **Transport Support**
   - Implement at least one transport (STDIO or SSE)
   - Handle transport-specific lifecycle correctly

3. **Tool Implementation**
   - Expose minimum viable tool set (semantic_search, get_content_details, discover_content)
   - Validate all tool parameters against declared schema
   - Return responses in MCP-compliant format

4. **Error Handling**
   - Convert all errors to JSON-RPC error codes
   - Provide actionable error messages
   - Never expose internal system details

5. **Security (Production)**
   - Implement OAuth 2.0 authentication for SSE transport
   - Enforce rate limiting per client/IP
   - Use TLS for all network communication

### 5.2 Recommended Requirements (SHOULD)

1. **Resource Management**
   - Implement resource endpoints for configuration and catalogs
   - Support resource URI scheme

2. **Prompt Templates**
   - Provide workflow starters for common tasks
   - Support parameterized prompts

3. **State Management**
   - Track client sessions (SSE transport)
   - Persist critical playback state

4. **Observability**
   - Log all tool invocations
   - Emit metrics for latency and error rates
   - Provide health check endpoint

### 5.3 Optional Requirements (MAY)

1. **Advanced Features**
   - Sampling support for multi-step workflows
   - Progress notifications during long operations
   - Bi-directional event streaming (SSE)

2. **Performance Optimizations**
   - Response caching
   - Connection pooling to Media Gateway
   - Batch tool invocations

---

## 6. Security Considerations

### 6.1 Authentication and Authorization

**OAuth 2.0 Flow** (SSE Transport):

```
Client                          MCP Server                    Auth Provider
  │                                 │                              │
  ├─ Connect /sse ─────────────────>│                              │
  │                                 ├─ Validate token ────────────>│
  │                                 │<── Token valid ───────────────┤
  │<── SSE connected ────────────────┤                              │
  │                                 │                              │
  ├─ tools/call (with token) ──────>│                              │
  │                                 ├─ Check authorization         │
  │                                 ├─ Execute if authorized       │
  │<── result ───────────────────────┤                              │
  │                                 │                              │
```

**Requirements**:
- Server MUST validate OAuth tokens on every request (SSE)
- Server MUST implement Resource Indicators per RFC 8707 (MCP spec June 2025)
- Server SHOULD cache token validation results (TTL: 5 minutes)

### 6.2 Rate Limiting

**Per-Client Limits**:
- 100 requests per 15-minute window (unauthenticated)
- 1000 requests per 15-minute window (authenticated)
- 429 status code when exceeded
- `Retry-After` header in response

### 6.3 Input Validation

**Validation Requirements**:
- Validate all tool parameters against JSON Schema
- Reject requests with unknown properties
- Sanitize string inputs to prevent injection
- Enforce maximum request size (1 MB)

---

## 7. Performance Requirements

| Metric | Target | Measurement Method |
|---|---|---|
| Tool invocation latency (p50) | < 200ms | Server-side timing |
| Tool invocation latency (p99) | < 2s | Server-side timing |
| Concurrent connections (SSE) | > 1000 | Load testing |
| Request timeout | 30s | Enforced by server |
| Initialization time | < 5s | Client-side measurement |
| Memory per connection (SSE) | < 5 MB | Process monitoring |

---

## 8. Testing Requirements

### 8.1 Unit Tests

- Test each tool handler independently
- Mock Media Gateway responses
- Validate error handling paths
- Test parameter validation logic

### 8.2 Integration Tests

- Test full request/response cycle
- Verify transport implementations
- Test against live Media Gateway
- Validate error propagation

### 8.3 Security Tests

- Test authentication bypass attempts
- Verify rate limiting enforcement
- Test input injection attacks
- Validate CORS restrictions

---

## 9. References

### 9.1 Specifications

- **Model Context Protocol**: [Official Specification (2024-11-05)](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
- **JSON-RPC 2.0**: [Specification](https://www.jsonrpc.org/specification)
- **ARW (Agent-Ready Web)**: [Draft v0.1](https://github.com/agenticsorg/hackathon-tv5/blob/main/spec/ARW-0.1-draft.md)
- **OAuth 2.0 Resource Indicators**: [RFC 8707](https://www.rfc-editor.org/rfc/rfc8707.html)

### 9.2 Reference Implementations

- **Agentics Hackathon MCP Server**: `/workspaces/media-gateway/apps/cli/src/mcp/`
- **AgentDB MCP Server**: `/workspaces/media-gateway/apps/agentdb/src/mcp/`
- **Claude Flow SDK**: [GitHub Repository](https://github.com/ruvnet/claude-flow)

### 9.3 External Resources

- [Tools - Model Context Protocol](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
- [Model Context Protocol (MCP) Spec Updates from June 2025](https://auth0.com/blog/mcp-specs-update-all-about-auth/)
- [A Complete Guide to the Model Context Protocol (MCP) in 2025](https://www.keywordsai.co/blog/introduction-to-mcp)
- [Model Context Protocol - Wikipedia](https://en.wikipedia.org/wiki/Model_Context_Protocol)
- [The Complete MCP Experience: Full Specification Support in VS Code](https://code.visualstudio.com/blogs/2025/06/12/full-mcp-spec-support)

---

## 10. Appendices

### Appendix A: Complete Tool Catalog

| Tool Name | Category | Required | Description |
|---|---|---|---|
| `semantic_search` | Ingestion | Yes | Natural language content search |
| `get_content_details` | Ingestion | Yes | Retrieve content metadata |
| `discover_content` | Ingestion | Yes | Browse trending/popular content |
| `get_recommendations` | Ingestion | No | Personalized recommendations |
| `initiate_playback` | Streaming | No | Start playback session |
| `control_playback` | Streaming | No | Control active playback |
| `list_devices` | Device | No | Discover playback devices |
| `get_device_status` | Device | No | Query device state |
| `get_genres` | Metadata | No | Retrieve genre list |
| `update_user_preferences` | Metadata | No | Update user profile |

### Appendix B: Example MCP Server Configuration

```typescript
// apps/media-gateway-mcp/src/config.ts
export const mcpConfig = {
  server: {
    name: 'media-gateway-mcp',
    version: '1.0.0',
    protocolVersion: '2024-11-05'
  },

  transport: {
    type: 'sse' as const,
    port: 3000,
    host: '0.0.0.0'
  },

  security: {
    enableAuth: true,
    enableRateLimiting: true,
    allowedOrigins: ['http://localhost:3000', 'http://localhost:3001'],
    requestTimeout: 30000
  },

  mediaGateway: {
    baseUrl: 'http://localhost:8080/api',
    timeout: 20000,
    maxRetries: 3
  },

  logging: {
    level: 'info',
    format: 'json',
    destination: './logs/mcp-server.log'
  }
};
```

### Appendix C: Glossary

- **ARW**: Agent-Ready Web - Specification for optimizing web content for AI agents
- **JSON-RPC**: Remote procedure call protocol encoded in JSON
- **MCP**: Model Context Protocol - Standard for AI assistant integrations
- **SSE**: Server-Sent Events - HTTP-based event streaming protocol
- **STDIO**: Standard Input/Output - Process-based communication mechanism
- **TMDB**: The Movie Database - Media content metadata provider
- **Tool**: Named operation exposed via MCP that AI assistants can invoke
- **Resource**: Read-only data source accessible via URI
- **Prompt**: Template for initiating workflows with the AI assistant

---

**End of Specification Document**

**Next Steps**: Proceed to SPARC Phase 2 (Pseudocode) to design algorithm flows and data transformations for the MCP server implementation.
