#!/bin/bash
# Test script for STDIO transport
# This script demonstrates basic STDIO communication with the MCP server

echo "Testing MCP Server STDIO Transport"
echo "===================================="
echo ""

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Warning: DATABASE_URL not set. Using default."
    export DATABASE_URL="postgresql://localhost/media_gateway"
fi

# Test 1: Initialize request
echo "Test 1: Sending initialize request..."
echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocol_version":"1.0","capabilities":{},"client_info":{"name":"test-client","version":"1.0.0"}}}' | \
    timeout 5 cargo run --bin mcp-server --quiet -- --stdio 2>/dev/null | head -1

echo ""

# Test 2: Tools list request
echo "Test 2: Sending tools/list request..."
echo '{"jsonrpc":"2.0","id":"2","method":"tools/list"}' | \
    timeout 5 cargo run --bin mcp-server --quiet -- --stdio 2>/dev/null | head -1

echo ""

# Test 3: Invalid method (should return error)
echo "Test 3: Sending invalid method (should error)..."
echo '{"jsonrpc":"2.0","id":"3","method":"invalid_method"}' | \
    timeout 5 cargo run --bin mcp-server --quiet -- --stdio 2>/dev/null | head -1

echo ""

# Test 4: Parse error (malformed JSON)
echo "Test 4: Sending malformed JSON (should error)..."
echo '{invalid json}' | \
    timeout 5 cargo run --bin mcp-server --quiet -- --stdio 2>/dev/null | head -1

echo ""
echo "===================================="
echo "STDIO transport tests completed!"
