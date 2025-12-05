# Advanced ARW CLI Release Configuration

This document covers advanced release scenarios including CI/CD integration, multi-platform builds, and automated workflows.

## GitHub Actions CI/CD

### Automated Testing on Push

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-typescript:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm run lint
      - run: npm run typecheck
      - run: npm test

  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

### Automated Release on Tag

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  publish-npm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - run: npm ci
      - run: npm run build
      - run: npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  publish-crates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  create-github-release:
    needs: [publish-npm, publish-crates]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: softprops/action-gh-release@v1
        with:
          generate_release_notes: true
```

## Multi-Platform Binary Releases

### Cross-Compilation Matrix

Create `.github/workflows/binaries.yml`:

```yaml
name: Build Binaries

on:
  release:
    types: [created]

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: arw-cli-linux-x64
          - target: x86_64-apple-darwin
            os: macos-latest
            name: arw-cli-macos-x64
          - target: aarch64-apple-darwin
            os: macos-latest
            name: arw-cli-macos-arm64
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            name: arw-cli-windows-x64.exe

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: target/${{ matrix.target }}/release/arw-cli${{ matrix.os == 'windows-latest' && '.exe' || '' }}
```

### Local Cross-Compilation

```bash
# Install cross-compilation tools
cargo install cross

# Build for multiple targets
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
cross build --release --target aarch64-unknown-linux-gnu

# Or use cargo directly with targets installed
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

## Automated Changelog Generation

### Using git-cliff

Install: `cargo install git-cliff`

Create `cliff.toml`:

```toml
[changelog]
header = """
# Changelog\n
All notable changes to this project will be documented in this file.\n
"""
body = """
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
- {{ commit.message | upper_first }} ({{ commit.id | truncate(length=7, end="") }})\
{% endfor %}
{% endfor %}
"""
footer = """
"""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
commit_parsers = [
  { message = "^feat", group = "Added" },
  { message = "^fix", group = "Fixed" },
  { message = "^doc", group = "Documentation" },
  { message = "^perf", group = "Performance" },
  { message = "^refactor", group = "Changed" },
  { message = "^style", group = "Styling" },
  { message = "^test", group = "Testing" },
  { message = "^chore", skip = true },
]
```

Generate changelog:

```bash
# Generate for unreleased changes
git cliff --unreleased --prepend CHANGELOG.md

# Generate for a specific version
git cliff --tag v1.2.0 --prepend CHANGELOG.md
```

### Using Conventional Commits

Follow [Conventional Commits](https://www.conventionalcommits.org/) spec:

```bash
# Features
git commit -m "feat: add new export command"
git commit -m "feat(cli): add --verbose flag"

# Bug fixes
git commit -m "fix: resolve crash on empty input"
git commit -m "fix(parser): handle unicode correctly"

# Breaking changes
git commit -m "feat!: change default output format"
git commit -m "feat(api)!: remove deprecated endpoint"

# With scope and body
git commit -m "feat(auth): add OAuth2 support

Implements RFC 6749 OAuth 2.0 authorization framework.
Closes #123"
```

## GPG Signing Releases

### Setup GPG Key

```bash
# Generate key
gpg --full-generate-key

# List keys
gpg --list-secret-keys --keyid-format=long

# Export public key (for GitHub)
gpg --armor --export YOUR_KEY_ID
```

### Configure Git

```bash
# Set signing key
git config --global user.signingkey YOUR_KEY_ID

# Sign all commits
git config --global commit.gpgsign true

# Sign all tags
git config --global tag.gpgsign true
```

### Signed Tags

```bash
# Create signed tag
git tag -s v1.0.0 -m "Release v1.0.0"

# Verify tag signature
git tag -v v1.0.0
```

## Pre-Release Channels

### npm Tags

```bash
# Publish beta
npm version 1.0.0-beta.1
npm publish --tag beta

# Publish release candidate
npm version 1.0.0-rc.1
npm publish --tag next

# Promote to latest
npm dist-tag add arw-cli@1.0.0 latest

# Users install with:
npm install arw-cli@beta
npm install arw-cli@next
```

### Cargo Features

In `Cargo.toml`:

```toml
[features]
default = []
unstable = ["experimental-feature"]
experimental-feature = []
```

Users enable with:

```bash
cargo install arw-cli --features unstable
```

## Monorepo Releases

### Workspace Configuration

```toml
# Root Cargo.toml
[workspace]
members = [
    "arw-cli",
    "arw-core",
    "arw-parser",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
```

### Synchronized Versioning

```bash
# Update all workspace packages
cargo set-version 1.1.0 --workspace

# Or use cargo-workspaces
cargo install cargo-workspaces
cargo ws version patch
```

### npm Workspaces

```json
{
  "workspaces": ["packages/*"],
  "scripts": {
    "publish-all": "npm publish --workspaces --access public"
  }
}
```

## Rollback Procedures

### npm

```bash
# Deprecate a version (warns on install)
npm deprecate arw-cli@1.0.0 "Critical bug, use 1.0.1"

# Unpublish (within 72 hours only)
npm unpublish arw-cli@1.0.0

# Point latest tag to previous version
npm dist-tag add arw-cli@0.9.0 latest
```

### crates.io

```bash
# Yank a version (prevents new installations)
cargo yank --vers 1.0.0

# Unyank if needed
cargo yank --vers 1.0.0 --undo
```

### Git

```bash
# Delete remote tag
git push origin :refs/tags/v1.0.0

# Delete local tag
git tag -d v1.0.0

# Revert release commit
git revert HEAD
git push origin main
```

## Release Automation Scripts

### Bump Version Script

```bash
#!/bin/bash
# scripts/bump-version.sh

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

# Update package.json
npm version $VERSION --no-git-tag-version

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Commit
git add package.json Cargo.toml
git commit -m "chore: bump version to $VERSION"

echo "Version bumped to $VERSION"
```

### Full Release Script

```bash
#!/bin/bash
# scripts/release.sh

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "Releasing v$VERSION..."

# Bump versions
./scripts/bump-version.sh $VERSION

# Generate changelog
git cliff --tag v$VERSION --prepend CHANGELOG.md
git add CHANGELOG.md
git commit --amend --no-edit

# Create tag
git tag -s v$VERSION -m "Release v$VERSION"

# Push
git push origin main --tags

echo "Release v$VERSION complete!"
echo "GitHub Actions will handle npm and crates.io publishing."
```

## Resources

- [npm publish docs](https://docs.npmjs.com/cli/publish)
- [Cargo publish docs](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Semantic Versioning](https://semver.org)
- [Keep a Changelog](https://keepachangelog.com)
- [Conventional Commits](https://www.conventionalcommits.org)
- [git-cliff](https://git-cliff.org)
