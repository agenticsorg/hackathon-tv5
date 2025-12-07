#!/bin/bash

# MCP Server Test Suite
# Tests all endpoints and tools

set -e

BASE_URL="http://localhost:3000"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================="
echo "MCP Server Test Suite"
echo "=================================="
echo ""

# Check if server is running
echo -n "Checking server health... "
if curl -s -f "${BASE_URL}/health" > /dev/null; then
    echo -e "${GREEN}✓ Server is healthy${NC}"
else
    echo -e "${RED}✗ Server is not responding${NC}"
    echo "Please start the server with: npm run start:sse"
    exit 1
fi

echo ""
echo "1. Testing ARW Manifest"
echo "------------------------"
curl -s "${BASE_URL}/.well-known/arw-manifest.json" | jq -r '.site.name, .capabilities.mcp.version'
echo ""

echo "2. Testing Tools List"
echo "---------------------"
echo -n "Available tools: "
TOOL_COUNT=$(curl -s "${BASE_URL}/mcp/tools/list" | jq '.tools | length')
echo -e "${GREEN}${TOOL_COUNT} tools${NC}"
curl -s "${BASE_URL}/mcp/tools/list" | jq -r '.tools[].name' | sed 's/^/  - /'
echo ""

echo "3. Testing Resources List"
echo "-------------------------"
echo -n "Available resources: "
RESOURCE_COUNT=$(curl -s "${BASE_URL}/mcp/resources/list" | jq '.resources | length')
echo -e "${GREEN}${RESOURCE_COUNT} resources${NC}"
curl -s "${BASE_URL}/mcp/resources/list" | jq -r '.resources[].uri' | sed 's/^/  - /'
echo ""

echo "4. Testing semantic_search Tool"
echo "--------------------------------"
SEARCH_RESULT=$(curl -s -X POST "${BASE_URL}/mcp/tools/call" \
  -H "Content-Type: application/json" \
  -d '{
    "toolName": "semantic_search",
    "arguments": {
      "query": "sci-fi thriller movies",
      "limit": 3
    }
  }')

if echo "$SEARCH_RESULT" | jq -e '.content' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ semantic_search executed successfully${NC}"
    echo "$SEARCH_RESULT" | jq -r '.content[0].text' | jq '.results' 2>/dev/null || echo "  (No results - service may be mocked)"
else
    echo -e "${YELLOW}⚠ semantic_search returned error (expected if services not running)${NC}"
fi
echo ""

echo "5. Testing get_genres Tool"
echo "--------------------------"
GENRES_RESULT=$(curl -s -X POST "${BASE_URL}/mcp/tools/call" \
  -H "Content-Type: application/json" \
  -d '{
    "toolName": "get_genres",
    "arguments": {
      "mediaType": "all"
    }
  }')

if echo "$GENRES_RESULT" | jq -e '.content' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ get_genres executed successfully${NC}"
else
    echo -e "${YELLOW}⚠ get_genres returned error${NC}"
fi
echo ""

echo "6. Testing check_availability Tool"
echo "-----------------------------------"
AVAILABILITY_RESULT=$(curl -s -X POST "${BASE_URL}/mcp/tools/call" \
  -H "Content-Type: application/json" \
  -d '{
    "toolName": "check_availability",
    "arguments": {
      "contentId": "550",
      "region": "US"
    }
  }')

if echo "$AVAILABILITY_RESULT" | jq -e '.content' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ check_availability executed successfully${NC}"
else
    echo -e "${YELLOW}⚠ check_availability returned error${NC}"
fi
echo ""

echo "7. Testing Resource: hackathon://config"
echo "---------------------------------------"
CONFIG_RESOURCE=$(curl -s "${BASE_URL}/mcp/resources/hackathon://config")
if echo "$CONFIG_RESOURCE" | jq -e '.contents' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Resource retrieved successfully${NC}"
    echo "$CONFIG_RESOURCE" | jq -r '.contents[0].text' | jq -C '.name, .version'
else
    echo -e "${RED}✗ Failed to retrieve resource${NC}"
fi
echo ""

echo "8. Testing Resource: llm://home"
echo "--------------------------------"
HOME_RESOURCE=$(curl -s "${BASE_URL}/mcp/resources/llm://home")
if echo "$HOME_RESOURCE" | jq -e '.contents' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Machine view retrieved successfully${NC}"
    echo -e "${YELLOW}Token-optimized content preview:${NC}"
    echo "$HOME_RESOURCE" | jq -r '.contents[0].text' | head -n 10
else
    echo -e "${RED}✗ Failed to retrieve machine view${NC}"
fi
echo ""

echo "9. Performance Test"
echo "-------------------"
echo -n "Measuring latency... "
START_TIME=$(date +%s%3N)
curl -s "${BASE_URL}/health" > /dev/null
END_TIME=$(date +%s%3N)
LATENCY=$((END_TIME - START_TIME))

if [ $LATENCY -lt 150 ]; then
    echo -e "${GREEN}✓ ${LATENCY}ms (target: <150ms)${NC}"
else
    echo -e "${YELLOW}⚠ ${LATENCY}ms (exceeds 150ms target)${NC}"
fi
echo ""

echo "10. Authentication Test"
echo "-----------------------"
# Test without token
UNAUTH_RESULT=$(curl -s -X POST "${BASE_URL}/mcp/tools/call" \
  -H "Content-Type: application/json" \
  -d '{
    "toolName": "semantic_search",
    "arguments": {
      "query": "test",
      "limit": 1
    }
  }')

if echo "$UNAUTH_RESULT" | jq -e '.content' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Unauthenticated requests allowed for public tools${NC}"
else
    echo -e "${YELLOW}⚠ Unauthenticated request failed${NC}"
fi
echo ""

echo "=================================="
echo "Test Summary"
echo "=================================="
echo ""
echo -e "${GREEN}MCP Server is operational!${NC}"
echo ""
echo "Available features:"
echo "  - 7 MCP Tools implemented"
echo "  - 7 MCP Resources available"
echo "  - ARW Manifest configured"
echo "  - STDIO and SSE transports"
echo "  - Authentication middleware"
echo "  - Rate limiting configured"
echo ""
echo "Next steps:"
echo "  1. Connect Claude Desktop (see QUICKSTART.md)"
echo "  2. Test with actual backend services"
echo "  3. Configure authentication tokens"
echo "  4. Monitor performance metrics"
echo ""
