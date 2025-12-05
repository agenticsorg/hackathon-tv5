#!/bin/bash
# Build all ARW CLI packages (napi-rs, standalone CLI, WASM)

set -e

echo "🏗️  Building ARW CLI Packages..."
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}▶ ${1}${NC}"
}

print_success() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

print_error() {
    echo -e "${RED}✗ ${1}${NC}"
}

print_warn() {
    echo -e "${YELLOW}⚠ ${1}${NC}"
}

# Get CLI package directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Navigate from .claude/skills/arw-release-manager/scripts to packages/cli
CLI_DIR="$(dirname "$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")")/packages/cli"

# If CLI_DIR doesn't exist, try from current directory
if [ ! -d "$CLI_DIR" ]; then
    if [ -f "Cargo.toml" ] && [ -f "package.json" ]; then
        CLI_DIR="$(pwd)"
    else
        print_error "Cannot find packages/cli directory. Run from repo root or packages/cli."
        exit 1
    fi
fi

cd "$CLI_DIR"

echo "📂 Building from: $CLI_DIR"
echo ""

# Parse arguments
BUILD_NAPI=true
BUILD_CLI=true
BUILD_WASM=false
RELEASE=false
DEV_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --napi-only)
            BUILD_CLI=false
            BUILD_WASM=false
            shift
            ;;
        --cli-only)
            BUILD_NAPI=false
            BUILD_WASM=false
            shift
            ;;
        --wasm-only)
            BUILD_NAPI=false
            BUILD_CLI=false
            BUILD_WASM=true
            shift
            ;;
        --with-wasm)
            BUILD_WASM=true
            shift
            ;;
        --release)
            RELEASE=true
            shift
            ;;
        --dev)
            DEV_MODE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Build Targets:"
            echo "  --napi-only   Build napi-rs native addon only"
            echo "  --cli-only    Build standalone CLI binary only"
            echo "  --wasm-only   Build WASM only"
            echo "  --with-wasm   Include WASM in build (default: excluded)"
            echo ""
            echo "Build Modes:"
            echo "  --dev         Development build (fast, with debug info)"
            echo "  --release     Production release build (optimized)"
            echo ""
            echo "Other:"
            echo "  -h, --help    Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                    # Build napi + CLI (dev)"
            echo "  $0 --release          # Build napi + CLI (release)"
            echo "  $0 --cli-only         # Build standalone CLI only"
            echo "  $0 --release --with-wasm  # Build all targets (release)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--dev] [--release] [--napi-only] [--cli-only] [--wasm-only] [--with-wasm]"
            exit 1
            ;;
    esac
done

# 1. Build napi-rs native addon
if [ "$BUILD_NAPI" = true ] && [ -f "package.json" ]; then
    print_status "Building napi-rs native addon..."

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing npm dependencies..."
        npm install
    fi

    # Build
    if [ "$DEV_MODE" = true ]; then
        if npm run build:debug 2>/dev/null; then
            print_success "napi-rs debug build complete"
        else
            print_error "napi-rs debug build failed"
            exit 1
        fi
    else
        if npm run build 2>/dev/null; then
            print_success "napi-rs release build complete"
        else
            print_error "napi-rs build failed"
            exit 1
        fi
    fi

    # Verify output
    NODE_FILE=$(ls -1 *.node 2>/dev/null | head -1)
    if [ -n "$NODE_FILE" ]; then
        SIZE=$(du -h "$NODE_FILE" | cut -f1)
        print_success "Native addon: $NODE_FILE ($SIZE)"
    fi
    echo ""
fi

# 2. Build standalone CLI binary
if [ "$BUILD_CLI" = true ] && [ -f "Cargo.toml" ]; then
    if [ "$RELEASE" = true ] || [ "$DEV_MODE" = false ]; then
        print_status "Building standalone CLI (release)..."

        if cargo build --release --features native; then
            print_success "Standalone CLI release build complete"

            # Show binary info
            if [ -f "target/release/arw" ]; then
                SIZE=$(du -h target/release/arw | cut -f1)
                print_success "Binary: target/release/arw ($SIZE)"
            fi
        else
            print_error "Standalone CLI release build failed"
            exit 1
        fi
    else
        print_status "Building standalone CLI (debug)..."

        if cargo build --features native; then
            print_success "Standalone CLI debug build complete"

            # Show binary location
            if [ -f "target/debug/arw" ]; then
                print_success "Binary: target/debug/arw"
            fi
        else
            print_error "Standalone CLI debug build failed"
            exit 1
        fi
    fi
    echo ""
fi

# 3. Build WASM (optional)
if [ "$BUILD_WASM" = true ] && [ -f "package.json" ]; then
    print_status "Building WASM packages..."

    if npm run build:wasm 2>/dev/null; then
        print_success "WASM (nodejs) build complete"
    else
        print_warn "WASM nodejs build failed or not available"
    fi

    if npm run build:wasm:web 2>/dev/null; then
        print_success "WASM (web) build complete"
    else
        print_warn "WASM web build failed or not available"
    fi

    if npm run build:wasm:bundler 2>/dev/null; then
        print_success "WASM (bundler) build complete"
    else
        print_warn "WASM bundler build failed or not available"
    fi
    echo ""
fi

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Build complete!${NC}"
echo ""

if [ "$BUILD_NAPI" = true ]; then
    NODE_FILE=$(ls -1 *.node 2>/dev/null | head -1)
    if [ -n "$NODE_FILE" ]; then
        echo "📦 napi-rs addon: $NODE_FILE"
        echo "   Use: const cli = require('./index.js')"
    fi
fi

if [ "$BUILD_CLI" = true ] && [ -d "target" ]; then
    if [ "$RELEASE" = true ] || [ "$DEV_MODE" = false ]; then
        if [ -f "target/release/arw" ]; then
            echo "📦 Standalone CLI: target/release/arw"
            echo "   Use: ./target/release/arw --help"
        fi
    else
        if [ -f "target/debug/arw" ]; then
            echo "📦 Standalone CLI: target/debug/arw"
            echo "   Use: ./target/debug/arw --help"
        fi
    fi
fi

if [ "$BUILD_WASM" = true ]; then
    echo "📦 WASM packages: wasm-pkg/"
fi

echo ""
echo "🚀 Next steps:"
echo "   ./scripts/verify-package.sh  # Verify package"
echo "   ./scripts/quick-publish.sh   # Publish"
