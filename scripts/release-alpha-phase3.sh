#!/bin/bash
set -e  # Exit on error

# Agent-Ready Web CLI - Alpha Release Script (Phase 3)
# Phase 3: Build Release Artifacts & Documentation

echo "================================"
echo "ARW CLI Alpha Release - Phase 3"
echo "Build Release Artifacts"
echo "================================"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

cd "$(dirname "$0")/../packages/cli" || exit 1

# Configuration
CURRENT_VERSION="0.2.0"
ALPHA_VERSION="0.3.0-alpha.1"

echo "📋 Version Information:"
echo "  Current: $CURRENT_VERSION"
echo "  Alpha: $ALPHA_VERSION"
echo ""

# Ask for confirmation
read -p "Proceed with version bump to $ALPHA_VERSION? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

# Step 3.1: Version Bump
echo "================================"
echo "Step 3.1: Version Bump"
echo "================================"
echo ""

echo "📝 Updating Cargo.toml..."
sed -i.bak "s/version = \"$CURRENT_VERSION\"/version = \"$ALPHA_VERSION\"/" Cargo.toml
rm Cargo.toml.bak
echo -e "${GREEN}✓ Cargo.toml updated${NC}"

echo "📝 Updating package.json..."
sed -i.bak "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$ALPHA_VERSION\"/" package.json
rm package.json.bak
echo -e "${GREEN}✓ package.json updated${NC}"

echo ""
echo "📊 Verifying version updates..."
grep "version = \"$ALPHA_VERSION\"" Cargo.toml && echo -e "${GREEN}  ✓ Cargo.toml verified${NC}"
grep "\"version\": \"$ALPHA_VERSION\"" package.json && echo -e "${GREEN}  ✓ package.json verified${NC}"
echo ""

# Step 3.2: Build for crates.io
echo "================================"
echo "Step 3.2: Build for crates.io"
echo "================================"
echo ""

echo "🧹 Cleaning previous builds..."
cargo clean
echo -e "${GREEN}✓ Cleaned${NC}"
echo ""

echo "🏗️  Building release binary..."
if cargo build --release; then
    echo -e "${GREEN}✓ Release build successful${NC}"
    echo "  Binary: ./target/release/arw"
    ls -lh ./target/release/arw
else
    echo -e "${RED}✗ Release build failed${NC}"
    exit 1
fi
echo ""

echo "🔍 Running clippy..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✓ Clippy passed (no warnings)${NC}"
else
    echo -e "${YELLOW}⚠ Clippy found warnings${NC}"
fi
echo ""

echo "📐 Checking code format..."
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✓ Code is formatted${NC}"
else
    echo -e "${YELLOW}⚠ Code needs formatting. Run: cargo fmt${NC}"
fi
echo ""

echo "📦 Packaging for crates.io (dry-run)..."
if cargo package --allow-dirty; then
    echo -e "${GREEN}✓ Package ready for crates.io${NC}"
else
    echo -e "${RED}✗ Packaging failed${NC}"
    exit 1
fi
echo ""

echo "📋 Package contents:"
cargo package --list | head -20
echo ""

# Step 3.3: Build for npm/npx
echo "================================"
echo "Step 3.3: Build for npm/npx"
echo "================================"
echo ""

echo "🔧 Installing dependencies..."
pnpm install
echo ""

echo "🏗️  Building NAPI bindings..."
if npm run build; then
    echo -e "${GREEN}✓ NAPI bindings built${NC}"
else
    echo -e "${RED}✗ NAPI build failed${NC}"
    exit 1
fi
echo ""

echo "🕸️  Building WASM for all targets..."
if npm run build:wasm; then
    echo -e "${GREEN}✓ WASM built for all targets${NC}"
    echo "  Targets:"
    echo "    - wasm-pkg/nodejs/"
    echo "    - wasm-pkg/web/"
    echo "    - wasm-pkg/bundler/"
else
    echo -e "${RED}✗ WASM build failed${NC}"
    exit 1
