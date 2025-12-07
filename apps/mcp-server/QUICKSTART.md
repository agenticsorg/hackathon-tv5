# MCP Server Quick Start Guide

Get the Media Gateway MCP server running in 5 minutes.

## Prerequisites

- Node.js 18+ installed
- npm or yarn package manager

## 1. Install Dependencies

```bash
cd apps/mcp-server
npm install
```

## 2. Configure Environment

```bash
cp .env.example .env
# Edit .env with your service URLs if needed
```

## 3. Build

```bash
npm run build
```

## 4. Run the Server

### Option A: STDIO (for Claude Desktop)

```bash
npm run start:stdio
```

Then configure Claude Desktop:

**File**: `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "node",
      "args": ["/absolute/path/to/media-gateway/apps/mcp-server/dist/index.js", "--transport", "stdio"]
    }
  }
}
```

### Option B: SSE (for HTTP/Web)

```bash
npm run start:sse
```

Server will start on `http://localhost:3000`

Test it:
```bash
curl http://localhost:3000/health
curl http://localhost:3000/.well-known/arw-manifest.json
```

## 5. Test the Tools

### Via HTTP (SSE mode)

```bash
# List available tools
curl http://localhost:3000/mcp/tools/list | jq

# Execute semantic search
curl -X POST http://localhost:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "toolName": "semantic_search",
    "arguments": {
      "query": "mind-bending sci-fi movies",
      "limit": 5
    }
  }' | jq

# Get genres
curl -X POST http://localhost:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "toolName": "get_genres",
    "arguments": {
      "mediaType": "all"
    }
  }' | jq

# Check resource
curl http://localhost:3000/mcp/resources/hackathon://config | jq
```

### Via Claude Desktop (STDIO mode)

After configuring Claude Desktop and restarting:

1. Open Claude Desktop
2. Look for "media-gateway" in the MCP tools section
3. Ask Claude: "Search for psychological thriller movies"
4. Claude will use the semantic_search tool automatically

## 6. Verify Installation

Check that all components are working:

```bash
# Health check
curl http://localhost:3000/health

# ARW manifest
curl http://localhost:3000/.well-known/arw-manifest.json | jq .capabilities

# List tools
curl http://localhost:3000/mcp/tools/list | jq '.tools[].name'

# List resources
curl http://localhost:3000/mcp/resources/list | jq '.resources[].uri'
```

Expected output:
- Health: `{"status":"healthy","timestamp":"..."}`
- 7 tools available
- 7 resources available
- ARW manifest with MCP capabilities

## Development Mode

For development with auto-reload:

```bash
npm run dev
```

## Troubleshooting

### Port already in use
```bash
# Change port in .env
PORT=3001
```

### Service URLs not responding
```bash
# Update service URLs in .env to point to running services
DISCOVERY_SERVICE_URL=http://localhost:4001
CONTENT_SERVICE_URL=http://localhost:4002
```

### STDIO not working in Claude Desktop
1. Check absolute path in config is correct
2. Ensure dist/index.js exists (`npm run build`)
3. Check Claude Desktop logs for errors
4. Restart Claude Desktop after config changes

## Next Steps

- Read [README.md](./README.md) for detailed documentation
- Check [Architecture Documentation](/workspaces/media-gateway/plans/sparc/architecture/SPARC_ARCHITECTURE_API.md)
- Review tool schemas in `src/tools/`
- Explore ARW manifest at `/.well-known/arw-manifest.json`
- Test with authentication (see README.md)

## Example Queries for Claude

Once configured with Claude Desktop, try:

1. **Discovery**: "Find me some mind-bending thriller movies from the last 10 years"
2. **Recommendations**: "I liked Inception and The Matrix, what else should I watch?"
3. **Availability**: "Where can I watch Fight Club?"
4. **Genres**: "What genres of movies are available?"
5. **Details**: "Tell me about the movie with ID 550"

## Performance Metrics

Expected performance (from SPARC specifications):

- **MCP request latency p95**: <150ms
- **Token efficiency**: 85% reduction vs HTML
- **Tool execution**: <100ms average
- **Rate limits**:
  - Unauthenticated: 10/15min
  - Authenticated: 1000/15min

## Support

Issues? Check:
- GitHub Issues: https://github.com/media-gateway/mcp-server/issues
- Documentation: `/workspaces/media-gateway/apps/mcp-server/README.md`
- Email: support@media-gateway.com
