# MCP Connector Specification - Executive Summary

## Overview

This document provides a high-level summary of the MCP (Model Context Protocol) Connector Specification for the Media Gateway system. For complete details, see the full specification document.

**Document:** MCP Connector Specification v0.1.0
**Location:** `/workspaces/media-gateway/docs/specifications/mcp-connector-specification.md`
**Status:** Draft (SPARC Phase 1 - Specification)
**Date:** 2025-12-06

---

## What is the MCP Connector?

The MCP connector is a **protocol bridge** that exposes Media Gateway functionality to AI assistants (like Claude, Gemini) through a standardized interface. It enables natural language interaction with media discovery, streaming control, and content management services.

```
AI Assistant (Claude) → MCP Protocol → MCP Server → Media Gateway APIs
```

---

## Key Components

### 1. Transport Mechanisms

**Two Standard Transports:**

| Transport | Use Case | Characteristics |
|---|---|---|
| **STDIO** | Desktop AI assistants (Claude Desktop) | Single client, direct process communication, low latency |
| **SSE** | Web applications, cloud services | Multiple clients, HTTP-based, stateful connections |

Both use **JSON-RPC 2.0** message format.

### 2. Core Capabilities

The MCP server provides three primitives per MCP specification:

1. **Tools** (Model-controlled): Executable operations like `semantic_search`, `initiate_playback`
2. **Resources** (Application-controlled): Read-only data like catalogs, configurations
3. **Prompts** (User-controlled): Workflow templates for common tasks

### 3. Tool Categories

**10 Primary Tools across 4 categories:**

- **Media Ingestion**: `semantic_search`, `get_content_details`, `discover_content`, `get_recommendations`
- **Streaming Control**: `initiate_playback`, `control_playback`
- **Metadata Management**: `get_genres`, `update_user_preferences`
- **Device Interaction**: `list_devices`, `get_device_status`

---

## Implementation Requirements

### Mandatory (MUST)
- JSON-RPC 2.0 protocol compliance
- MCP protocol version 2024-11-05 support
- At least one transport implementation (STDIO or SSE)
- Minimum viable tool set (search, details, discover)
- Parameter validation against JSON Schema
- Standardized error handling
- OAuth 2.0 authentication (production)

### Recommended (SHOULD)
- Resource endpoints for configs/catalogs
- Prompt templates for workflows
- Session state management (SSE)
- Observability (logging, metrics, health checks)

### Optional (MAY)
- Response caching
- Connection pooling
- Batch operations
- Progress notifications

---

## Security Model

### Authentication
- OAuth 2.0 with Resource Indicators (RFC 8707) for SSE transport
- Token validation on every request
- Process isolation for STDIO transport

### Rate Limiting
- 100 requests / 15 min (unauthenticated)
- 1000 requests / 15 min (authenticated)
- HTTP 429 response when exceeded

### Input Validation
- JSON Schema validation for all parameters
- Input sanitization to prevent injection
- Maximum request size: 1 MB

---

## Performance Targets

| Metric | Target |
|---|---|
| Tool invocation latency (p50) | < 200ms |
| Tool invocation latency (p99) | < 2s |
| Concurrent connections (SSE) | > 1000 |
| Request timeout | 30s |
| Initialization time | < 5s |

---

## Integration Patterns

### 1. Initialization Flow
```
Client → initialize → MCP Server → Media Gateway (validate connectivity)
Client ← capabilities ← MCP Server
Client → tools/list → MCP Server
Client ← [tool definitions] ← MCP Server
```

### 2. Tool Invocation Flow
```
Client → tools/call: semantic_search(query) → MCP Server
MCP Server → Validate params + Parse intent
MCP Server → POST /api/search → Media Gateway
MCP Server ← Results ← Media Gateway
Client ← Formatted response ← MCP Server
```

### 3. Error Propagation
Media Gateway errors are mapped to JSON-RPC error codes:
- 400 Bad Request → `-32602` Invalid params
- 401 Unauthorized → `-32000` Authentication required
- 429 Rate Limit → `-32000` Rate limit exceeded
- 503 Unavailable → `-32603` Service unavailable

