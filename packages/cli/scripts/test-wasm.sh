#!/bin/bash
# Test script for WASM bindings in Node.js

set -e

echo "🧪 Testing WASM bindings..."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Check if build exists
if [ ! -d "wasm-pkg/nodejs" ]; then
    echo -e "${RED}Error: WASM build not found${NC}"
    echo "Run ./scripts/build-wasm.sh first"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed${NC}"
    exit 1
fi

# Run Node.js tests
echo "📦 Running Node.js tests..."
cd wasm-tests
npm test

echo ""
echo -e "${GREEN}✓ All WASM tests passed!${NC}"
