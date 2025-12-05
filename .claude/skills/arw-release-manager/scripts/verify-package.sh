#!/bin/bash
# Verify ARW CLI package is ready for publishing
# Checks both npm (TypeScript) and crates.io (Rust) requirements

set -e

echo "🔍 Verifying ARW CLI Package..."
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counters
ERRORS=0
WARNINGS=0

print_check() {
    echo -e "${BLUE}▶ Checking: ${1}${NC}"
}

print_pass() {
    echo -e "${GREEN}  ✓ ${1}${NC}"
}

print_fail() {
    echo -e "${RED}  ✗ ${1}${NC}"
    ERRORS=$((ERRORS + 1))
}

print_warn() {
    echo -e "${YELLOW}  ⚠ ${1}${NC}"
    WARNINGS=$((WARNINGS + 1))
}

# Get script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(dirname "$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")")"

cd "$ROOT_DIR"

echo "📂 Project root: $ROOT_DIR"
echo ""

# 1. Check git status
print_check "Git status"
if [ -d ".git" ]; then
    if [ -z "$(git status --porcelain)" ]; then
        print_pass "Working tree is clean"
    else
        print_warn "Uncommitted changes detected"
    fi

    BRANCH=$(git branch --show-current)
    if [ "$BRANCH" = "main" ] || [ "$BRANCH" = "master" ]; then
        print_pass "On branch: $BRANCH"
    else
        print_warn "On branch: $BRANCH (expected main/master for release)"
    fi
else
    print_warn "Not a git repository"
fi

# 2. Check package.json (npm)
print_check "package.json validity"
if [ -f "package.json" ]; then
    if node -e "require('./package.json')" 2>/dev/null; then
        print_pass "package.json is valid JSON"

        # Check version
        VERSION=$(node -p "require('./package.json').version" 2>/dev/null)
        if [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
            print_pass "Version: $VERSION (valid semver)"
        else
            print_fail "Version '$VERSION' is not valid semver"
        fi

        # Check required fields
        NAME=$(node -p "require('./package.json').name" 2>/dev/null)
        [ "$NAME" != "undefined" ] && print_pass "Name: $NAME" || print_fail "Name missing"

        DESC=$(node -p "require('./package.json').description" 2>/dev/null)
        [ "$DESC" != "undefined" ] && print_pass "Description exists" || print_warn "Description missing"

        LICENSE=$(node -p "require('./package.json').license" 2>/dev/null)
        [ "$LICENSE" != "undefined" ] && print_pass "License: $LICENSE" || print_warn "License missing"
    else
        print_fail "package.json has invalid JSON"
    fi
else
    print_warn "package.json not found (skip if Rust-only)"
fi

# 3. Check Cargo.toml (Rust)
print_check "Cargo.toml validity"
if [ -f "Cargo.toml" ]; then
    print_pass "Cargo.toml exists"

    # Check version
    CARGO_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
    if [[ $CARGO_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        print_pass "Cargo version: $CARGO_VERSION"
    else
        print_warn "Could not parse Cargo version"
    fi

    # Check required fields for crates.io
    grep -q '^description' Cargo.toml && print_pass "Description exists" || print_warn "Description missing (required for crates.io)"
    grep -q '^license' Cargo.toml && print_pass "License field exists" || print_warn "License missing (required for crates.io)"
    grep -q '^repository' Cargo.toml && print_pass "Repository field exists" || print_warn "Repository missing (recommended)"
else
    print_warn "Cargo.toml not found (skip if npm-only)"
fi

# 4. Check for secrets
print_check "Secrets scanning"
SECRETS_FOUND=false

if grep -r "sk-ant-" . --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git --exclude="*.md" -q 2>/dev/null; then
    print_fail "Found Anthropic API keys in code!"
    SECRETS_FOUND=true
fi

if grep -r "ANTHROPIC_API_KEY.*=.*sk-" . --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git -q 2>/dev/null; then
    print_fail "Found hardcoded API keys!"
    SECRETS_FOUND=true
fi

if grep -rE "(password|secret|token).*=.*['\"][^'\"]{10,}" . --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git --exclude="*.md" --exclude="*.lock" -q 2>/dev/null; then
    print_warn "Possible hardcoded credentials detected"
fi

if [ "$SECRETS_FOUND" = false ]; then
    print_pass "No obvious secrets found"
fi

# 5. Check built files
print_check "Built files"

# TypeScript dist
if [ -d "dist" ]; then
    print_pass "dist/ directory exists"
    if [ -f "dist/index.js" ] || [ -f "dist/cli.js" ]; then
        print_pass "Entry point exists"
    else
        print_warn "Entry point not found in dist/"
    fi
else
    print_warn "dist/ not found (run npm build)"
fi

# Rust target
if [ -d "target/release" ]; then
    print_pass "target/release/ exists"
    if [ -f "target/release/arw-cli" ] || [ -f "target/release/arw-cli.exe" ]; then
        print_pass "Release binary exists"
    else
        print_warn "Release binary not found"
    fi
else
    print_warn "target/release/ not found (run cargo build --release)"
fi

# 6. Check documentation
print_check "Documentation"
[ -f "README.md" ] && print_pass "README.md exists" || print_warn "README.md missing"
[ -f "LICENSE" ] && print_pass "LICENSE exists" || print_warn "LICENSE missing"
[ -f "CHANGELOG.md" ] && print_pass "CHANGELOG.md exists" || print_warn "CHANGELOG.md missing"

# 7. Version sync check
print_check "Version synchronization"
if [ -f "package.json" ] && [ -f "Cargo.toml" ]; then
    NPM_VER=$(node -p "require('./package.json').version" 2>/dev/null)
    CARGO_VER=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

    if [ "$NPM_VER" = "$CARGO_VER" ]; then
        print_pass "Versions match: $NPM_VER"
    else
        print_warn "Version mismatch: npm=$NPM_VER, cargo=$CARGO_VER"
    fi
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! Ready to publish.${NC}"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠️  ${WARNINGS} warnings found${NC}"
    echo "Package can be published but consider fixing warnings"
    exit 0
else
    echo -e "${RED}❌ ${ERRORS} errors found${NC}"
    [ $WARNINGS -gt 0 ] && echo -e "${YELLOW}   ${WARNINGS} warnings found${NC}"
    echo "Fix errors before publishing"
    exit 1
fi
