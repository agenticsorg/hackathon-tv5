#!/bin/bash
# Test script to verify ARW endpoint mime types and responses
# Run this after starting the dev server

set -e

echo "================================================"
echo "ARW Endpoint Testing Script"
echo "================================================"
echo ""
echo "Testing endpoints for correct Content-Type headers"
echo "and proper response formats..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Base URLs
WWW_URL="${WWW_URL:-http://localhost:3000}"
MARKETING_URL="${MARKETING_URL:-http://localhost:3001}"

# Test function
test_endpoint() {
    local url=$1
    local expected_type=$2
    local name=$3

    echo -n "Testing $name... "

    # Get content type
    content_type=$(curl -s -I "$url" 2>/dev/null | grep -i "content-type:" | sed 's/content-type: //i' | tr -d '\r\n' | awk '{print $1}')

    if [ -z "$content_type" ]; then
        echo -e "${RED}FAIL${NC} - No response"
        return 1
    fi

    # Check if content type matches (ignoring charset)
    if echo "$content_type" | grep -qi "$expected_type"; then
        echo -e "${GREEN}PASS${NC} - $content_type"
        return 0
    else
        echo -e "${RED}FAIL${NC} - Got: $content_type, Expected: $expected_type"
        return 1
    fi
}

# Test response format
test_format() {
    local url=$1
    local format=$2
    local name=$3

    echo -n "Testing $name format... "

    response=$(curl -s "$url" 2>/dev/null)

    if [ -z "$response" ]; then
        echo -e "${RED}FAIL${NC} - No response"
        return 1
    fi

    case $format in
        "yaml")
            if echo "$response" | grep -q "^version:"; then
                echo -e "${GREEN}PASS${NC} - Valid YAML"
                return 0
            else
                echo -e "${RED}FAIL${NC} - Invalid YAML format"
                return 1
            fi
            ;;
        "json")
            if echo "$response" | python3 -m json.tool > /dev/null 2>&1; then
                echo -e "${GREEN}PASS${NC} - Valid JSON"
                return 0
            else
                echo -e "${RED}FAIL${NC} - Invalid JSON format"
                return 1
            fi
            ;;
    esac
}

# Test AI-* headers
test_ai_headers() {
    local url=$1
    local name=$2

    echo -n "Testing $name AI headers... "

    ai_manifest=$(curl -s -I "$url" 2>/dev/null | grep -i "ai-manifest:" | sed 's/ai-manifest: //i' | tr -d '\r\n')

    if [ "$ai_manifest" = "true" ]; then
        echo -e "${GREEN}PASS${NC} - AI-Manifest header present"
        return 0
    else
        echo -e "${YELLOW}WARN${NC} - AI-Manifest header missing or incorrect"
        return 1
    fi
}

echo "================================================"
echo "Testing WWW App ($WWW_URL)"
echo "================================================"
echo ""

test_endpoint "$WWW_URL/llms.txt" "text/plain" "llms.txt Content-Type"
test_format "$WWW_URL/llms.txt" "yaml" "llms.txt"
test_ai_headers "$WWW_URL/llms.txt" "llms.txt"
echo ""

test_endpoint "$WWW_URL/llms.json" "application/json" "llms.json Content-Type"
test_format "$WWW_URL/llms.json" "json" "llms.json"
test_ai_headers "$WWW_URL/llms.json" "llms.json"
echo ""

test_endpoint "$WWW_URL/.well-known/arw-manifest.json" "application/json" "arw-manifest.json Content-Type"
test_format "$WWW_URL/.well-known/arw-manifest.json" "json" "arw-manifest.json"
test_ai_headers "$WWW_URL/.well-known/arw-manifest.json" "arw-manifest.json"
echo ""

echo "================================================"
echo "Testing Marketing App ($MARKETING_URL)"
echo "================================================"
echo ""

test_endpoint "$MARKETING_URL/llms.txt" "text/plain" "llms.txt Content-Type"
test_format "$MARKETING_URL/llms.txt" "yaml" "llms.txt"
test_ai_headers "$MARKETING_URL/llms.txt" "llms.txt"
echo ""

test_endpoint "$MARKETING_URL/llms.json" "application/json" "llms.json Content-Type"
test_format "$MARKETING_URL/llms.json" "json" "llms.json"
test_ai_headers "$MARKETING_URL/llms.json" "llms.json"
echo ""

test_endpoint "$MARKETING_URL/.well-known/arw-manifest.json" "application/json" "arw-manifest.json Content-Type"
test_format "$MARKETING_URL/.well-known/arw-manifest.json" "json" "arw-manifest.json"
test_ai_headers "$MARKETING_URL/.well-known/arw-manifest.json" "arw-manifest.json"
echo ""

echo "================================================"
echo "Testing Complete!"
echo "================================================"
echo ""
echo "Note: Start dev servers with:"
echo "  cd apps/www && npm run dev (port 3000)"
echo "  cd platform/apps/marketing && npm run dev (port 3001)"
echo ""
echo "Or set custom URLs:"
echo "  WWW_URL=https://arw.dev MARKETING_URL=https://arw.dev ./scripts/test-arw-endpoints.sh"
