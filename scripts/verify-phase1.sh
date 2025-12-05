#!/bin/bash
# Phase 1 Verification Script - MIME Types and Robots.txt Discovery Hints
# Tests all ARW endpoints for correct Content-Type headers

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Base URLs to test (modify for your environment)
BASE_URL_WWW="${BASE_URL_WWW:-http://localhost:3000}"
BASE_URL_MARKETING="${BASE_URL_MARKETING:-http://localhost:3001}"

echo "================================================"
echo "Phase 1 Verification - MIME Types & Discovery"
echo "================================================"
echo ""

# Function to check MIME type
check_mime_type() {
    local url=$1
    local expected=$2
    local endpoint_name=$3

    echo -n "Testing ${endpoint_name}... "

    # Get Content-Type header
    response=$(curl -I -s "${url}" 2>/dev/null || echo "FAILED")

    if [[ "$response" == "FAILED" ]]; then
        echo -e "${YELLOW}SKIP${NC} (endpoint not running)"
        return 1
    fi

    content_type=$(echo "$response" | grep -i "content-type:" | cut -d' ' -f2- | tr -d '\r')

    if [[ "$content_type" =~ $expected ]]; then
        echo -e "${GREEN}PASS${NC} (${content_type})"
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Expected: ${expected}"
        echo "  Got: ${content_type}"
        return 1
    fi
}

# Function to check robots.txt content
check_robots_txt() {
    local url=$1
    local name=$2

    echo -n "Testing ${name} robots.txt... "

    response=$(curl -s "${url}/robots.txt" 2>/dev/null || echo "FAILED")

    if [[ "$response" == "FAILED" ]]; then
        echo -e "${YELLOW}SKIP${NC} (endpoint not running)"
        return 1
    fi

    # Check for ARW hints (optional but recommended)
    if echo "$response" | grep -q "arw-manifest\|ARW-Manifest"; then
        echo -e "${GREEN}PASS${NC} (ARW hints present)"
        return 0
    else
        echo -e "${YELLOW}WARN${NC} (ARW hints missing - optional)"
        return 0
    fi
}

echo "=== Testing WWW App (${BASE_URL_WWW}) ==="
echo ""

# Test .llm.md files (if any exist)
check_mime_type "${BASE_URL_WWW}/blog/introducing-arw.llm.md" "text/markdown.*charset=utf-8" "WWW .llm.md"

# Test llms.txt
check_mime_type "${BASE_URL_WWW}/llms.txt" "text/plain.*charset=utf-8" "WWW llms.txt"

# Test llms.json
check_mime_type "${BASE_URL_WWW}/llms.json" "application/json.*charset=utf-8" "WWW llms.json"

# Test .well-known/arw-manifest.json
check_mime_type "${BASE_URL_WWW}/.well-known/arw-manifest.json" "application/json.*charset=utf-8" "WWW arw-manifest.json"

# Test robots.txt
check_robots_txt "${BASE_URL_WWW}" "WWW"

echo ""
echo "=== Testing Marketing App (${BASE_URL_MARKETING}) ==="
echo ""

# Test .llm.md files
check_mime_type "${BASE_URL_MARKETING}/blog/introducing-arw.llm.md" "text/markdown.*charset=utf-8" "Marketing .llm.md"

# Test llms.txt
check_mime_type "${BASE_URL_MARKETING}/llms.txt" "text/plain.*charset=utf-8" "Marketing llms.txt"

# Test llms.json
check_mime_type "${BASE_URL_MARKETING}/llms.json" "application/json.*charset=utf-8" "Marketing llms.json"

# Test .well-known/arw-manifest.json
check_mime_type "${BASE_URL_MARKETING}/.well-known/arw-manifest.json" "application/json.*charset=utf-8" "Marketing arw-manifest.json"

# Test robots.txt
check_robots_txt "${BASE_URL_MARKETING}" "Marketing"

echo ""
echo "================================================"
echo "Phase 1 Verification Complete"
echo "================================================"
echo ""
echo "To run this script:"
echo "  1. Start your development servers"
echo "  2. Run: bash scripts/verify-phase1.sh"
echo ""
echo "Or with custom URLs:"
echo "  BASE_URL_WWW=http://localhost:3000 BASE_URL_MARKETING=http://localhost:3001 bash scripts/verify-phase1.sh"
echo ""
