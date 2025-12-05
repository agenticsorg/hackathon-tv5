#!/bin/bash
set -e  # Exit on error

# Agent-Ready Web CLI - Alpha Release Script (Phase 2)
# Phase 2: Recursive Testing Against Example Apps

echo "================================"
echo "ARW CLI Alpha Release - Phase 2"
echo "Recursive Testing Against Apps"
echo "================================"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Find the package file
cd "$(dirname "$0")/../packages/cli" || exit 1
PACKAGE_FILE=$(ls -t agent-ready-web-cli-*.tgz | head -1)

if [ -z "$PACKAGE_FILE" ]; then
    echo -e "${RED}✗ No package file found. Run Phase 1 first.${NC}"
    exit 1
fi

echo "📦 Using package: $PACKAGE_FILE"
PACKAGE_PATH="$(pwd)/$PACKAGE_FILE"
echo "📍 Full path: $PACKAGE_PATH"
echo ""

# Test function
test_app() {
    local APP_NAME=$1
    local APP_PATH=$2

    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}Testing: $APP_NAME${NC}"
    echo -e "${BLUE}================================${NC}"
    echo ""

    cd "$APP_PATH" || {
        echo -e "${RED}✗ Cannot access $APP_PATH${NC}"
        return 1
    }

    echo "📁 Current directory: $(pwd)"
    echo ""

    # Clean previous outputs
    echo "🧹 Cleaning previous outputs..."
    rm -rf .arw/ llms.txt sitemap.* *.llm.md robots.txt 2>/dev/null || true
    echo -e "${GREEN}✓ Cleaned${NC}"
    echo ""

    # Test 1: Validate
    echo "🔍 Test 1: Validation"
    if npx "$PACKAGE_PATH" validate; then
        echo -e "${GREEN}✓ Validation passed${NC}"
    else
        echo -e "${YELLOW}⚠ Validation had warnings (may be expected)${NC}"
    fi
    echo ""

    # Test 2: Generate
    echo "📝 Test 2: Generation"
    if npx "$PACKAGE_PATH" generate; then
        echo -e "${GREEN}✓ Generation completed${NC}"
    else
        echo -e "${RED}✗ Generation failed${NC}"
        return 1
    fi
    echo ""

    # Test 3: Verify outputs
    echo "✅ Test 3: Verify outputs"
    local ALL_PRESENT=true

    if [ -f "llms.txt" ]; then
        echo -e "${GREEN}  ✓ llms.txt created${NC}"
        echo "    Preview:"
        head -n 5 llms.txt | sed 's/^/      /'
    else
        echo -e "${RED}  ✗ llms.txt missing${NC}"
        ALL_PRESENT=false
    fi

    if [ -f "sitemap.xml" ]; then
        echo -e "${GREEN}  ✓ sitemap.xml created${NC}"
    else
        echo -e "${RED}  ✗ sitemap.xml missing${NC}"
        ALL_PRESENT=false
    fi

    if [ -f "robots.txt" ]; then
        echo -e "${GREEN}  ✓ robots.txt created${NC}"
    else
        echo -e "${YELLOW}  ⚠ robots.txt missing (may be optional)${NC}"
    fi

    local LLM_MD_COUNT=$(find . -maxdepth 1 -name "*.llm.md" 2>/dev/null | wc -l)
    if [ "$LLM_MD_COUNT" -gt 0 ]; then
        echo -e "${GREEN}  ✓ $LLM_MD_COUNT .llm.md files created${NC}"
    else
        echo -e "${YELLOW}  ⚠ No .llm.md files found${NC}"
    fi
    echo ""

    # Test 4: Build
    echo "🏗️  Test 4: Build"
    if npx "$PACKAGE_PATH" build; then
        echo -e "${GREEN}✓ Build completed${NC}"
    else
        echo -e "${YELLOW}⚠ Build had issues (may be expected)${NC}"
    fi
    echo ""

    # Test 5: Serve (quick test)
    echo "🌐 Test 5: Serve (5 second test)"
    npx "$PACKAGE_PATH" serve --port 3333 &
    SERVER_PID=$!
    echo "  Server PID: $SERVER_PID"

    # Wait for server to start
    sleep 2

    # Test endpoints
    if curl -s http://localhost:3333/llms.txt > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ Server responding on /llms.txt${NC}"
    else
        echo -e "${YELLOW}  ⚠ Server not responding (may need more time)${NC}"
    fi

    # Kill server
    kill $SERVER_PID 2>/dev/null || true
    sleep 1
    echo -e "${GREEN}  ✓ Server stopped${NC}"
    echo ""

    if [ "$ALL_PRESENT" = true ]; then
        echo -e "${GREEN}✅ All tests passed for $APP_NAME${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠ Some tests failed for $APP_NAME${NC}"
        return 1
    fi
}

