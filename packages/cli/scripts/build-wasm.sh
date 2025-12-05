#!/bin/bash
# Build script for WASM compilation

set -e

echo "🦀 Building ARW CLI for WebAssembly..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}Error: wasm-pack is not installed${NC}"
    echo "Install with: cargo install wasm-pack"
    exit 1
fi

# Check if wasm-opt is installed (optional but recommended)
if ! command -v wasm-opt &> /dev/null; then
    echo -e "${YELLOW}Warning: wasm-opt is not installed (optional)${NC}"
    echo "Install binaryen for better optimization: https://github.com/WebAssembly/binaryen"
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg wasm-pkg

# Build for Node.js
echo "📦 Building for Node.js..."
wasm-pack build \
    --target nodejs \
    --out-dir wasm-pkg/nodejs \
    --release \
    --features wasm \
    -- --profile wasm-release

# Build for browser
echo "🌐 Building for browser..."
wasm-pack build \
    --target web \
    --out-dir wasm-pkg/web \
    --release \
    --features wasm \
    -- --profile wasm-release

# Build for bundlers (webpack, rollup, etc.)
echo "📦 Building for bundlers..."
wasm-pack build \
    --target bundler \
    --out-dir wasm-pkg/bundler \
    --release \
    --features wasm \
    -- --profile wasm-release

# Optimize WASM binaries if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "⚡ Optimizing WASM binaries..."

    for dir in wasm-pkg/*/; do
        if [ -f "$dir"arw_lib_bg.wasm ]; then
            echo "  Optimizing $dir"arw_lib_bg.wasm
            wasm-opt -Oz --enable-bulk-memory \
                "$dir"arw_lib_bg.wasm \
                -o "$dir"arw_lib_bg.wasm.opt
            mv "$dir"arw_lib_bg.wasm.opt "$dir"arw_lib_bg.wasm
        fi
    done
fi

# Display sizes
echo ""
echo "📊 Build sizes:"
for dir in wasm-pkg/*/; do
    if [ -f "$dir"arw_lib_bg.wasm ]; then
        size=$(du -h "$dir"arw_lib_bg.wasm | cut -f1)
        target=$(basename "$dir")
        echo "  $target: $size"
    fi
done

echo ""
echo -e "${GREEN}✓ WASM build complete!${NC}"
echo ""
echo "Output directories:"
echo "  - wasm-pkg/nodejs/   (Node.js target)"
echo "  - wasm-pkg/web/      (Browser target)"
echo "  - wasm-pkg/bundler/  (Webpack/Rollup target)"
