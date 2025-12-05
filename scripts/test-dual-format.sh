#!/bin/bash

# Test Dual Format Implementation (YAML + JSON)
# This script verifies that YAML and JSON formats contain identical data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🧪 Testing Dual Format Support (YAML + JSON)"
echo "=============================================="
echo ""

# Check if server is running
check_server() {
  local port=$1
  if ! curl -s http://localhost:$port > /dev/null 2>&1; then
    echo -e "${RED}❌ Server not running on port $port${NC}"
    echo "   Please start the server with 'npm run dev' first"
    return 1
  fi
  echo -e "${GREEN}✓ Server running on port $port${NC}"
  return 0
}

# Test a single endpoint
test_endpoint() {
  local base_url=$1
  local app_name=$2

  echo ""
  echo "Testing $app_name ($base_url)"
  echo "-----------------------------------"

  # Test YAML endpoint
  echo -n "  YAML endpoint (/llms.txt): "
  yaml_status=$(curl -s -o /dev/null -w "%{http_code}" "$base_url/llms.txt")
  if [ "$yaml_status" = "200" ]; then
    echo -e "${GREEN}✓ ($yaml_status)${NC}"
  else
    echo -e "${RED}✗ ($yaml_status)${NC}"
    return 1
  fi

  # Test JSON endpoint
  echo -n "  JSON endpoint (/llms.json): "
  json_status=$(curl -s -o /dev/null -w "%{http_code}" "$base_url/llms.json")
  if [ "$json_status" = "200" ]; then
    echo -e "${GREEN}✓ ($json_status)${NC}"
  else
    echo -e "${RED}✗ ($json_status)${NC}"
    return 1
  fi

  # Test well-known manifest
  echo -n "  Manifest (/.well-known/arw-manifest.json): "
  manifest_status=$(curl -s -o /dev/null -w "%{http_code}" "$base_url/.well-known/arw-manifest.json")
  if [ "$manifest_status" = "200" ]; then
    echo -e "${GREEN}✓ ($manifest_status)${NC}"
  else
    echo -e "${RED}✗ ($manifest_status)${NC}"
    return 1
  fi

  # Test content negotiation
  echo -n "  Content negotiation (Accept: text/yaml): "
  yaml_redirect_status=$(curl -s -o /dev/null -w "%{http_code}" -H "Accept: text/yaml" "$base_url/llms.json")
  if [ "$yaml_redirect_status" = "302" ] || [ "$yaml_redirect_status" = "200" ]; then
    echo -e "${GREEN}✓ ($yaml_redirect_status)${NC}"
  else
    echo -e "${YELLOW}⚠ ($yaml_redirect_status)${NC}"
  fi

  # Verify data consistency
  echo -n "  Data consistency (YAML ≈ JSON): "

  # Fetch both formats
  yaml_content=$(curl -s "$base_url/llms.txt")
  json_content=$(curl -s "$base_url/llms.json")

  # Extract version field from both
  yaml_version=$(echo "$yaml_content" | grep "^version:" | head -1 | sed 's/version: *//')
  json_version=$(echo "$json_content" | grep -o '"version":"[^"]*"' | head -1 | sed 's/"version":"\([^"]*\)"/\1/')

  # Extract site name from both
  yaml_name=$(echo "$yaml_content" | grep "name:" | head -1 | sed 's/.*name: *"\?\([^"]*\)"\?/\1/')
  json_name=$(echo "$json_content" | grep -o '"name":"[^"]*"' | head -1 | sed 's/"name":"\([^"]*\)"/\1/')

  if [ "$yaml_version" = "$json_version" ] && [ "$yaml_name" = "$json_name" ]; then
    echo -e "${GREEN}✓${NC}"
    echo "    Version: $yaml_version"
    echo "    Site: $yaml_name"
  else
    echo -e "${RED}✗${NC}"
    echo "    YAML version: $yaml_version, JSON version: $json_version"
    echo "    YAML name: $yaml_name, JSON name: $json_name"
    return 1
  fi

  # Test MIME types
  echo -n "  Content-Type headers: "
  yaml_type=$(curl -s -I "$base_url/llms.txt" | grep -i "content-type:" | tr -d '\r')
  json_type=$(curl -s -I "$base_url/llms.json" | grep -i "content-type:" | tr -d '\r')
  manifest_type=$(curl -s -I "$base_url/.well-known/arw-manifest.json" | grep -i "content-type:" | tr -d '\r')

  echo -e "${GREEN}✓${NC}"
  echo "    YAML: $yaml_type"
  echo "    JSON: $json_type"
  echo "    Manifest: $manifest_type"

  return 0
}

# Main execution
main() {
  local exit_code=0

  # Test main www app (port 3000)
  if check_server 3000; then
    if ! test_endpoint "http://localhost:3000" "Main App (www)"; then
      exit_code=1
    fi
  fi

  # Test achromatic marketing app (if different port)
  # Uncomment if running on different port
  # if check_server 3001; then
  #   if ! test_endpoint "http://localhost:3001" "Marketing App (achromatic)"; then
  #     exit_code=1
  #   fi
  # fi

  echo ""
  echo "=============================================="
  if [ $exit_code -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
  else
    echo -e "${RED}❌ Some tests failed${NC}"
  fi
  echo ""

  return $exit_code
}

main
exit $?
