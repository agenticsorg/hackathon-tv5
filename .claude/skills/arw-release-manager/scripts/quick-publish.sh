#!/bin/bash
# Quick publish script for ARW CLI
# Runs all checks, builds, and publishes to npm and/or crates.io

set -e

echo "🚀 ARW CLI - Quick Publish Script"
echo "=================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(dirname "$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")")"

cd "$ROOT_DIR"

# Parse arguments
DRY_RUN=false
SKIP_TESTS=false
SKIP_BUILD=false
NPM_ONLY=false
CARGO_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --npm-only)
            NPM_ONLY=true
            shift
            ;;
        --cargo-only)
            CARGO_ONLY=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --dry-run      Preview without publishing"
            echo "  --skip-tests   Skip test suite"
            echo "  --skip-build   Skip build step"
            echo "  --npm-only     Only publish to npm"
            echo "  --cargo-only   Only publish to crates.io"
            echo "  -h, --help     Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# 1. Verify package
echo -e "${BLUE}▶ Step 1: Verifying package...${NC}"
if [ -f "$SCRIPT_DIR/verify-package.sh" ]; then
    bash "$SCRIPT_DIR/verify-package.sh" || {
        echo -e "${RED}❌ Verification failed. Fix errors before publishing.${NC}"
        exit 1
    }
else
    echo -e "${YELLOW}⚠️  verify-package.sh not found, skipping verification${NC}"
fi
echo ""

# 2. Run linter (TypeScript)
if [ "$CARGO_ONLY" = false ] && [ -f "package.json" ]; then
    echo -e "${BLUE}▶ Step 2: Running linter...${NC}"
    if npm run lint 2>/dev/null; then
        echo -e "${GREEN}✓ Linting passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Linting not configured or failed${NC}"
    fi
    echo ""
fi

# 3. Run type checking (TypeScript)
if [ "$CARGO_ONLY" = false ] && [ -f "package.json" ]; then
    echo -e "${BLUE}▶ Step 3: Running TypeScript type checking...${NC}"
    if npm run typecheck 2>/dev/null; then
        echo -e "${GREEN}✓ Type checking passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Type checking not configured or failed${NC}"
    fi
    echo ""
fi

# 4. Run Rust checks
if [ "$NPM_ONLY" = false ] && [ -f "Cargo.toml" ]; then
    echo -e "${BLUE}▶ Step 4: Running Rust checks...${NC}"

    if cargo fmt --check 2>/dev/null; then
        echo -e "${GREEN}✓ Rust formatting OK${NC}"
    else
        echo -e "${YELLOW}⚠️  Run 'cargo fmt' to fix formatting${NC}"
    fi

    if cargo clippy -- -D warnings 2>/dev/null; then
        echo -e "${GREEN}✓ Clippy passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Clippy warnings found${NC}"
    fi
    echo ""
fi

# 5. Build packages
if [ "$SKIP_BUILD" = false ]; then
    echo -e "${BLUE}▶ Step 5: Building packages...${NC}"
    if [ -f "$SCRIPT_DIR/build-all.sh" ]; then
        bash "$SCRIPT_DIR/build-all.sh" --release
    else
        [ -f "package.json" ] && npm run build
        [ -f "Cargo.toml" ] && cargo build --release
    fi
    echo ""
else
    echo -e "${YELLOW}⚠️  Skipping build step${NC}"
    echo ""
fi

# 6. Run tests
if [ "$SKIP_TESTS" = false ]; then
    echo -e "${BLUE}▶ Step 6: Running tests...${NC}"

    if [ "$CARGO_ONLY" = false ] && [ -f "package.json" ]; then
        if npm test 2>/dev/null; then
            echo -e "${GREEN}✓ npm tests passed${NC}"
        else
            echo -e "${YELLOW}⚠️  npm tests not configured or failed${NC}"
        fi
    fi

    if [ "$NPM_ONLY" = false ] && [ -f "Cargo.toml" ]; then
        if cargo test 2>/dev/null; then
            echo -e "${GREEN}✓ Cargo tests passed${NC}"
        else
            echo -e "${YELLOW}⚠️  Cargo tests not configured or failed${NC}"
        fi
    fi
    echo ""
else
    echo -e "${YELLOW}⚠️  Skipping tests${NC}"
    echo ""
fi

# 7. Preview package contents
echo -e "${BLUE}▶ Step 7: Package preview...${NC}"
if [ "$CARGO_ONLY" = false ] && [ -f "package.json" ]; then
    echo "npm package:"
    npm pack --dry-run 2>/dev/null || true
fi
echo ""

# 8. Dry run mode
if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}📦 Dry run mode - not publishing${NC}"
    echo ""
    echo "To actually publish, run without --dry-run:"
    echo "  $0"
    exit 0
fi

# 9. Get version and confirm
if [ -f "package.json" ]; then
    VERSION=$(node -p "require('./package.json').version" 2>/dev/null)
elif [ -f "Cargo.toml" ]; then
    VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
else
    VERSION="unknown"
fi

echo -e "${YELLOW}❓ Ready to publish version ${VERSION}?${NC}"
read -p "   Continue? (yes/no) " -r
echo

if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    echo "❌ Publish cancelled"
    exit 1
fi

# 10. Publish to npm
if [ "$CARGO_ONLY" = false ] && [ -f "package.json" ]; then
    echo -e "${BLUE}▶ Publishing to npm...${NC}"
    npm publish --access public
    echo -e "${GREEN}✓ Published to npm${NC}"
    echo ""
fi

# 11. Publish to crates.io
if [ "$NPM_ONLY" = false ] && [ -f "Cargo.toml" ]; then
    echo -e "${BLUE}▶ Publishing to crates.io...${NC}"
    cargo publish
    echo -e "${GREEN}✓ Published to crates.io${NC}"
    echo ""
fi

# Success!
echo ""
echo -e "${GREEN}✅ Successfully published arw-cli@${VERSION}!${NC}"
echo ""
echo "📝 Next steps:"
echo ""
echo "   1. Create GitHub release:"
echo "      git tag v${VERSION}"
echo "      git push origin v${VERSION}"
echo "      gh release create v${VERSION} --generate-notes"
echo ""

if [ "$CARGO_ONLY" = false ] && [ -f "package.json" ]; then
    PKG_NAME=$(node -p "require('./package.json').name" 2>/dev/null)
    echo "   2. Verify on npmjs.com:"
    echo "      https://www.npmjs.com/package/${PKG_NAME}"
    echo ""
fi

if [ "$NPM_ONLY" = false ] && [ -f "Cargo.toml" ]; then
    CRATE_NAME=$(grep -m1 '^name' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
    echo "   3. Verify on crates.io:"
    echo "      https://crates.io/crates/${CRATE_NAME}"
    echo ""
fi

echo "   4. Test installation:"
[ "$CARGO_ONLY" = false ] && echo "      npm install -g arw-cli@${VERSION}"
[ "$NPM_ONLY" = false ] && echo "      cargo install arw-cli@${VERSION}"
echo "      arw-cli --version"
