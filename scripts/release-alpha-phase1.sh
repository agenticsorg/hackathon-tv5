#!/bin/bash
set -e  # Exit on error

# Agent-Ready Web CLI - Alpha Release Script (Phase 1)
# Phase 1: Local Testing & Build

echo "================================"
echo "ARW CLI Alpha Release - Phase 1"
echo "Local Testing & Build"
echo "================================"
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Ensure rustup's Rust is used instead of Homebrew's
export PATH="$HOME/.cargo/bin:$PATH"

# Navigate to CLI package
echo "📁 Navigating to CLI package..."
cd "$(dirname "$0")/../packages/cli" || exit 1
echo "✓ Current directory: $(pwd)"
echo "✓ Using Rust from: $(which rustc)"
echo "✓ Rust version: $(rustc --version)"
echo ""

# Step 1.1: Run Complete Test Suite
echo "================================"
echo "Step 1.1: Running Test Suite"
echo "================================"
echo ""

echo "🧪 Running Rust tests (150+ tests)..."
if cargo test --all-features --verbose; then
    echo -e "${GREEN}✓ All Rust tests passed${NC}"
else
    echo -e "${RED}✗ Rust tests failed${NC}"
    exit 1
fi
echo ""

echo "🌐 Running WASM tests..."
if npm run test:wasm; then
    echo -e "${GREEN}✓ WASM tests passed${NC}"
else
    echo -e "${RED}✗ WASM tests failed${NC}"
    exit 1
fi
echo ""

echo "📊 Generating test coverage report..."
if [ -f "./scripts/test-coverage.sh" ]; then
    ./scripts/test-coverage.sh
    echo -e "${GREEN}✓ Coverage report generated${NC}"
else
    echo -e "${YELLOW}⚠ Coverage script not found, skipping${NC}"
fi
echo ""

# Step 1.2: Build NAPI Bindings
echo "================================"
echo "Step 1.2: Building NAPI Bindings"
echo "================================"
echo ""

echo "🔧 Installing dependencies..."
pnpm install
echo -e "${GREEN}✓ Dependencies installed${NC}"
echo ""

echo "🏗️  Building NAPI bindings for current platform..."
if npm run build; then
    echo -e "${GREEN}✓ NAPI bindings built${NC}"
else
    echo -e "${RED}✗ NAPI build failed${NC}"
    exit 1
fi
echo ""

echo "📦 Verifying build artifacts..."
if ls -la *.node 2>/dev/null; then
    echo -e "${GREEN}✓ NAPI bindings verified${NC}"
else
    echo -e "${YELLOW}⚠ No .node files found (may be platform-specific)${NC}"
fi
echo ""

# Step 1.3: Build WASM Package
echo "================================"
echo "Step 1.3: Building WASM Package"
echo "================================"
echo ""

echo "🕸️  Building WASM for all targets..."
if npm run build:wasm; then
    echo -e "${GREEN}✓ WASM built for all targets${NC}"
else
    echo -e "${RED}✗ WASM build failed${NC}"
    exit 1
fi
echo ""

echo "📦 Verifying WASM artifacts..."
if [ -d "wasm-pkg/nodejs" ] && [ -d "wasm-pkg/web" ] && [ -d "wasm-pkg/bundler" ]; then
    echo -e "${GREEN}✓ All WASM targets verified${NC}"
    echo "  - wasm-pkg/nodejs/"
    echo "  - wasm-pkg/web/"
    echo "  - wasm-pkg/bundler/"
else
    echo -e "${YELLOW}⚠ Some WASM targets missing${NC}"
fi
echo ""

# Step 1.4: Create Local NPM Package
echo "================================"
echo "Step 1.4: Creating NPM Package"
echo "================================"
echo ""

echo "📦 Packing npm package..."
if npm pack; then
    PACKAGE_FILE=$(ls -t agent-ready-web-cli-*.tgz | head -1)
    echo -e "${GREEN}✓ Package created: $PACKAGE_FILE${NC}"
else
    echo -e "${RED}✗ npm pack failed${NC}"
    exit 1
fi
echo ""

echo "✅ Testing package with npx..."
if npx "./$PACKAGE_FILE" --version; then
    echo -e "${GREEN}✓ Package works with npx${NC}"
else
    echo -e "${RED}✗ npx test failed${NC}"
    exit 1
fi
echo ""

# Summary
echo "================================"
echo "Phase 1 Complete! ✅"
echo "================================"
echo ""
echo "Summary:"
echo "  ✓ All tests passed (150+ tests)"
echo "  ✓ NAPI bindings built"
echo "  ✓ WASM built for all targets"
echo "  ✓ NPM package created: $PACKAGE_FILE"
echo ""
echo "Next steps:"
echo "  1. Run Phase 2: ./scripts/release-alpha-phase2.sh"
echo "  2. Test against example apps"
echo ""
echo "Package ready for local testing:"
echo "  npx ./$PACKAGE_FILE validate"
echo ""
