# MCP Server Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     MCP SERVER ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐           ┌──────────────────┐            │
│  │   STDIO Client   │           │   HTTP Client    │            │
│  │ (Claude Desktop) │           │  (Web/Mobile)    │            │
│  └────────┬─────────┘           └────────┬─────────┘            │
│           │                              │                      │
│           │ JSON-RPC 2.0                 │ HTTP/SSE             │
│           │                              │                      │
│  ┌────────▼──────────┐           ┌──────▼──────────┐            │
│  │ STDIO Transport   │           │  SSE Transport  │            │
│  │  - Readline I/O   │           │  - Express.js   │            │
│  │  - Line protocol  │           │  - REST API     │            │
│  └────────┬──────────┘           │  - Event Stream │            │
│           │                      └──────┬──────────┘            │
│           │                             │                       │
│           └──────────┬──────────────────┘                       │
│                      │                                          │
│             ┌────────▼────────┐                                 │
│             │   MCP Server    │                                 │
│             │   Core Router   │                                 │
│             └────────┬────────┘                                 │
│                      │                                          │
│        ┌─────────────┼─────────────┐                            │
│        │             │             │                            │
│   ┌────▼────┐   ┌───▼────┐   ┌───▼─────┐                       │
│   │  Tools  │   │Resources│   │ Prompts │                       │
│   │ (7)     │   │  (7)    │   │  (3)    │                       │
│   └────┬────┘   └───┬────┘   └───┬─────┘                       │
│        │            │            │                              │
│        └────────────┼────────────┘                              │
│                     │                                           │
│        ┌────────────▼────────────┐                              │
│        │   Service Integration   │                              │
│        ├─────────────────────────┤                              │
│        │  - Discovery Service    │                              │
│        │  - Content Service      │                              │
│        │  - Recommendation (SONA)│                              │
│        │  - User Service         │                              │
│        └─────────────────────────┘                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. Transport Layer

**STDIO Transport** (`src/transports/stdio.ts`)
- Purpose: Desktop integration (Claude Desktop)
- Protocol: JSON-RPC 2.0 over stdin/stdout
- Line-based communication
- Signal handling (SIGINT, SIGTERM)
- Bidirectional messaging

**SSE Transport** (`src/transports/sse.ts`)
- Purpose: Web/HTTP integration
- Protocol: Server-Sent Events + REST
- Express.js HTTP server
- Multiple concurrent clients
- Event broadcasting

### 2. Core Server (`src/server.ts`)

**Request Router**:
- `initialize`: Server capability negotiation
- `tools/list`: List available tools
- `tools/call`: Execute tool
- `resources/list`: List available resources
- `resources/read`: Retrieve resource
- `prompts/list`: List available prompts
- `prompts/get`: Generate prompt

**Error Handling**:
- JSON-RPC 2.0 error codes
- Custom MCP error codes
- Detailed error messages
- Error context preservation

### 3. Tools System (`src/tools/`)

**Tool Architecture**:
```typescript
interface Tool {
  name: string;
  description: string;
  inputSchema: ZodSchema;
  executor: (input, context?) => Promise<Result>;
}
```

**Available Tools**:
1. `semantic_search`: Natural language search
2. `get_recommendations`: Personalized suggestions
3. `check_availability`: Platform availability
4. `get_content_details`: Content metadata
5. `list_devices`: User devices
6. `initiate_playback`: Start playback
7. `get_genres`: Genre catalog

### 4. Resources System (`src/resources/`)

**Resource Types**:
- Configuration (`hackathon://config`)
- Dynamic data (`media://trending`)
- Machine views (`llm://home`, `llm://search`)

**Resource Providers**:
- Static resources: Direct JSON
- Dynamic resources: Backend API calls
- Machine views: Token-optimized markdown

### 5. Authentication (`src/middleware/auth.ts`)

**Flow**:
```
Request → Extract JWT → Verify → Extract Context → Authorize → Next
```

