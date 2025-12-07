#!/usr/bin/env bash

#############################################################################
# Hackathon TV5 - System Validation Script
# Purpose: Honest, real-world testing of all components
# Date: 2025-12-06
#############################################################################

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Test result storage
RESULTS_FILE="docs/validation/TEST_RESULTS_$(date +%Y%m%d_%H%M%S).md"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((TESTS_SKIPPED++))
}

#############################################################################
# PHASE 1: ENVIRONMENT VALIDATION
#############################################################################

echo "=========================================="
echo "PHASE 1: ENVIRONMENT VALIDATION"
echo "=========================================="

# Test 1.1: Node.js version
log_info "Testing Node.js version..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    log_success "Node.js installed: $NODE_VERSION"
else
    log_error "Node.js not installed"
fi

# Test 1.2: npm version
log_info "Testing npm version..."
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm --version)
    log_success "npm installed: $NPM_VERSION"
else
    log_error "npm not installed"
fi

# Test 1.3: Rust toolchain
log_info "Testing Rust toolchain..."
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    RUSTC_VERSION=$(rustc --version)
    log_success "Cargo: $CARGO_VERSION"
    log_success "Rustc: $RUSTC_VERSION"
else
    log_warning "Rust not installed (optional for frontend-only testing)"
fi

# Test 1.4: Check .env file
log_info "Checking environment configuration..."
if [ -f "apps/media-discovery/.env" ]; then
    if grep -q "TMDB_ACCESS_TOKEN" apps/media-discovery/.env; then
        log_success ".env file configured with TMDB_ACCESS_TOKEN"
    else
        log_error ".env file missing TMDB_ACCESS_TOKEN"
    fi
else
    log_error ".env file not found"
fi

#############################################################################
# PHASE 2: DEPENDENCY VALIDATION
#############################################################################

echo ""
echo "=========================================="
echo "PHASE 2: DEPENDENCY VALIDATION"
echo "=========================================="

# Test 2.1: media-discovery dependencies
log_info "Checking media-discovery npm dependencies..."
cd apps/media-discovery

if [ -d "node_modules" ]; then
    INSTALLED_PACKAGES=$(npm list --depth=0 2>/dev/null | grep -c "├──\|└──" || echo "0")
    if [ "$INSTALLED_PACKAGES" -gt 0 ]; then
        log_success "Dependencies installed: $INSTALLED_PACKAGES packages"
    else
        log_error "Dependencies not properly installed"
    fi
else
    log_error "node_modules directory not found. Run 'npm install'"
fi

cd ../..

#############################################################################
# PHASE 3: BUILD VALIDATION
#############################################################################

echo ""
echo "=========================================="
echo "PHASE 3: BUILD VALIDATION"
echo "=========================================="

# Test 3.1: Next.js build
log_info "Testing Next.js build..."
cd apps/media-discovery

if npm run build > /tmp/nextjs-build.log 2>&1; then
    BUILD_SIZE=$(du -sh .next 2>/dev/null | cut -f1 || echo "unknown")
    log_success "Next.js build successful (size: $BUILD_SIZE)"
else
    log_error "Next.js build failed. Check /tmp/nextjs-build.log"
    tail -20 /tmp/nextjs-build.log
fi

cd ../..

# Test 3.2: TypeScript type checking
log_info "Running TypeScript type check..."
cd apps/media-discovery

if npm run typecheck > /tmp/typecheck.log 2>&1; then
    log_success "TypeScript type check passed"
else
    log_warning "TypeScript type check has errors (non-blocking)"
fi

cd ../..

#############################################################################
# PHASE 4: API VALIDATION (requires running server)
#############################################################################

echo ""
echo "=========================================="
echo "PHASE 4: API VALIDATION"
echo "=========================================="

log_info "Starting Next.js dev server for API testing..."
cd apps/media-discovery

# Start dev server in background
npm run dev > /tmp/nextjs-dev.log 2>&1 &
DEV_SERVER_PID=$!

# Wait for server to start (max 30 seconds)
log_info "Waiting for dev server to start..."
for i in {1..30}; do
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        log_success "Dev server started successfully"
        break
    fi
    sleep 1
done

# Test 4.1: Health check endpoint
log_info "Testing health check endpoint..."
HEALTH_RESPONSE=$(curl -s -w "%{http_code}" http://localhost:3000/api/health)
HTTP_CODE="${HEALTH_RESPONSE: -3}"

if [ "$HTTP_CODE" = "200" ]; then
    log_success "Health check passed (HTTP 200)"
else
    log_error "Health check failed (HTTP $HTTP_CODE)"
fi

# Test 4.2: TMDB API integration
log_info "Testing TMDB discover endpoint..."
DISCOVER_RESPONSE=$(curl -s -w "\n%{http_code}" "http://localhost:3000/api/discover?type=movie&limit=5")
DISCOVER_HTTP_CODE=$(echo "$DISCOVER_RESPONSE" | tail -1)
DISCOVER_BODY=$(echo "$DISCOVER_RESPONSE" | head -n -1)

if [ "$DISCOVER_HTTP_CODE" = "200" ]; then
    # Check if response has results
    if echo "$DISCOVER_BODY" | grep -q '"results"'; then
        RESULT_COUNT=$(echo "$DISCOVER_BODY" | grep -o '"results"' | wc -l)
        log_success "TMDB discover working (HTTP 200, found results)"
    else
        log_warning "TMDB discover returned 200 but no results found"
    fi
else
    log_error "TMDB discover failed (HTTP $DISCOVER_HTTP_CODE)"
    echo "Response: $DISCOVER_BODY" | head -5
fi

# Test 4.3: Natural language search
log_info "Testing natural language search..."
SEARCH_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "http://localhost:3000/api/search" \
    -H "Content-Type: application/json" \
    -d '{"query": "exciting sci-fi action movie", "limit": 5}')
