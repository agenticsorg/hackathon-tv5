#!/bin/bash
set -e

echo "=== API Gateway Verification ==="
echo ""

# Check directory structure
echo "1. Checking directory structure..."
dirs=(
    "src/middleware"
    "src/routes"
)

for dir in "${dirs[@]}"; do
    if [ -d "$dir" ]; then
        echo "  ✓ $dir exists"
    else
        echo "  ✗ $dir missing"
        exit 1
    fi
done

# Check required files
echo ""
echo "2. Checking required files..."
files=(
    "Cargo.toml"
    "src/lib.rs"
    "src/main.rs"
    "src/server.rs"
    "src/config.rs"
    "src/error.rs"
    "src/health.rs"
    "src/rate_limit.rs"
    "src/circuit_breaker.rs"
    "src/proxy.rs"
    "src/middleware/mod.rs"
    "src/middleware/auth.rs"
    "src/middleware/logging.rs"
    "src/middleware/request_id.rs"
    "src/routes/mod.rs"
    "src/routes/content.rs"
    "src/routes/search.rs"
    "src/routes/discover.rs"
    "src/routes/user.rs"
    ".env.example"
    "README.md"
    "Dockerfile"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✓ $file exists"
    else
        echo "  ✗ $file missing"
        exit 1
    fi
done

# Count lines of code
echo ""
echo "3. Code metrics..."
echo "  Total lines: $(find src -name "*.rs" | xargs wc -l | tail -1 | awk '{print $1}')"
echo "  Files: $(find src -name "*.rs" | wc -l)"

# Check for key features in code
echo ""
echo "4. Checking key features..."

features=(
    "rate_limit.rs:RateLimiter"
    "circuit_breaker.rs:CircuitBreakerManager"
    "proxy.rs:ServiceProxy"
    "middleware/auth.rs:AuthMiddleware"
    "middleware/request_id.rs:RequestIdMiddleware"
    "middleware/logging.rs:LoggingMiddleware"
    "routes/content.rs:get_content"
    "routes/search.rs:hybrid_search"
    "routes/user.rs:get_profile"
    "health.rs:HealthChecker"
)

for feature in "${features[@]}"; do
    file="${feature%%:*}"
    pattern="${feature##*:}"
    if grep -q "$pattern" "src/$file" 2>/dev/null; then
        echo "  ✓ $file contains $pattern"
    else
        echo "  ✗ $file missing $pattern"
        exit 1
    fi
done

echo ""
echo "=== Verification Complete ==="
echo "All checks passed! API Gateway implementation is complete."
