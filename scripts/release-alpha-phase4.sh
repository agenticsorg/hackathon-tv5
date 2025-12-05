#!/bin/bash
set -e  # Exit on error

# Agent-Ready Web CLI - Alpha Release Script (Phase 4)
# Phase 4: Publish to crates.io and npm

echo "================================"
echo "ARW CLI Alpha Release - Phase 4"
echo "Publish to Registries"
echo "================================"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

cd "$(dirname "$0")/../packages/cli" || exit 1

ALPHA_VERSION="0.3.0-alpha.1"

# Pre-flight checks
echo "🔍 Pre-flight Checks"
echo "================================"
echo ""

echo "1️⃣  Checking version in files..."
if grep -q "version = \"$ALPHA_VERSION\"" Cargo.toml; then
    echo -e "${GREEN}  ✓ Cargo.toml: $ALPHA_VERSION${NC}"
else
    echo -e "${RED}  ✗ Cargo.toml version mismatch${NC}"
    exit 1
fi

if grep -q "\"version\": \"$ALPHA_VERSION\"" package.json; then
    echo -e "${GREEN}  ✓ package.json: $ALPHA_VERSION${NC}"
else
    echo -e "${RED}  ✗ package.json version mismatch${NC}"
    exit 1
fi
echo ""

echo "2️⃣  Checking package files..."
PACKAGE_FILE=$(ls -t agent-ready-web-cli-*.tgz 2>/dev/null | head -1)
if [ -n "$PACKAGE_FILE" ]; then
    echo -e "${GREEN}  ✓ NPM package found: $PACKAGE_FILE${NC}"
else
    echo -e "${RED}  ✗ NPM package not found. Run Phase 3 first.${NC}"
    exit 1
fi

if [ -f "target/package/arw-cli-$ALPHA_VERSION.crate" ]; then
    echo -e "${GREEN}  ✓ Cargo package found${NC}"
else
    echo -e "${YELLOW}  ⚠ Cargo package not found (will be created during publish)${NC}"
fi
echo ""

echo "3️⃣  Checking git status..."
if git diff --quiet; then
    echo -e "${GREEN}  ✓ Working directory clean${NC}"
else
    echo -e "${YELLOW}  ⚠ Uncommitted changes detected${NC}"
    git status --short
    echo ""
    read -p "  Continue anyway? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi
echo ""

echo "4️⃣  Checking credentials..."
echo "  Checking cargo login..."
if cargo login --help > /dev/null 2>&1; then
    echo -e "${GREEN}  ✓ Cargo CLI available${NC}"
else
    echo -e "${RED}  ✗ Cargo not available${NC}"
    exit 1
fi

echo "  Checking npm login..."
if npm whoami > /dev/null 2>&1; then
    NPM_USER=$(npm whoami)
    echo -e "${GREEN}  ✓ Logged into npm as: $NPM_USER${NC}"
else
    echo -e "${RED}  ✗ Not logged into npm${NC}"
    echo "    Run: npm login"
    exit 1
fi
echo ""

# Final confirmation
echo "================================"
echo "⚠️  FINAL CONFIRMATION"
echo "================================"
echo ""
echo "About to publish:"
echo "  Package: @agent-ready-web/cli"
echo "  Version: $ALPHA_VERSION"
echo "  Tag: alpha (NOT latest)"
echo ""
echo "Registries:"
echo "  - crates.io (Rust/Cargo)"
echo "  - npmjs.com (Node.js/NPX)"
echo ""
echo "This action CANNOT be easily undone!"
echo ""

read -p "Proceed with publishing? (y/n) " -n 1 -r
echo ""
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Step 4.1: Publish to crates.io
echo "================================"
echo "Step 4.1: Publish to crates.io"
echo "================================"
echo ""

echo "🦀 Publishing to crates.io..."
echo ""
echo "DRY RUN first..."
if cargo publish --dry-run; then
    echo -e "${GREEN}✓ Dry run successful${NC}"
else
    echo -e "${RED}✗ Dry run failed${NC}"
    exit 1
fi
echo ""

read -p "Proceed with actual publish to crates.io? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if cargo publish; then
        echo -e "${GREEN}✓ Published to crates.io!${NC}"
        echo ""
        echo "  View at: https://crates.io/crates/arw-cli"
        CRATES_PUBLISHED=true
    else
        echo -e "${RED}✗ Publish to crates.io failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ Skipped crates.io publish${NC}"
    CRATES_PUBLISHED=false
fi
echo ""

# Step 4.2: Publish to npm
echo "================================"
echo "Step 4.2: Publish to npm"
echo "================================"
echo ""

echo "📦 Publishing to npm with 'alpha' tag..."
echo "  Command: npm publish --tag alpha --access public"
echo ""

read -p "Proceed with npm publish? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if npm publish --tag alpha --access public; then
        echo -e "${GREEN}✓ Published to npm!${NC}"
        echo ""
        echo "  View at: https://www.npmjs.com/package/@agent-ready-web/cli"
        NPM_PUBLISHED=true
    else
        echo -e "${RED}✗ Publish to npm failed${NC}"

        # Rollback crates.io if npm failed
        if [ "$CRATES_PUBLISHED" = true ]; then
            echo ""
            echo "⚠️  npm publish failed but crates.io succeeded."
            echo "    Consider yanking the crates.io version:"
            echo "    cargo yank --vers $ALPHA_VERSION"
        fi
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ Skipped npm publish${NC}"
    NPM_PUBLISHED=false
