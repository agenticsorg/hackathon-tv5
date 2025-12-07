# MCP Server STDIO Transport

## Overview

The Media Gateway MCP Server supports STDIO (Standard Input/Output) transport, which enables integration with Claude Desktop and other MCP clients that communicate via standard I/O streams.

## Features

- **Line-delimited JSON-RPC 2.0**: Each request and response is a single line of JSON
- **Asynchronous processing**: Built on Tokio for efficient async I/O
- **Error handling**: Robust error responses for malformed requests
- **Protocol compliance**: Full MCP 1.0 specification support

## Usage

### Starting the Server with STDIO

```bash
# Run the MCP server with STDIO transport
./mcp-server --stdio

# Or using cargo
cargo run --bin mcp-server -- --stdio
```

### Starting the Server with HTTP (Default)

```bash
# Run the MCP server with HTTP transport (default)
./mcp-server

# Or specify environment variables
MCP_HOST=0.0.0.0 MCP_PORT=3000 ./mcp-server
```

## Claude Desktop Integration

### Configuration

Add the following to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "/path/to/mcp-server",
      "args": ["--stdio"],
      "env": {
        "DATABASE_URL": "postgresql://user:password@localhost/media_gateway",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (required)
- `RUST_LOG`: Logging level (optional, default: `info`)
  - Levels: `error`, `warn`, `info`, `debug`, `trace`

## Protocol

### Request Format

Requests are line-delimited JSON-RPC 2.0 messages:

```json
{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocol_version":"1.0","capabilities":{},"client_info":{"name":"Claude Desktop","version":"1.0"}}}
```

### Response Format

Responses are also line-delimited JSON:

```json
{"jsonrpc":"2.0","id":"1","result":{"protocol_version":"1.0","capabilities":{"tools":{"list_changed":false},"resources":{"subscribe":false,"list_changed":false},"prompts":{"list_changed":false}},"server_info":{"name":"Media Gateway MCP Server","version":"0.1.0"}}}
```

### Error Responses

Errors follow JSON-RPC 2.0 error format:

```json
{"jsonrpc":"2.0","id":"1","error":{"code":-32601,"message":"Method not found: unknown_method"},"result":null}
```

## Supported Methods

### Initialization

- `initialize`: Initialize the MCP connection

### Tools

- `tools/list`: List all available tools
- `tools/call`: Execute a tool

Available tools:
- `semantic_search`: Search content using semantic similarity
- `get_recommendations`: Get personalized content recommendations
- `check_availability`: Check content availability across platforms
- `get_content_details`: Retrieve detailed content information
- `sync_watchlist`: Synchronize user watchlist

### Resources

- `resources/list`: List available resources
- `resources/read`: Read resource content

Resource types:
- `content://[id]`: Content metadata and details
- `user://[id]/preferences`: User viewing preferences
- `user://[id]/watchlist`: User watchlist items

### Prompts

- `prompts/list`: List available prompts
- `prompts/get`: Get a specific prompt

Available prompts:
- `discover_content`: Discover new content based on preferences
- `find_similar`: Find content similar to a reference title
- `watchlist_suggestions`: Get personalized watchlist suggestions

## Error Codes

Standard JSON-RPC 2.0 error codes:

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON was received |
| -32600 | Invalid Request | The JSON sent is not a valid Request object |
| -32601 | Method not found | The method does not exist / is not available |
| -32602 | Invalid params | Invalid method parameter(s) |
| -32603 | Internal error | Internal JSON-RPC error |

## Testing

### Unit Tests

```bash
# Run all MCP server tests
cargo test --package media-gateway-mcp

# Run STDIO transport tests specifically
cargo test --package media-gateway-mcp --test stdio_transport_test
```

### Manual Testing with STDIO

You can test the STDIO transport manually using stdin/stdout:

```bash
# Start the server
./mcp-server --stdio

# Send a request (paste this line and press Enter)
{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocol_version":"1.0","capabilities":{},"client_info":{"name":"test","version":"1.0"}}}

# You should receive an initialize response
```

### Testing with netcat (HTTP mode)

```bash
# Start HTTP server
./mcp-server

# In another terminal, send a request
echo '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":null}' | \
  curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d @-
```

## Architecture

### Transport Layer

The STDIO transport is implemented in `/crates/mcp-server/src/transport/stdio.rs`:

- Reads line-delimited JSON from stdin using Tokio's async I/O
- Parses each line as a JSON-RPC request
- Routes requests to appropriate handlers
- Serializes responses and writes to stdout
- Handles errors gracefully with proper JSON-RPC error responses

### Handler Reuse

The STDIO transport reuses the same handler logic as the HTTP transport:

- `handle_initialize`: Server initialization
- `handle_tools_list`: List available tools
- `handle_tools_call`: Execute tool with parameters
- `handle_resources_list`: List resources
- `handle_resources_read`: Read resource content
- `handle_prompts_list`: List prompts
- `handle_prompts_get`: Get prompt text

This ensures consistent behavior across both transport modes.

## Troubleshooting

### Connection Issues

1. **Database connection fails**:
   - Verify `DATABASE_URL` is set correctly
   - Ensure PostgreSQL is running
   - Check network connectivity

2. **Claude Desktop doesn't recognize the server**:
   - Verify the binary path in configuration
   - Check that `--stdio` flag is included in args
   - Review Claude Desktop logs for error messages

3. **Parse errors**:
   - Ensure requests are valid JSON
   - Verify one request per line
   - Check for proper JSON-RPC 2.0 format

### Logging

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug ./mcp-server --stdio
```

Logs are written to stderr (not stdout) to avoid interfering with the JSON-RPC protocol.

## Performance

The STDIO transport is optimized for:

- **Low latency**: Async I/O with Tokio runtime
- **Memory efficiency**: Streaming line-by-line parsing
- **Backpressure handling**: Proper flow control with async/await
- **Error recovery**: Continues processing after individual request failures

## Security Considerations

1. **Environment variables**: Sensitive data (DATABASE_URL) should use secure environment variable management
2. **Input validation**: All inputs are validated before processing
3. **Database isolation**: Uses connection pooling with proper resource limits
4. **Error messages**: Sensitive information is not leaked in error responses

## Future Enhancements

Planned improvements:

- [ ] Message batching support
- [ ] Compression for large responses
- [ ] Server-sent notifications
- [ ] Streaming responses for large datasets
- [ ] Hot-reload configuration support

## References

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Claude Desktop MCP Guide](https://modelcontextprotocol.io/docs/tools/claude-desktop)