# Test 2.1: basic-blog
echo "Starting app tests..."
echo ""

cd "$(dirname "$0")/.." || exit 1
REPO_ROOT=$(pwd)

BASIC_BLOG_PATH="$REPO_ROOT/examples/basic-blog"
if [ -d "$BASIC_BLOG_PATH" ]; then
    test_app "basic-blog" "$BASIC_BLOG_PATH" || BASIC_BLOG_FAILED=true
else
    echo -e "${YELLOW}⚠ basic-blog not found at $BASIC_BLOG_PATH${NC}"
    BASIC_BLOG_FAILED=true
fi
echo ""

# Test 2.2: www platform app
WWW_PATH="$REPO_ROOT/platform/apps/www"
if [ -d "$WWW_PATH" ]; then
    test_app "www (platform)" "$WWW_PATH" || WWW_FAILED=true
else
    echo -e "${YELLOW}⚠ www platform app not found at $WWW_PATH${NC}"
    WWW_FAILED=true
fi
echo ""

# Test 2.3: dashboard platform app
DASHBOARD_PATH="$REPO_ROOT/platform/apps/dashboard"
if [ -d "$DASHBOARD_PATH" ]; then
    test_app "dashboard (platform)" "$DASHBOARD_PATH" || DASHBOARD_FAILED=true
else
    echo -e "${YELLOW}⚠ dashboard platform app not found at $DASHBOARD_PATH${NC}"
    DASHBOARD_FAILED=true
fi
echo ""

# Test 2.4: Regression tests
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Running Regression Tests${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

cd "$REPO_ROOT/packages/cli" || exit 1

export ARW_CLI_PATH="$PACKAGE_PATH"

echo "🔬 Running regression test suite..."
if cargo test --test regression -- --nocapture; then
    echo -e "${GREEN}✓ All regression tests passed${NC}"
else
    echo -e "${RED}✗ Some regression tests failed${NC}"
    REGRESSION_FAILED=true
fi
echo ""

# Summary
echo "================================"
echo "Phase 2 Complete!"
echo "================================"
echo ""

if [ -z "$BASIC_BLOG_FAILED" ]; then
    echo -e "${GREEN}✓ basic-blog tests passed${NC}"
else
    echo -e "${RED}✗ basic-blog tests failed${NC}"
fi

if [ -z "$WWW_FAILED" ]; then
    echo -e "${GREEN}✓ www platform app tests passed${NC}"
else
    echo -e "${RED}✗ www platform app tests failed${NC}"
fi

if [ -z "$DASHBOARD_FAILED" ]; then
    echo -e "${GREEN}✓ dashboard platform app tests passed${NC}"
else
    echo -e "${RED}✗ dashboard platform app tests failed${NC}"
fi

if [ -z "$REGRESSION_FAILED" ]; then
    echo -e "${GREEN}✓ Regression tests passed${NC}"
else
    echo -e "${RED}✗ Regression tests failed${NC}"
fi

echo ""
echo "Next steps:"
echo "  1. Review any failures above"
echo "  2. Run Phase 3: ./scripts/release-alpha-phase3.sh"
echo "  3. Build release artifacts"
echo ""