fi
echo ""

# Step 4.3: Verify Publications
echo "================================"
echo "Step 4.3: Verify Publications"
echo "================================"
echo ""

if [ "$NPM_PUBLISHED" = true ]; then
    echo "📋 Verifying npm publication..."

    echo "  Checking dist-tags..."
    npm dist-tag ls @agent-ready-web/cli
    echo ""

    echo "  Testing installation with npx..."
    if npx @agent-ready-web/cli@alpha --version; then
        echo -e "${GREEN}  ✓ npx installation works!${NC}"
    else
        echo -e "${RED}  ✗ npx installation failed${NC}"
    fi
    echo ""
fi

if [ "$CRATES_PUBLISHED" = true ]; then
    echo "🦀 Verifying crates.io publication..."
    echo "  Note: May take a few minutes to appear on crates.io"
    echo ""
    echo "  To verify:"
    echo "    cargo search arw-cli"
    echo "    cargo install arw-cli --version $ALPHA_VERSION"
    echo ""
fi

# Step 4.4: Create Git Tag
echo "================================"
echo "Step 4.4: Create Git Tag"
echo "================================"
echo ""

echo "🏷️  Creating git tag: v$ALPHA_VERSION"
echo ""

read -p "Create and push git tag? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Create tag
    if git tag -a "v$ALPHA_VERSION" -m "Alpha release $ALPHA_VERSION"; then
        echo -e "${GREEN}  ✓ Tag created${NC}"
    else
        echo -e "${YELLOW}  ⚠ Tag may already exist${NC}"
    fi

    # Push tag
    if git push origin "v$ALPHA_VERSION"; then
        echo -e "${GREEN}  ✓ Tag pushed${NC}"
    else
        echo -e "${RED}  ✗ Failed to push tag${NC}"
    fi

    # Check if gh CLI is available
    if command -v gh &> /dev/null; then
        echo ""
        echo "📦 Creating GitHub release..."
        read -p "Create GitHub release? (y/n) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if [ -f "CHANGELOG.md" ]; then
                gh release create "v$ALPHA_VERSION" \
                    --title "v$ALPHA_VERSION - Alpha Release" \
                    --notes-file CHANGELOG.md \
                    --prerelease

                # Upload package
                gh release upload "v$ALPHA_VERSION" "$PACKAGE_FILE"

                echo -e "${GREEN}  ✓ GitHub release created${NC}"
            else
                echo -e "${YELLOW}  ⚠ CHANGELOG.md not found${NC}"
            fi
        fi
    else
        echo -e "${YELLOW}  ⚠ gh CLI not installed, skipping GitHub release${NC}"
        echo "    Install: https://cli.github.com/"
    fi
else
    echo -e "${YELLOW}⚠ Skipped git tag${NC}"
fi
echo ""

# Summary
echo "================================"
echo "🎉 Phase 4 Complete!"
echo "================================"
echo ""

echo "Publication Status:"
if [ "$CRATES_PUBLISHED" = true ]; then
    echo -e "${GREEN}  ✓ crates.io: Published${NC}"
    echo "    https://crates.io/crates/arw-cli"
    echo "    Install: cargo install arw-cli --version $ALPHA_VERSION"
else
    echo -e "${YELLOW}  ⚠ crates.io: Skipped${NC}"
fi

if [ "$NPM_PUBLISHED" = true ]; then
    echo -e "${GREEN}  ✓ npm: Published${NC}"
    echo "    https://www.npmjs.com/package/@agent-ready-web/cli"
    echo "    Install: npm install -g @agent-ready-web/cli@alpha"
    echo "    Use: npx @agent-ready-web/cli@alpha"
else
    echo -e "${YELLOW}  ⚠ npm: Skipped${NC}"
fi
echo ""

echo "Next Steps:"
echo "  1. Test installations:"
echo "     cargo install arw-cli --version $ALPHA_VERSION"
echo "     npx @agent-ready-web/cli@alpha --version"
echo ""
echo "  2. Announce alpha release:"
echo "     - GitHub Discussions"
echo "     - Community channels"
echo "     - Social media"
echo ""
echo "  3. Monitor for issues:"
echo "     - GitHub Issues"
echo "     - User feedback"
echo "     - Bug reports"
echo ""
echo "  4. Plan beta release based on feedback"
echo ""

# Rollback instructions
echo "================================"
echo "Rollback Instructions (if needed)"
echo "================================"
echo ""
echo "If critical issues are found:"
echo ""
echo "NPM:"
echo "  # Deprecate (within 72 hours)"
echo "  npm deprecate @agent-ready-web/cli@$ALPHA_VERSION \"Critical bug - use stable\""
echo ""
echo "  # Or unpublish (within 72 hours)"
echo "  npm unpublish @agent-ready-web/cli@$ALPHA_VERSION"
echo ""
echo "Crates.io:"
echo "  # Yank version (keeps accessible but discourages use)"
echo "  cargo yank --vers $ALPHA_VERSION"
echo ""
echo "  # Un-yank if fixed"
echo "  cargo yank --vers $ALPHA_VERSION --undo"
echo ""

echo "🚀 Alpha release complete! Happy testing! 🎉"
echo ""