---

## Example Tool: Semantic Search

**Natural Language Query:**
```
"exciting sci-fi movies like Inception with plot twists"
```

**MCP Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "semantic_search",
    "arguments": {
      "query": "exciting sci-fi movies like Inception with plot twists",
      "filters": {
        "mediaType": "movie",
        "ratingMin": 7.0
      },
      "explain": true
    }
  }
}
```

**MCP Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"results\": [...15 movies...], \"totalCount\": 47, \"queryIntent\": {...}}"
    }]
  }
}
```

---

## Resource URI Scheme

Standard URIs for accessing Media Gateway data:

- `media://config` - Gateway configuration
- `media://genres` - Available content genres
- `media://devices` - Connected playback devices
- `media://catalog/movies` - Movie catalog
- `media://catalog/tv` - TV show catalog

---

## Technology Stack

### Required Dependencies
- **JSON-RPC 2.0**: Protocol implementation
- **Express.js** (SSE transport): HTTP server
- **readline** (STDIO transport): Line-based input parsing
- **Helmet**: Security headers
- **express-rate-limit**: Rate limiting middleware

### Optional Dependencies
- **OAuth client library**: Token validation
- **Zod/Ajv**: JSON Schema validation
- **Winston/Pino**: Structured logging

---

## Testing Strategy

### 1. Unit Tests
- Individual tool handlers
- Parameter validation logic
- Error handling paths
- Mocked Media Gateway responses

### 2. Integration Tests
- Full request/response cycles
- Transport implementations (STDIO + SSE)
- Live Media Gateway integration
- Error propagation verification

### 3. Security Tests
- Authentication bypass attempts
- Rate limiting enforcement
- Input injection attacks
- CORS restriction validation

### 4. Performance Tests
- Latency benchmarks (p50, p99)
- Concurrent connection stress testing
- Memory leak detection
- Timeout handling

---

## Configuration Example

```typescript
// Claude Desktop Config (STDIO)
{
  "mcpServers": {
    "media-gateway": {
      "command": "npx",
      "args": ["media-gateway-mcp", "mcp", "stdio"]
    }
  }
}

// SSE Server Config
{
  "transport": { "type": "sse", "port": 3000 },
  "security": {
    "enableAuth": true,
    "enableRateLimiting": true,
    "allowedOrigins": ["http://localhost:3000"]
  },
  "mediaGateway": {
    "baseUrl": "http://localhost:8080/api",
    "timeout": 20000
  }
}
```

---

## References

### Specifications
- [Model Context Protocol (MCP) Specification](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [ARW (Agent-Ready Web) Draft v0.1](https://github.com/agenticsorg/hackathon-tv5/blob/main/spec/ARW-0.1-draft.md)
- [OAuth 2.0 Resource Indicators - RFC 8707](https://www.rfc-editor.org/rfc/rfc8707.html)

### Implementation Examples
- Reference MCP Server: `/workspaces/media-gateway/apps/cli/src/mcp/`
- AgentDB MCP Server: `/workspaces/media-gateway/apps/agentdb/src/mcp/`
- Agentics Hackathon: [GitHub Repository](https://github.com/agenticsorg/hackathon-tv5)

### External Resources
- [MCP Spec Updates June 2025](https://auth0.com/blog/mcp-specs-update-all-about-auth/)
- [Complete Guide to MCP in 2025](https://www.keywordsai.co/blog/introduction-to-mcp)
- [VS Code MCP Integration](https://code.visualstudio.com/blogs/2025/06/12/full-mcp-spec-support)

---

## Next Steps (SPARC Methodology)

1. **Phase 2 - Pseudocode**: Design algorithm flows and data transformations
2. **Phase 3 - Architecture**: Define system components and interfaces
3. **Phase 4 - Refinement**: Implement with TDD approach
4. **Phase 5 - Completion**: Integration testing and deployment

---

**Document Version:** 0.1.0
**Last Updated:** 2025-12-06
**Author:** MCP Specialist Agent (Research & Analysis)
