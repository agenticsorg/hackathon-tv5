#!/bin/bash
# Universal Agent Compatibility Test Suite
# Tests ARW implementation against all major AI platform requirements
# Based on docs/AGENT-COMPATIBILITY-TESTING.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
URL="${1:-http://localhost:3000}"
VERBOSE="${2:-false}"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNINGS=0

# Helper functions
log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    WARNINGS=$((WARNINGS + 1))
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Check if URL is accessible
check_url_accessible() {
    log_test "Checking if $URL is accessible"
    if curl -s -f -o /dev/null "$URL"; then
        log_pass "URL is accessible"
        return 0
    else
        log_fail "URL is not accessible"
        return 1
    fi
}

# Test 1: Discovery - /.well-known/arw-manifest.json
test_wellknown_discovery() {
    log_test "Testing /.well-known/arw-manifest.json discovery"

    # Separate status and headers from body
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$URL/.well-known/arw-manifest.json")
    HEADERS=$(curl -s -I "$URL/.well-known/arw-manifest.json")
    CONTENT_TYPE=$(echo "$HEADERS" | grep -i "content-type:" | cut -d: -f2- | tr -d '\r' | xargs)

    if [ "$STATUS" = "200" ]; then
        log_pass "Well-known endpoint returns 200 OK"

        if echo "$CONTENT_TYPE" | grep -q "application/json"; then
            log_pass "Content-Type is application/json"
        else
            log_fail "Content-Type is not application/json (got: $CONTENT_TYPE)"
        fi

        # Check if charset is specified
        if echo "$CONTENT_TYPE" | grep -q "charset=utf-8"; then
            log_pass "Charset UTF-8 is specified"
        else
            log_warn "Charset UTF-8 not specified in Content-Type"
        fi
    else
        log_fail "Well-known endpoint returned $STATUS instead of 200"
    fi
}

# Test 2: JSON Format Discovery
test_json_discovery() {
    log_test "Testing /llms.json discovery"

    # Get status, headers, and body separately
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$URL/llms.json")
    HEADERS=$(curl -s -I "$URL/llms.json")
    CONTENT_TYPE=$(echo "$HEADERS" | grep -i "content-type:" | cut -d: -f2- | tr -d '\r' | xargs)
    BODY=$(curl -s "$URL/llms.json")

    if [ "$STATUS" = "200" ]; then
        log_pass "/llms.json returns 200 OK"

        if echo "$CONTENT_TYPE" | grep -q "application/json"; then
            log_pass "Content-Type is application/json"
        else
            log_fail "Content-Type is not application/json (got: $CONTENT_TYPE)"
        fi

        # Validate JSON structure
        if echo "$BODY" | jq empty 2>/dev/null; then
            log_pass "Valid JSON structure"

            # Check required fields
            if echo "$BODY" | jq -e '.version' >/dev/null 2>&1; then
                log_pass "Version field present"
            else
                log_fail "Version field missing"
            fi

            if echo "$BODY" | jq -e '.profile' >/dev/null 2>&1; then
                log_pass "Profile field present"
            else
                log_fail "Profile field missing"
            fi

            if echo "$BODY" | jq -e '.site.name' >/dev/null 2>&1; then
                log_pass "Site name present"
            else
                log_fail "Site name missing"
            fi

            if echo "$BODY" | jq -e '.site.homepage' >/dev/null 2>&1; then
                log_pass "Site homepage present"
            else
                log_fail "Site homepage missing"
            fi
        else
            log_fail "Invalid JSON structure"
        fi
    else
        log_fail "/llms.json returned $STATUS instead of 200"
    fi
}

# Test 3: YAML Format Discovery
test_yaml_discovery() {
    log_test "Testing /llms.txt discovery (YAML)"

    # Get status and headers separately
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$URL/llms.txt")
    HEADERS=$(curl -s -I "$URL/llms.txt")
    CONTENT_TYPE=$(echo "$HEADERS" | grep -i "content-type:" | cut -d: -f2- | tr -d '\r' | xargs)

    if [ "$STATUS" = "200" ]; then
        log_pass "/llms.txt returns 200 OK"

        # Check for text/plain (ARW v0.2 requirement)
        if echo "$CONTENT_TYPE" | grep -q "text/plain"; then
            log_pass "Content-Type is text/plain (ARW v0.2 compliant)"
        else
            log_warn "Content-Type is not text/plain (got: $CONTENT_TYPE). ARW v0.2 recommends text/plain for universal compatibility"
        fi

        # Check for charset
        if echo "$CONTENT_TYPE" | grep -q "charset=utf-8"; then
            log_pass "Charset UTF-8 is specified"
        else
            log_warn "Charset UTF-8 not specified"
        fi
    else
        log_fail "/llms.txt returned $STATUS instead of 200"
    fi
}

