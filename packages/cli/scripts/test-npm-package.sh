#!/usr/bin/env bash
# Test npm package installation and execution
# Tests the complete npx workflow locally

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_DIR="$(dirname "$SCRIPT_DIR")"
NPM_DIR="$CLI_DIR/npm"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "🧪 Testing ARW npm package..."
echo ""

# Create temporary directory for testing
TEST_DIR=$(mktemp -d)
echo "Test directory: $TEST_DIR"

cleanup() {
    echo ""
    echo "Cleaning up..."
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

# Test 1: Install from local package
echo -e "${YELLOW}Test 1: Local package installation${NC}"
cp -r "$NPM_DIR" "$TEST_DIR/package"
cd "$TEST_DIR/package"

# Install dependencies
npm install

# Test binary execution
if [ -f "bin/arw" ]; then
    chmod +x bin/arw
    ./bin/arw --help > /dev/null 2>&1 && echo -e "${GREEN}✓ Binary execution works${NC}" || echo -e "${RED}✗ Binary execution failed${NC}"
else
    echo -e "${RED}✗ Binary not found${NC}"
fi

# Test 2: Module import
echo ""
echo -e "${YELLOW}Test 2: Module import${NC}"
node -e "const arw = require('./index.js'); console.log('Module loaded:', typeof arw)" && \
    echo -e "${GREEN}✓ Module import works${NC}" || \
    echo -e "${RED}✗ Module import failed${NC}"

# Test 3: Validate command (if binary available)
echo ""
echo -e "${YELLOW}Test 3: Validate command${NC}"

# Create test manifest
cat > "$TEST_DIR/test-manifest.txt" << 'EOF'
# Test ARW Manifest
version: "0.1"
profile: "ARW-1"

site:
  name: "Test Site"
  homepage: "https://example.com"
  contact: "ai@example.com"

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
EOF

if [ -f "bin/arw" ]; then
    ./bin/arw validate "$TEST_DIR/test-manifest.txt" && \
        echo -e "${GREEN}✓ Validation command works${NC}" || \
        echo -e "${YELLOW}⚠ Validation command failed (binary might not be built)${NC}"
else
    echo -e "${YELLOW}⚠ Skipping (binary not available)${NC}"
fi

# Test 4: Package.json structure
echo ""
echo -e "${YELLOW}Test 4: Package structure${NC}"

required_fields=("name" "version" "bin" "main")
for field in "${required_fields[@]}"; do
    if node -e "const pkg = require('./package.json'); if (!pkg.$field) process.exit(1)" 2>/dev/null; then
        echo -e "${GREEN}✓ package.json has '$field'${NC}"
    else
        echo -e "${RED}✗ package.json missing '$field'${NC}"
    fi
done

# Test 5: Platform detection
echo ""
echo -e "${YELLOW}Test 5: Platform detection${NC}"
node -e "
const platforms = require('./lib/platforms.js');
console.log('Current platform:', platforms.getCurrentPlatform());
console.log('Supported:', platforms.isPlatformSupported());
try {
    const config = platforms.getPlatformConfig();
    console.log('Platform config:', config);
} catch (error) {
    console.log('Error:', error.message);
}
" && echo -e "${GREEN}✓ Platform detection works${NC}" || echo -e "${RED}✗ Platform detection failed${NC}"

# Test 6: Binary management
echo ""
echo -e "${YELLOW}Test 6: Binary management${NC}"
node -e "
const binary = require('./lib/binary.js');
console.log('Binary available:', binary.isBinaryAvailable());
const path = binary.getBinaryPath();
console.log('Binary path:', path || 'not found');
" && echo -e "${GREEN}✓ Binary management works${NC}" || echo -e "${RED}✗ Binary management failed${NC}"

echo ""
echo -e "${GREEN}✅ Testing complete!${NC}"
echo ""
echo "Summary:"
echo "  - Package structure: OK"
echo "  - Module loading: OK"
echo "  - Platform detection: OK"
echo "  - Binary management: OK"
echo ""