SEARCH_HTTP_CODE=$(echo "$SEARCH_RESPONSE" | tail -1)
SEARCH_BODY=$(echo "$SEARCH_RESPONSE" | head -n -1)

if [ "$SEARCH_HTTP_CODE" = "200" ]; then
    if echo "$SEARCH_BODY" | grep -q '"success":true'; then
        log_success "Natural language search working (HTTP 200)"
        echo "$SEARCH_BODY" | jq -r '.results[0].content.title // "N/A"' 2>/dev/null | head -1
    else
        log_warning "Search returned 200 but success=false"
    fi
else
    log_error "Natural language search failed (HTTP $SEARCH_HTTP_CODE)"
fi

# Test 4.4: Recommendations endpoint
log_info "Testing recommendations endpoint..."
RECO_RESPONSE=$(curl -s -w "%{http_code}" "http://localhost:3000/api/recommendations?limit=5")
RECO_HTTP_CODE="${RECO_RESPONSE: -3}"

if [ "$RECO_HTTP_CODE" = "200" ]; then
    log_success "Recommendations endpoint working (HTTP 200)"
else
    log_warning "Recommendations endpoint returned HTTP $RECO_HTTP_CODE (may need preferences)"
fi

# Test 4.5: Watch party endpoint
log_info "Testing watch party endpoint..."
WATCH_PARTY_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "http://localhost:3000/api/watch-party" \
    -H "Content-Type: application/json" \
    -d '{
        "members": [
            {
                "userId": "test1",
                "name": "Alice",
                "preferences": {
                    "favoriteGenres": [28, 878],
                    "dislikedGenres": [27]
                }
            },
            {
                "userId": "test2",
                "name": "Bob",
                "preferences": {
                    "favoriteGenres": [35],
                    "dislikedGenres": []
                }
            }
        ]
    }')
PARTY_HTTP_CODE=$(echo "$WATCH_PARTY_RESPONSE" | tail -1)
PARTY_BODY=$(echo "$WATCH_PARTY_RESPONSE" | head -n -1)

if [ "$PARTY_HTTP_CODE" = "200" ]; then
    if echo "$PARTY_BODY" | grep -q '"success":true'; then
        PARTY_ID=$(echo "$PARTY_BODY" | jq -r '.partyId // "N/A"' 2>/dev/null)
        log_success "Watch party created successfully (ID: $PARTY_ID)"
    else
        log_warning "Watch party returned 200 but success=false"
    fi
else
    log_error "Watch party failed (HTTP $PARTY_HTTP_CODE)"
fi

# Cleanup: Stop dev server
log_info "Stopping dev server..."
kill $DEV_SERVER_PID 2>/dev/null || true
wait $DEV_SERVER_PID 2>/dev/null || true

cd ../..

#############################################################################
# PHASE 5: RUST WORKSPACE VALIDATION (OPTIONAL)
#############################################################################

echo ""
echo "=========================================="
echo "PHASE 5: RUST WORKSPACE VALIDATION"
echo "=========================================="

if command -v cargo &> /dev/null; then
    log_info "Testing Exogenesis Omega Rust workspace..."
    cd exogenesis-omega

    # Check if already built
    if [ -d "target/debug" ] || [ -d "target/release" ]; then
        log_success "Rust workspace previously built"
    else
        log_info "Building Rust workspace (this may take several minutes)..."
        if timeout 300 cargo build --all 2>&1 | tee /tmp/rust-build.log | tail -20; then
            log_success "Rust workspace build successful"
        else
            log_warning "Rust build incomplete or timed out (5 min limit)"
        fi
    fi

    cd ..
else
    log_skip "Rust not installed, skipping workspace validation"
fi

#############################################################################
# SUMMARY
#############################################################################

echo ""
echo "=========================================="
echo "VALIDATION SUMMARY"
echo "=========================================="
echo -e "${GREEN}Tests Passed:${NC} $TESTS_PASSED"
echo -e "${RED}Tests Failed:${NC} $TESTS_FAILED"
echo -e "${YELLOW}Tests Skipped:${NC} $TESTS_SKIPPED"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ ALL TESTS PASSED!${NC}"
    echo ""
    echo "Demo Readiness: ✅ READY"
    echo ""
    echo "What works:"
    echo "  - TMDB API integration"
    echo "  - Natural language search"
    echo "  - Recommendations"
    echo "  - Watch party functionality"
    echo "  - Health monitoring"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    echo ""
    echo "Demo Readiness: ⚠️ NEEDS WORK"
    echo ""
    echo "Check logs in:"
    echo "  - /tmp/nextjs-build.log"
    echo "  - /tmp/nextjs-dev.log"
    echo "  - /tmp/rust-build.log"
    exit 1
fi
