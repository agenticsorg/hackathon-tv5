#!/bin/bash
# Build script for NAPI-RS bindings
# This script builds native Node.js bindings for the current platform

set -e

echo "🔨 Building ARW CLI with NAPI-RS..."

# Check if @napi-rs/cli is installed
if ! command -v napi &> /dev/null; then
    echo "📦 Installing @napi-rs/cli..."
    npm install -g @napi-rs/cli
fi

# Build for current platform
echo "🚀 Building for current platform..."
napi build --platform --release --features napi

echo "✅ Build complete!"
echo ""
echo "📋 Next steps:"
echo "  - Test the build: npm test"
echo "  - Run benchmarks: npm run bench"
echo "  - Build all platforms: see .github/workflows/napi-build.yml"