# Test 4: Format Parity
test_format_parity() {
    log_test "Testing YAML/JSON format parity"

    JSON_BODY=$(curl -s "$URL/llms.json")
    YAML_BODY=$(curl -s "$URL/llms.txt")

    if [ -n "$JSON_BODY" ] && [ -n "$YAML_BODY" ]; then
        # Compare key fields
        JSON_VERSION=$(echo "$JSON_BODY" | jq -r '.version')
        JSON_PROFILE=$(echo "$JSON_BODY" | jq -r '.profile')
        JSON_SITE_NAME=$(echo "$JSON_BODY" | jq -r '.site.name')

        YAML_VERSION=$(echo "$YAML_BODY" | grep "^version:" | awk '{print $2}' | tr -d '"')
        YAML_PROFILE=$(echo "$YAML_BODY" | grep "^profile:" | awk '{print $2}' | tr -d '"')
        YAML_SITE_NAME=$(echo "$YAML_BODY" | grep "name:" | head -1 | awk '{print $2}' | tr -d '"')

        if [ "$JSON_VERSION" = "$YAML_VERSION" ]; then
            log_pass "Version field matches between YAML and JSON"
        else
            log_fail "Version mismatch: YAML=$YAML_VERSION, JSON=$JSON_VERSION"
        fi

        if [ "$JSON_PROFILE" = "$YAML_PROFILE" ]; then
            log_pass "Profile field matches between YAML and JSON"
        else
            log_fail "Profile mismatch: YAML=$YAML_PROFILE, JSON=$JSON_PROFILE"
        fi
    else
        log_fail "Could not retrieve both YAML and JSON for comparison"
    fi
}

# Test 5: AI-* Headers
test_ai_headers() {
    log_test "Testing AI-* headers on /llms.json"

    HEADERS=$(curl -s -I "$URL/llms.json")

    # Check for various AI-* headers
    if echo "$HEADERS" | grep -qi "ai-attribution"; then
        log_pass "AI-Attribution header present"
    else
        log_warn "AI-Attribution header not found (recommended for ARW-2+)"
    fi

    if echo "$HEADERS" | grep -qi "ai-inference"; then
        log_pass "AI-Inference header present"
    else
        log_warn "AI-Inference header not found (recommended for ARW-2+)"
    fi

    if echo "$HEADERS" | grep -qi "ai-training"; then
        log_pass "AI-Training header present"
    else
        log_warn "AI-Training header not found (recommended for ARW-2+)"
    fi
}

# Test 6: CORS Headers
test_cors_headers() {
    log_test "Testing CORS headers for agent access"

    HEADERS=$(curl -s -I -H "Origin: https://agent.example.com" "$URL/llms.json")

    if echo "$HEADERS" | grep -qi "access-control-allow-origin"; then
        log_pass "CORS headers present"
    else
        log_warn "CORS headers not found (may block some AI agents)"
    fi
}

# Test 7: Robots.txt Discovery Hints
test_robots_hints() {
    log_test "Testing robots.txt for ARW discovery hints"

    ROBOTS=$(curl -s "$URL/robots.txt")

    if [ -n "$ROBOTS" ]; then
        if echo "$ROBOTS" | grep -q "arw-manifest"; then
            log_pass "ARW discovery hints found in robots.txt"
        else
            log_info "No ARW hints in robots.txt (optional feature)"
        fi
    else
        log_info "No robots.txt found (optional)"
    fi
}

# Test 8: Machine View MIME Types
test_machine_view_mime() {
    log_test "Testing .llm.md MIME types"

    # Try to find a machine view URL from the manifest
    MACHINE_VIEW_URL=$(curl -s "$URL/llms.json" | jq -r '.content[0].machine_view // empty' 2>/dev/null)

    if [ -n "$MACHINE_VIEW_URL" ]; then
        FULL_URL="$URL$MACHINE_VIEW_URL"
        STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$FULL_URL")
        HEADERS=$(curl -s -I "$FULL_URL")
        CONTENT_TYPE=$(echo "$HEADERS" | grep -i "content-type:" | cut -d: -f2- | tr -d '\r' | xargs)

        if [ "$STATUS" = "200" ]; then
            log_pass "Machine view accessible"

            if echo "$CONTENT_TYPE" | grep -q "text/markdown"; then
                log_pass "Machine view uses text/markdown (recommended)"
            elif echo "$CONTENT_TYPE" | grep -q "text/plain"; then
                log_pass "Machine view uses text/plain (acceptable)"
            else
                log_warn "Machine view MIME type: $CONTENT_TYPE (ARW recommends text/markdown)"
            fi
        else
            log_warn "Machine view returned $STATUS"
        fi
    else
        log_info "No machine view found in manifest to test"
    fi
}

