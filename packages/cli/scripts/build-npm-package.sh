#!/usr/bin/env bash
# Build NPM package with native binaries for all platforms
# This script uses cross-compilation to build binaries for multiple targets

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_DIR="$(dirname "$SCRIPT_DIR")"
NPM_DIR="$CLI_DIR/npm"
BINARIES_DIR="$NPM_DIR/binaries"

echo "🔧 Building ARW CLI for multiple platforms..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo -e "${YELLOW}⚠ 'cross' not found. Installing...${NC}"
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Create binaries directory
mkdir -p "$BINARIES_DIR"

# Build targets
TARGETS=(
    "x86_64-unknown-linux-gnu:linux-x64"
    "aarch64-unknown-linux-gnu:linux-arm64"
    "x86_64-apple-darwin:darwin-x64"
    "aarch64-apple-darwin:darwin-arm64"
    "x86_64-pc-windows-gnu:win32-x64"
)

echo ""
echo "📦 Building for ${#TARGETS[@]} platforms..."
echo ""

for target_mapping in "${TARGETS[@]}"; do
    IFS=':' read -r target platform <<< "$target_mapping"

    echo -e "${YELLOW}🔨 Building for $platform ($target)...${NC}"

    # Use cross for non-native targets, cargo for native
    if [[ "$OSTYPE" == "darwin"* ]] && [[ "$target" == "x86_64-apple-darwin" || "$target" == "aarch64-apple-darwin" ]]; then
        # Native macOS build
        (cd "$CLI_DIR" && cargo build --release --target "$target")
    elif [[ "$OSTYPE" == "linux-gnu"* ]] && [[ "$target" == "x86_64-unknown-linux-gnu" ]]; then
        # Native Linux build
        (cd "$CLI_DIR" && cargo build --release --target "$target")
    else
        # Cross-compile
        (cd "$CLI_DIR" && cross build --release --target "$target")
    fi

    # Copy binary to binaries directory with platform-specific name
    if [[ "$platform" == win32-* ]]; then
        binary_name="arw.exe"
        output_name="arw-$platform.exe"
    else
        binary_name="arw"
        output_name="arw-$platform"
    fi

    cp "$CLI_DIR/target/$target/release/$binary_name" "$BINARIES_DIR/$output_name"

    echo -e "${GREEN}✓ Built $platform${NC}"
    echo ""
done

echo -e "${GREEN}✅ All platforms built successfully!${NC}"
echo ""
echo "📦 Binaries available in: $BINARIES_DIR"
ls -lh "$BINARIES_DIR"
