#!/bin/bash
# Comprehensive test script for ARW CLI

set -e  # Exit on any error

echo "🧪 ARW CLI Test Suite"
echo "===================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Track test results
FAILED=0

run_test_group() {
    local name=$1
    local command=$2

    echo -e "${YELLOW}Running $name...${NC}"
    if eval "$command"; then
        echo -e "${GREEN}✓ $name passed${NC}"
        echo ""
    else
        echo -e "${RED}✗ $name failed${NC}"
        FAILED=1
        echo ""
    fi
}

# 1. Build the project
echo "📦 Building project..."
cargo build
echo ""

# 2. Run unit tests
run_test_group "Unit Tests" "cargo test --lib"

# 3. Run integration tests
run_test_group "Integration Tests" "cargo test --test '*'"

# 4. Run E2E workflow tests
run_test_group "E2E Workflow Tests" "cargo test --test 'e2e::*'"

# 5. Run scenario tests
run_test_group "Scenario Tests" "cargo test --test 'scenarios::*'"

# 6. Run CLI tests
run_test_group "CLI Integration Tests" "cargo test --test 'cli::*'"

# 7. Run performance tests
run_test_group "Performance Tests" "cargo test --test 'performance::*' -- --nocapture"

# 8. Run regression tests
run_test_group "Regression Tests" "cargo test --test 'regression::*'"

# 9. Run linter
run_test_group "Clippy Lints" "cargo clippy -- -D warnings"

# 10. Check formatting
run_test_group "Code Formatting" "cargo fmt -- --check"

# Summary
echo "===================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
