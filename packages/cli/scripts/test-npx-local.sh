#!/bin/bash
# Test NPX package locally before publishing

set -e

echo "🧪 Testing NPX package locally..."
echo ""

cd "$(dirname "$0")/.."

# Check if WASM is built
if [ ! -d "npm/pkg" ]; then
    echo "📦 Building WASM module..."
    npm run build:wasm
fi

echo "✅ WASM module found"
echo ""

# Create npm link
echo "🔗 Creating npm link..."
cd npm
npm link

echo ""
echo "✅ Package linked successfully!"
echo ""
echo "Try these commands in another terminal:"
echo ""
echo "  npx @agent-ready-web/cli --help"
echo "  npx @agent-ready-web/cli --version"
echo ""
echo "Or test the JavaScript API:"
echo ""
echo "  node -e \"const arw = require('@agent-ready-web/cli'); arw.getVersionInfo()\""
echo ""
echo "To unlink:"
echo "  npm unlink -g @agent-ready-web/cli"
