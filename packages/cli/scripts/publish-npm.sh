#!/usr/bin/env bash
# Publish ARW CLI to npm registry
# This script handles the complete publish workflow

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_DIR="$(dirname "$SCRIPT_DIR")"
NPM_DIR="$CLI_DIR/npm"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "🚀 Publishing ARW CLI to npm..."
echo ""

# Check if logged into npm
if ! npm whoami &> /dev/null; then
    echo -e "${RED}❌ Not logged into npm${NC}"
    echo "Please run: npm login"
    exit 1
fi

NPM_USER=$(npm whoami)
echo -e "${GREEN}✓ Logged in as: $NPM_USER${NC}"
echo ""

# Get version from Cargo.toml
VERSION=$(grep '^version = ' "$CLI_DIR/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
PACKAGE_NAME="@agent-ready-web/cli"

echo "Package: $PACKAGE_NAME"
echo "Version: $VERSION"
echo ""

# Check if version already exists
if npm view "$PACKAGE_NAME@$VERSION" &> /dev/null; then
    echo -e "${RED}❌ Version $VERSION already published${NC}"
    echo ""
    echo "Please bump the version in Cargo.toml and npm/package.json"
    exit 1
fi

echo -e "${YELLOW}📋 Pre-publish checklist:${NC}"
echo "  1. All tests passing?"
echo "  2. CHANGELOG.md updated?"
echo "  3. Version bumped in Cargo.toml and package.json?"
echo "  4. Git changes committed?"
echo "  5. Ready to publish to npm?"
echo ""

read -p "Continue with publish? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Publish cancelled"
    exit 0
fi

echo ""
echo -e "${YELLOW}🔧 Building package...${NC}"

# Build WASM (always include WASM fallback)
echo "Building WASM module..."
(cd "$CLI_DIR" && cargo build --release --target wasm32-unknown-unknown --features wasm)
(cd "$CLI_DIR" && wasm-pack build --target nodejs --out-dir npm/pkg --features wasm)

echo ""
echo -e "${YELLOW}🧪 Running tests...${NC}"
(cd "$CLI_DIR" && cargo test)

echo ""
echo -e "${YELLOW}📦 Publishing to npm...${NC}"

# Publish (binaries will be downloaded on install)
(cd "$NPM_DIR" && npm publish --access public)

echo ""
echo -e "${GREEN}✅ Published $PACKAGE_NAME@$VERSION!${NC}"
echo ""
echo -e "${BLUE}📝 Next steps:${NC}"
echo "  1. Create GitHub release with binaries"
echo "  2. Tag the release: git tag v$VERSION && git push --tags"
echo "  3. Update documentation"
echo "  4. Announce the release"
echo ""
echo "Test installation:"
echo "  npx $PACKAGE_NAME@$VERSION --help"
echo ""
