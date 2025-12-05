#!/usr/bin/env bash
# Package binaries into archives for distribution
# Creates tar.gz for Unix-like systems and zip for Windows

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_DIR="$(dirname "$SCRIPT_DIR")"
NPM_DIR="$CLI_DIR/npm"
BINARIES_DIR="$NPM_DIR/binaries"
DIST_DIR="$CLI_DIR/dist"

echo "📦 Packaging ARW binaries for distribution..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Create dist directory
mkdir -p "$DIST_DIR"

# Get version from Cargo.toml
VERSION=$(grep '^version = ' "$CLI_DIR/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

echo "Version: $VERSION"
echo ""

# Package Unix binaries (tar.gz)
UNIX_BINARIES=(
    "linux-x64"
    "linux-arm64"
    "darwin-x64"
    "darwin-arm64"
)

for platform in "${UNIX_BINARIES[@]}"; do
    echo -e "${YELLOW}📦 Packaging $platform...${NC}"

    binary_file="arw-$platform"
    archive_name="arw-$platform.tar.gz"

    if [ -f "$BINARIES_DIR/$binary_file" ]; then
        # Create temporary directory for packaging
        temp_dir=$(mktemp -d)
        cp "$BINARIES_DIR/$binary_file" "$temp_dir/arw"
        chmod +x "$temp_dir/arw"

        # Copy README and LICENSE
        if [ -f "$CLI_DIR/README.md" ]; then
            cp "$CLI_DIR/README.md" "$temp_dir/"
        fi
        if [ -f "$CLI_DIR/../LICENSE" ]; then
            cp "$CLI_DIR/../LICENSE" "$temp_dir/" 2>/dev/null || true
        fi

        # Create archive
        (cd "$temp_dir" && tar -czf "$DIST_DIR/$archive_name" *)

        # Generate checksum
        (cd "$DIST_DIR" && shasum -a 256 "$archive_name" > "$archive_name.sha256")

        # Cleanup
        rm -rf "$temp_dir"

        echo -e "${GREEN}✓ Created $archive_name${NC}"
    else
        echo "⚠ Binary not found: $binary_file"
    fi
    echo ""
done

# Package Windows binary (zip)
echo -e "${YELLOW}📦 Packaging win32-x64...${NC}"

binary_file="arw-win32-x64.exe"
archive_name="arw-windows-x64.zip"

if [ -f "$BINARIES_DIR/$binary_file" ]; then
    # Create temporary directory for packaging
    temp_dir=$(mktemp -d)
    cp "$BINARIES_DIR/$binary_file" "$temp_dir/arw.exe"

    # Copy README and LICENSE
    if [ -f "$CLI_DIR/README.md" ]; then
        cp "$CLI_DIR/README.md" "$temp_dir/"
    fi
    if [ -f "$CLI_DIR/../LICENSE" ]; then
        cp "$CLI_DIR/../LICENSE" "$temp_dir/" 2>/dev/null || true
    fi

    # Create zip archive
    (cd "$temp_dir" && zip -r "$DIST_DIR/$archive_name" *)

    # Generate checksum
    (cd "$DIST_DIR" && shasum -a 256 "$archive_name" > "$archive_name.sha256")

    # Cleanup
    rm -rf "$temp_dir"

    echo -e "${GREEN}✓ Created $archive_name${NC}"
else
    echo "⚠ Binary not found: $binary_file"
fi

echo ""
echo -e "${GREEN}✅ All binaries packaged!${NC}"
echo ""
echo "📦 Distribution files in: $DIST_DIR"
ls -lh "$DIST_DIR"