**User Context**:
```typescript
{
  userId?: string;
  role?: string;
  scopes?: string[];
  tier?: 'free' | 'pro' | 'enterprise';
}
```

**Authorization**:
- Scope-based permissions
- Role-based access control
- Tier-based rate limits

### 6. ARW Manifest (`src/arw-manifest.ts`)

**Structure**:
```json
{
  "capabilities": { "mcp": { ... } },
  "content": { "machineViews": [...] },
  "actions": [...],
  "protocols": { "rest": {...}, "mcp": {...} },
  "policies": { ... }
}
```

## Data Flow

### Tool Execution Flow

```
1. Client sends request
   ↓
2. Transport receives and validates
   ↓
3. Auth middleware extracts context
   ↓
4. Server routes to tool handler
   ↓
5. Tool validates input (Zod)
   ↓
6. Tool executes logic
   ↓
7. Tool calls backend service
   ↓
8. Tool formats response
   ↓
9. Server wraps in MCP format
   ↓
10. Transport sends response
```

### Resource Retrieval Flow

```
1. Client requests resource
   ↓
2. Server validates URI
   ↓
3. Resource provider fetches data
   ↓
4. Data formatted (JSON/Markdown)
   ↓
5. Wrapped in MCP format
   ↓
6. Sent to client
```

## Performance Considerations

### Caching Strategy
- Resource caching: 5 minutes TTL
- Tool result caching: Context-dependent
- Client connection pooling

### Rate Limiting
- Unauthenticated: 10 req/15min
- Authenticated: 1000 req/15min
- Per-endpoint overrides

### Optimization Targets
- Request latency p95: <150ms
- MCP overhead: <50ms
- Token efficiency: 85% reduction (ARW)

## Security Architecture

### Layers
1. **Network**: HTTPS, CORS
2. **Authentication**: JWT verification
3. **Authorization**: Scope checking
4. **Validation**: Input sanitization (Zod)
5. **Rate Limiting**: DDoS prevention

### Threat Mitigation
- SQL Injection: N/A (no direct DB access)
- XSS: Helmet middleware
- CSRF: CORS configuration
- DoS: Rate limiting
- Token theft: Short expiry, HTTPS only

## Scalability

### Horizontal Scaling
- Stateless server design
- Client connection per instance
- Load balancer ready

### Vertical Scaling
- Async I/O throughout
- Connection pooling
- Resource caching

### Backend Integration
- Service discovery
- Circuit breakers
- Retry logic
- Fallback strategies

## Monitoring & Observability

### Metrics
- Request latency
- Error rates
- Tool execution time
- Active connections
- Rate limit hits

### Logging
- Structured JSON logs
- Request/response logging
- Error tracking
- Performance metrics

### Health Checks
- `/health` endpoint
- Service dependency checks
- Resource availability

## Deployment Architecture

### Container
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY dist/ ./dist/
CMD ["node", "dist/index.js"]
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mcp-server
  template:
    metadata:
      labels:
        app: mcp-server
    spec:
      containers:
      - name: mcp-server
        image: media-gateway/mcp-server:latest
        ports:
        - containerPort: 3000
        env:
        - name: TRANSPORT
          value: "sse"
```

## Future Enhancements

### Phase 2
- WebSocket transport
- GraphQL integration
- Batch tool execution
- Tool composition

### Phase 3
- Plugin system
- Custom tool registration
- Dynamic resource providers
- Multi-tenancy

### Phase 4
- Edge deployment
- Offline mode
- P2P transport
- Blockchain verification

## References

- MCP Specification: https://modelcontextprotocol.io
- ARW Specification: https://arw.agentics.org
- SPARC Architecture: /workspaces/media-gateway/plans/sparc/architecture/
- TypeScript Docs: https://typescriptlang.org

---

**Last Updated**: 2025-12-06
**Version**: 1.0.0
**Status**: Production Ready