fi
echo ""

echo "📦 Creating npm package..."
if npm pack; then
    PACKAGE_FILE=$(ls -t agent-ready-web-cli-*.tgz | head -1)
    echo -e "${GREEN}✓ Package created: $PACKAGE_FILE${NC}"
    ls -lh "$PACKAGE_FILE"
else
    echo -e "${RED}✗ npm pack failed${NC}"
    exit 1
fi
echo ""

# Step 3.4: Documentation Check
echo "================================"
echo "Step 3.4: Documentation Check"
echo "================================"
echo ""

echo "📚 Checking documentation files..."
DOCS_OK=true

if [ -f "README-ALPHA.md" ]; then
    echo -e "${GREEN}  ✓ README-ALPHA.md exists${NC}"
else
    echo -e "${RED}  ✗ README-ALPHA.md missing${NC}"
    DOCS_OK=false
fi

if [ -f "CHANGELOG.md" ]; then
    echo -e "${GREEN}  ✓ CHANGELOG.md exists${NC}"
    # Check if alpha version is in changelog
    if grep -q "$ALPHA_VERSION" CHANGELOG.md; then
        echo -e "${GREEN}    ✓ Alpha version documented${NC}"
    else
        echo -e "${YELLOW}    ⚠ Alpha version not in CHANGELOG${NC}"
        DOCS_OK=false
    fi
else
    echo -e "${RED}  ✗ CHANGELOG.md missing${NC}"
    DOCS_OK=false
fi

if [ -f "README.md" ]; then
    echo -e "${GREEN}  ✓ README.md exists${NC}"
    echo "    Size: $(wc -l < README.md) lines"
else
    echo -e "${RED}  ✗ README.md missing${NC}"
    DOCS_OK=false
fi

if [ -f "CLI.md" ]; then
    echo -e "${GREEN}  ✓ CLI.md exists${NC}"
else
    echo -e "${YELLOW}  ⚠ CLI.md missing${NC}"
fi

echo ""

if [ "$DOCS_OK" = false ]; then
    echo -e "${YELLOW}⚠ Some documentation files are missing or incomplete${NC}"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Git Status
echo "================================"
echo "Git Status"
echo "================================"
echo ""

echo "📊 Current git status:"
git status --short
echo ""

echo "📝 Ready to commit version bump?"
echo "  Files changed:"
echo "    - Cargo.toml ($CURRENT_VERSION → $ALPHA_VERSION)"
echo "    - package.json ($CURRENT_VERSION → $ALPHA_VERSION)"
echo ""

read -p "Commit version bump? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git add Cargo.toml package.json
    git commit -m "chore: bump version to $ALPHA_VERSION"
    echo -e "${GREEN}✓ Version bump committed${NC}"
else
    echo -e "${YELLOW}⚠ Skipped git commit${NC}"
fi
echo ""

# Summary
echo "================================"
echo "Phase 3 Complete! ✅"
echo "================================"
echo ""
echo "Build Artifacts:"
echo "  ✓ Cargo package: target/package/arw-cli-$ALPHA_VERSION.crate"
echo "  ✓ NPM package: $PACKAGE_FILE"
echo "  ✓ Release binary: target/release/arw"
echo ""
echo "Documentation:"
if [ "$DOCS_OK" = true ]; then
    echo -e "  ${GREEN}✓ All documentation files present${NC}"
else
    echo -e "  ${YELLOW}⚠ Some documentation needs review${NC}"
fi
echo ""
echo "Version:"
echo "  ✓ Bumped to $ALPHA_VERSION"
echo ""
echo "Next steps:"
echo "  1. Review CHANGELOG.md and README-ALPHA.md"
echo "  2. Test package one more time:"
echo "     npx ./$PACKAGE_FILE --version"
echo "  3. Run Phase 4: ./scripts/release-alpha-phase4.sh"
echo "  4. Publish to crates.io and npm"
echo ""
