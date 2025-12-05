#!/bin/bash
# Advanced WASM optimization script

set -e

echo "⚡ Optimizing WASM binaries..."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if wasm-opt is installed
if ! command -v wasm-opt &> /dev/null; then
    echo -e "${RED}Error: wasm-opt is not installed${NC}"
    echo "Install binaryen: https://github.com/WebAssembly/binaryen"
    exit 1
fi

# Check if build exists
if [ ! -d "wasm-pkg" ]; then
    echo -e "${RED}Error: WASM build not found${NC}"
    echo "Run ./scripts/build-wasm.sh first"
    exit 1
fi

# Function to optimize a WASM file
optimize_wasm() {
    local file=$1
    local level=${2:-Oz}  # Default to Oz (aggressive size optimization)

    if [ ! -f "$file" ]; then
        echo -e "${YELLOW}Warning: $file not found, skipping${NC}"
        return
    fi

    local original_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")

    echo "  Optimizing $file with -$level..."
    wasm-opt -"$level" \
        --enable-bulk-memory \
        --enable-sign-ext \
        --enable-simd \
        --enable-nontrapping-float-to-int \
        "$file" \
        -o "$file.opt"

    mv "$file.opt" "$file"

    local optimized_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
    local reduction=$(( 100 - (optimized_size * 100 / original_size) ))

    echo "    Before: $(numfmt --to=iec-i --suffix=B $original_size 2>/dev/null || echo $original_size bytes)"
    echo "    After:  $(numfmt --to=iec-i --suffix=B $optimized_size 2>/dev/null || echo $optimized_size bytes)"
    echo "    Saved:  ${reduction}%"
}

echo ""
echo "🔧 Running aggressive optimizations..."
echo ""

# Optimize all WASM builds
for dir in wasm-pkg/*/; do
    if [ -f "$dir"arw_lib_bg.wasm ]; then
        target=$(basename "$dir")
        echo "📦 Optimizing $target build:"
        optimize_wasm "$dir"arw_lib_bg.wasm "Oz"
        echo ""
    fi
done

# Optional: Create gzipped versions
echo "📦 Creating gzipped versions..."
for dir in wasm-pkg/*/; do
    if [ -f "$dir"arw_lib_bg.wasm ]; then
        gzip -9 -k -f "$dir"arw_lib_bg.wasm
        gz_size=$(stat -f%z "$dir"arw_lib_bg.wasm.gz 2>/dev/null || stat -c%s "$dir"arw_lib_bg.wasm.gz)
        echo "  $(basename "$dir"): $(numfmt --to=iec-i --suffix=B $gz_size 2>/dev/null || echo $gz_size bytes) (gzipped)"
    fi
done

echo ""
echo -e "${GREEN}✓ Optimization complete!${NC}"
