#!/bin/bash
# Watch for file changes and run tests automatically

set -e

echo "👀 Watching for file changes..."
echo "==============================="
echo ""

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

# Watch and run tests on changes
cargo watch \
    -x "test --lib" \
    -x "test --test '*'" \
    --clear \
    --delay 1
