# Media Gateway MCP Server

Model Context Protocol (MCP) server for AI agent integration with the Media Gateway platform.

## Features

- **7 MCP Tools**: Semantic search, recommendations, availability checking, content details, device management, playback control, and genre listing
- **7 MCP Resources**: Configuration, trending content, genres, platforms, and ARW-optimized machine views
- **3 MCP Prompts**: Discovery assistant, recommendation guide, and availability checker
- **Dual Transport**: STDIO (for Claude Desktop) and SSE (for web integration)
- **ARW Manifest**: 85% token reduction for AI agent consumption
- **Authentication**: JWT-based with scope-based authorization
- **Rate Limiting**: Tiered limits (10/15min unauthenticated, 1000/15min authenticated)

## Installation

```bash
npm install
npm run build
```

## Configuration

Copy `.env.example` to `.env` and configure:

```env
NODE_ENV=development
TRANSPORT=stdio
PORT=3000

# Service URLs
DISCOVERY_SERVICE_URL=http://localhost:4001
CONTENT_SERVICE_URL=http://localhost:4002
RECOMMENDATION_SERVICE_URL=http://localhost:4003
USER_SERVICE_URL=http://localhost:4004

# Authentication
JWT_SECRET=your-jwt-secret-here

# Rate Limiting
RATE_LIMIT_UNAUTHENTICATED=10
RATE_LIMIT_AUTHENTICATED=1000
```

## Usage

### STDIO Transport (Claude Desktop)

```bash
npm run start:stdio
```

**Claude Desktop Configuration** (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "npx",
      "args": ["@media-gateway/mcp-server", "--transport", "stdio"]
    }
  }
}
```

### SSE Transport (Web/HTTP)

```bash
npm run start:sse
# or
npm start -- --transport sse --port 3000
```

Access at: `http://localhost:3000`

## Available Tools

### 1. semantic_search

Search for movies and TV shows using natural language queries.

```typescript
{
  query: "mind-bending thrillers with unreliable narrators",
  filters: {
    mediaType: "movie",
    ratingMin: 7.5,
    releaseYearMin: 2000
  },
  limit: 10,
  explain: true
}
```

### 2. get_recommendations

Get personalized content recommendations.

```typescript
{
  preferences: {
    genres: [18, 35, 10749],  // Drama, Comedy, Romance
    mood: "relaxed"
  },
  limit: 20
}
```

### 3. check_availability

Check where specific content is available to watch.

```typescript
{
  contentId: "550",
  region: "US",
  platforms: ["netflix", "prime_video"]
}
```

### 4. get_content_details

Get detailed information about specific content.

```typescript
{
  contentId: "550",
  include: ["credits", "images", "availability", "similar"]
}
```

### 5. list_devices

List user's registered devices (requires authentication).

```typescript
{
  includeOffline: false
}
```

### 6. initiate_playback

Start content playback on a target device (requires `playback:control` scope).

```typescript
{
  contentId: "550",
  deviceId: "device-123",
  platformId: "netflix",
  startPosition: 0
}
```

### 7. get_genres

Get list of available content genres.

```typescript
{
  mediaType: "all"  // "movie", "tv", or "all"
}
```

## Available Resources

- `hackathon://config` - Hackathon configuration and metadata
- `hackathon://tracks` - Hackathon submission tracks
- `media://trending` - Currently trending content
- `media://genres` - All available genres
- `media://platforms` - Supported streaming platforms
- `llm://home` - ARW-optimized homepage (85% token reduction)
- `llm://search` - ARW-optimized search interface

## Available Prompts

- `discovery_assistant` - Content discovery guidance
- `recommendation_guide` - Recommendation explanations
- `availability_checker` - Availability checking workflow

## API Endpoints (SSE Transport)

### Health Check
```
GET /health
```

### ARW Manifest
```
GET /.well-known/arw-manifest.json
```

### Tool Execution
```
POST /mcp/tools/call
Content-Type: application/json
Authorization: Bearer <token>

{
  "toolName": "semantic_search",
  "arguments": {
    "query": "sci-fi movies",
    "limit": 10
  }
}
```

### Resource Retrieval
```
GET /mcp/resources/media://trending
```

### List Tools
```
GET /mcp/tools/list
```

### List Resources
```
GET /mcp/resources/list
```

## Performance Targets

From SPARC architecture specifications:

- **MCP request latency p95**: <150ms
- **MCP overhead**: <50ms
- **Token efficiency**: 85% reduction vs HTML scraping
- **Rate limits**:
  - Unauthenticated: 10 requests per 15 minutes
  - Authenticated: 1000 requests per 15 minutes

## Development

```bash
# Install dependencies
npm install

# Development mode with auto-reload
npm run dev

# Type checking
npm run typecheck

# Linting
npm run lint

# Build for production
npm run build

# Clean build artifacts
npm run clean
```

## Architecture

```
src/
├── index.ts                  # Main entry point
├── server.ts                 # MCP server core
├── config.ts                 # Configuration
├── arw-manifest.ts          # ARW manifest definition
├── types/
│   └── index.ts             # TypeScript type definitions
├── tools/
│   ├── index.ts             # Tool registry
│   ├── semantic_search.ts   # Semantic search tool
│   ├── get_recommendations.ts
│   ├── check_availability.ts
│   ├── get_content_details.ts
│   ├── list_devices.ts
│   ├── initiate_playback.ts
│   └── get_genres.ts
├── resources/
│   └── index.ts             # Resource definitions
├── prompts/
│   └── index.ts             # Prompt definitions
├── transports/
│   ├── stdio.ts             # STDIO transport
│   └── sse.ts               # SSE transport
└── middleware/
    └── auth.ts              # Authentication middleware
```

## Authentication

The server supports JWT-based authentication:

```bash
curl -X POST http://localhost:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-jwt-token>" \
  -d '{
    "toolName": "get_recommendations",
    "arguments": { "limit": 10 }
  }'
```

JWT payload should include:
- `sub` or `userId`: User identifier
- `role`: User role (default: "user")
- `scopes`: Array of permission scopes
- `tier`: User tier ("free", "pro", "enterprise")

## License

MIT

## Support

For issues or questions:
- GitHub: https://github.com/media-gateway/mcp-server
- Email: support@media-gateway.com
