# Media Gateway MCP Server

Model Context Protocol (MCP) server for AI-assisted content discovery and recommendation.

## Overview

The MCP Server implements the Model Context Protocol, enabling AI assistants like Claude to interact with the Media Gateway platform for intelligent content discovery, personalized recommendations, and watchlist management.

## Architecture

- **Protocol**: JSON-RPC 2.0
- **Transport**: HTTP/SSE (Server-Sent Events)
- **Port**: 3000
- **Framework**: Axum (Rust async web framework)

## Features

### Tools

1. **semantic_search** - Search content using natural language queries
   - Parameters: `query` (string), `limit` (int), `content_type` (optional)

2. **get_recommendations** - Get personalized content recommendations
   - Parameters: `user_id` (UUID), `limit` (int)

3. **check_availability** - Check content availability across platforms
   - Parameters: `content_id` (UUID)

4. **get_content_details** - Get detailed information about content
   - Parameters: `content_id` (UUID)

5. **sync_watchlist** - Synchronize user's watchlist across devices
   - Parameters: `user_id` (UUID), `device_id` (string)

### Resources

1. **content://catalog** - Complete content catalog with metadata
2. **user://preferences/{user_id}** - User preferences and settings
3. **content://item/{content_id}** - Detailed content item information

### Prompts

1. **discover_content** - Discover new content based on preferences
   - Arguments: `genre` (optional), `mood` (optional)

2. **find_similar** - Find content similar to a reference title
   - Arguments: `reference_title` (required)

3. **watchlist_suggestions** - Get suggestions to add to watchlist
   - Arguments: `user_id` (required)

## Configuration

Environment variables:

```bash
# Server configuration
MCP_HOST=0.0.0.0
MCP_PORT=3000

# Database
DATABASE_URL=postgresql://user:password@localhost/media_gateway

# Logging
RUST_LOG=info
```

## Running the Server

### Development

```bash
# From workspace root
cargo run --bin mcp-server

# With custom configuration
MCP_PORT=8080 cargo run --bin mcp-server
```

### Production

```bash
# Build optimized binary
cargo build --release --bin mcp-server

# Run
./target/release/mcp-server
```

### Docker

```bash
# Build image
docker build -f docker/mcp-server.Dockerfile -t media-gateway-mcp .

# Run container
docker run -p 3000:3000 \
  -e DATABASE_URL=postgresql://user:password@db/media_gateway \
  media-gateway-mcp
```

## API Examples

### Initialize Connection

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocol_version": "1.0",
    "capabilities": {},
    "client_info": {
      "name": "claude-desktop",
      "version": "1.0.0"
    }
  }
}
```

### List Available Tools

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

### Execute Semantic Search

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "semantic_search",
    "arguments": {
      "query": "action movies with complex plots",
      "limit": 10
    }
  }
}
```

### Read Resource

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/read",
  "params": {
    "uri": "content://catalog"
  }
}
```

### Get Prompt

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "prompts/get",
  "params": {
    "name": "discover_content",
    "arguments": {
      "genre": "sci-fi",
      "mood": "thought-provoking"
    }
  }
}
```

## Health Check

```bash
curl http://localhost:3000/health
```

Response:
```json
{
  "status": "healthy"
}
```

## Testing

```bash
# Run unit tests
cargo test --package media-gateway-mcp --lib

# Run integration tests
cargo test --package media-gateway-mcp --test integration_test

# Run all tests
cargo test --package media-gateway-mcp
```

## Integration with Claude Desktop

Add to Claude Desktop MCP configuration (`~/Library/Application Support/Claude/claude_desktop_config.json`):

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

## Protocol Compliance

- ✅ JSON-RPC 2.0 specification
- ✅ MCP 1.0 protocol
- ✅ STDIO transport (for Claude Desktop)
- ✅ HTTP/SSE transport (for web clients)
- ✅ Error handling and validation
- ✅ Tool execution
- ✅ Resource access
- ✅ Prompt templates

## Development

### Project Structure

```
crates/mcp-server/
├── src/
│   ├── lib.rs          # Library root and configuration
│   ├── main.rs         # Binary entry point
│   ├── protocol.rs     # MCP protocol types and JSON-RPC
│   ├── tools.rs        # Tool implementations
│   ├── resources.rs    # Resource handlers
│   └── handlers.rs     # HTTP/JSON-RPC request handlers
├── tests/
│   ├── integration_test.rs
│   └── fixtures/
│       └── test_content.sql
├── Cargo.toml
└── README.md
```

### Adding New Tools

1. Create tool struct in `src/tools.rs`
2. Implement `ToolExecutor` trait
3. Add `definition()` method with JSON schema
4. Register in `handle_tools_call()` in `src/handlers.rs`
5. Add to tools list in `handle_tools_list()`

### Adding New Resources

1. Add resource URI pattern to `list_resources()` in `src/resources.rs`
2. Implement read logic in `read_resource()`
3. Add database queries as needed

## License

MIT License - see LICENSE file for details
