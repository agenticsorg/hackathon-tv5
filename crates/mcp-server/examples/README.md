# MCP Server Examples

This directory contains example scripts and configurations for the Media Gateway MCP Server.

## Files

### test_stdio.sh

A bash script that demonstrates STDIO transport communication with the MCP server.

**Usage**:
```bash
cd /workspaces/media-gateway
export DATABASE_URL="postgresql://user:password@localhost/media_gateway"
./crates/mcp-server/examples/test_stdio.sh
```

**Tests performed**:
1. Initialize request - Establishes MCP connection
2. Tools list request - Retrieves available tools
3. Invalid method request - Verifies error handling
4. Malformed JSON request - Tests parse error handling

## Claude Desktop Configuration Examples

### Basic Configuration

Minimal configuration for Claude Desktop integration:

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "/path/to/mcp-server",
      "args": ["--stdio"],
      "env": {
        "DATABASE_URL": "postgresql://user:password@localhost/media_gateway"
      }
    }
  }
}
```

### Development Configuration

Configuration with debug logging enabled:

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "/path/to/target/debug/mcp-server",
      "args": ["--stdio"],
      "env": {
        "DATABASE_URL": "postgresql://localhost/media_gateway",
        "RUST_LOG": "debug,sqlx=warn"
      }
    }
  }
}
```

### Production Configuration

Optimized configuration for production use:

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "/usr/local/bin/mcp-server",
      "args": ["--stdio"],
      "env": {
        "DATABASE_URL": "postgresql://readonly_user:secure_password@prod-db.example.com:5432/media_gateway?sslmode=require",
        "RUST_LOG": "info,media_gateway_mcp=debug"
      }
    }
  }
}
```

## Manual Testing

### Interactive STDIO Testing

Start the server and interact manually:

```bash
# Terminal 1: Start server
export DATABASE_URL="postgresql://localhost/media_gateway"
cargo run --bin mcp-server -- --stdio

# Terminal 2: Send requests
# Copy/paste these one at a time:

# Initialize
{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocol_version":"1.0","capabilities":{},"client_info":{"name":"manual-test","version":"1.0"}}}

# List tools
{"jsonrpc":"2.0","id":"2","method":"tools/list"}

# List resources
{"jsonrpc":"2.0","id":"3","method":"resources/list"}

# List prompts
{"jsonrpc":"2.0","id":"4","method":"prompts/list"}

# Call semantic search tool
{"jsonrpc":"2.0","id":"5","method":"tools/call","params":{"name":"semantic_search","arguments":{"query":"science fiction movies","limit":5}}}
```

### Testing with Python

```python
#!/usr/bin/env python3
import subprocess
import json

# Start the MCP server
process = subprocess.Popen(
    ['./mcp-server', '--stdio'],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    env={'DATABASE_URL': 'postgresql://localhost/media_gateway'}
)

# Send initialize request
request = {
    "jsonrpc": "2.0",
    "id": "1",
    "method": "initialize",
    "params": {
        "protocol_version": "1.0",
        "capabilities": {},
        "client_info": {"name": "python-test", "version": "1.0"}
    }
}

process.stdin.write(json.dumps(request) + '\n')
process.stdin.flush()

# Read response
response = process.stdout.readline()
print(json.dumps(json.loads(response), indent=2))

# Send tools/list request
request = {
    "jsonrpc": "2.0",
    "id": "2",
    "method": "tools/list"
}

process.stdin.write(json.dumps(request) + '\n')
process.stdin.flush()

response = process.stdout.readline()
print(json.dumps(json.loads(response), indent=2))

process.terminate()
```

## HTTP Transport Testing

For comparison, you can also test the HTTP transport:

```bash
# Terminal 1: Start HTTP server
export DATABASE_URL="postgresql://localhost/media_gateway"
cargo run --bin mcp-server

# Terminal 2: Send HTTP requests
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocol_version":"1.0","capabilities":{},"client_info":{"name":"curl-test","version":"1.0"}}}'

curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"2","method":"tools/list"}'

# Health check
curl http://localhost:3000/health
```

## Troubleshooting

### Common Issues

1. **Database connection errors**:
   ```
   Error: Database connection failed
   ```
   Solution: Verify DATABASE_URL and ensure PostgreSQL is running

2. **Parse errors**:
   ```
   {"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error: ..."},"id":null}
   ```
   Solution: Ensure requests are valid JSON, one per line

3. **Method not found**:
   ```
   {"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found: ..."},"id":"1"}
   ```
   Solution: Check method name spelling and verify it's supported

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug,sqlx=info cargo run --bin mcp-server -- --stdio
```

### Performance Testing

Test with multiple concurrent requests:

```bash
# Generate 100 requests
for i in {1..100}; do
  echo "{\"jsonrpc\":\"2.0\",\"id\":\"$i\",\"method\":\"tools/list\"}"
done | cargo run --bin mcp-server --quiet --release -- --stdio
```

## Further Reading

- [STDIO Transport Documentation](../docs/STDIO_TRANSPORT.md)
- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [Main README](../../../README.md)