# Test 9: Binary Data Detection
test_no_binary_data() {
    log_test "Testing for binary data responses"

    # Test that responses are not binary
    JSON_RESPONSE=$(curl -s "$URL/llms.json" | head -c 200)

    # Check if response starts with valid JSON/YAML characters
    if echo "$JSON_RESPONSE" | grep -qE '^[{[]|^[a-zA-Z_#]'; then
        log_pass "No binary data in JSON response"
    else
        log_fail "Binary data detected in JSON response (first bytes: $(echo "$JSON_RESPONSE" | head -c 20))"
    fi

    YAML_RESPONSE=$(curl -s "$URL/llms.txt" | head -c 200)
    if echo "$YAML_RESPONSE" | grep -qE '^[{[]|^[a-zA-Z_#]'; then
        log_pass "No binary data in YAML response"
    else
        log_fail "Binary data detected in YAML response (first bytes: $(echo "$YAML_RESPONSE" | head -c 20))"
    fi
}

# Test 10: Claude Compatibility (critical test)
test_claude_compatibility() {
    log_test "Testing Claude WebFetch compatibility"

    # Claude requires text/plain for YAML and application/json for JSON
    YAML_CT=$(curl -s -I "$URL/llms.txt" | grep -i "content-type:" | cut -d: -f2- | tr -d '\r' | xargs)
    JSON_CT=$(curl -s -I "$URL/llms.json" | grep -i "content-type:" | cut -d: -f2- | tr -d '\r' | xargs)

    if echo "$YAML_CT" | grep -q "text/plain"; then
        log_pass "YAML uses text/plain (Claude compatible)"
    else
        log_fail "YAML does not use text/plain. Claude WebFetch may return binary data. Got: $YAML_CT"
    fi

    if echo "$JSON_CT" | grep -q "application/json"; then
        log_pass "JSON uses application/json (Claude compatible)"
    else
        log_fail "JSON does not use application/json (got: $JSON_CT)"
    fi
}

# Main test execution
main() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  ARW Universal Agent Compatibility Test Suite             ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "Testing URL: ${BLUE}$URL${NC}"
    echo ""

    # Check dependencies
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}Error: jq is required but not installed.${NC}"
        echo "Install with: brew install jq (macOS) or apt-get install jq (Linux)"
        exit 1
    fi

    # Run all tests
    check_url_accessible || exit 1
    echo ""

    echo -e "${BLUE}═══ Discovery Tests ═══${NC}"
    test_wellknown_discovery
    test_json_discovery
    test_yaml_discovery
    echo ""

    echo -e "${BLUE}═══ Format & Parity Tests ═══${NC}"
    test_format_parity
    test_no_binary_data
    echo ""

    echo -e "${BLUE}═══ Header Tests ═══${NC}"
    test_ai_headers
    test_cors_headers
    echo ""

    echo -e "${BLUE}═══ Optional Features ═══${NC}"
    test_robots_hints
    test_machine_view_mime
    echo ""

    echo -e "${BLUE}═══ Platform-Specific Tests ═══${NC}"
    test_claude_compatibility
    echo ""

    # Summary
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Test Summary                                              ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "Total Tests:    ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed:         ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed:         ${RED}$FAILED_TESTS${NC}"
    echo -e "Warnings:       ${YELLOW}$WARNINGS${NC}"
    echo ""

    # Calculate percentage
    if [ $TOTAL_TESTS -gt 0 ]; then
        PASS_PERCENT=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo -e "Pass Rate:      ${BLUE}$PASS_PERCENT%${NC}"
        echo ""
    fi

    # Overall result
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}✅ All critical tests passed!${NC}"
        echo -e "${GREEN}Your ARW implementation has universal agent compatibility.${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed.${NC}"
        echo -e "${YELLOW}Review the failures above and check docs/AGENT-COMPATIBILITY-TESTING.md${NC}"
        exit 1
    fi
}

# Run main function
main
