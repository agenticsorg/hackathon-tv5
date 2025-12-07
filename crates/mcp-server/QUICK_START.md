# MCP Server Quick Start Guide

## Run the Server

### Development
```bash
# From workspace root
cargo run --bin mcp-server

# With custom port
MCP_PORT=8080 cargo run --bin mcp-server
```

### Production
```bash
# Build release binary
cargo build --release --bin mcp-server

# Run
./target/release/mcp-server
```

### Docker
```bash
# Build
docker build -f docker/mcp-server.Dockerfile -t media-gateway-mcp .

# Run
docker run -p 3000:3000 \
  -e DATABASE_URL=postgresql://user:pass@db/media_gateway \
  media-gateway-mcp
```

## Test the Server

### Health Check
```bash
curl http://localhost:3000/health
```

### Initialize Connection
```bash
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocol_version": "1.0",
      "capabilities": {},
      "client_info": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

### List Tools
```bash
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }'
```

### Execute Search
```bash
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "semantic_search",
      "arguments": {
        "query": "action movies",
        "limit": 5
      }
    }
  }'
```

## Environment Variables

```bash
# Server
export MCP_HOST=0.0.0.0
export MCP_PORT=3000

# Database
export DATABASE_URL=postgresql://user:password@localhost/media_gateway

# Logging
export RUST_LOG=info
```

## Available Tools

| Tool | Description | Arguments |
|------|-------------|-----------|
| `semantic_search` | Search content | query, limit, content_type |
| `get_recommendations` | Get recommendations | user_id, limit |
| `check_availability` | Check platform availability | content_id |
| `get_content_details` | Get content details | content_id |
| `sync_watchlist` | Sync watchlist | user_id, device_id |

## Available Resources

| URI Pattern | Description |
|-------------|-------------|
| `content://catalog` | Full content catalog |
| `user://preferences/{user_id}` | User preferences |
| `content://item/{content_id}` | Content item details |

## Available Prompts

| Prompt | Description | Arguments |
|--------|-------------|-----------|
| `discover_content` | Discover content | genre, mood |
| `find_similar` | Find similar content | reference_title |
| `watchlist_suggestions` | Watchlist suggestions | user_id |

## Run Tests

```bash
# Unit tests
cargo test --package media-gateway-mcp --lib

# Integration tests
cargo test --package media-gateway-mcp --test integration_test

# All tests
cargo test --package media-gateway-mcp
```

## Claude Desktop Integration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "/path/to/mcp-server",
      "env": {
        "DATABASE_URL": "postgresql://localhost/media_gateway"
      }
    }
  }
}
```

## Troubleshooting

### Server won't start
- Check DATABASE_URL is set correctly
- Verify PostgreSQL is running
- Check port 3000 is available

### Database connection fails
- Verify database exists
- Check connection string format
- Ensure user has proper permissions

### Tools return empty results
- Check database has content data
- Verify quality_score column exists
- Check migrations are applied

## More Information

- Full documentation: [README.md](./README.md)
- Implementation details: [/docs/BATCH_009_TASK_007_MCP_SERVER_IMPLEMENTATION.md](../../docs/BATCH_009_TASK_007_MCP_SERVER_IMPLEMENTATION.md)
- MCP Specification: https://modelcontextprotocol.io/
