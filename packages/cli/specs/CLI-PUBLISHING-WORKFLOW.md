# ARW CLI Publishing Workflow

## Multi-Platform Distribution Strategy for Rust and Node.js Ecosystems

**Version:** 1.0
**Date:** January 2025
**Related:** CLI-EXPANSION-PLAN.md

---

## Table of Contents

1. [Overview](#1-overview)
2. [Publishing to crates.io (Rust)](#2-publishing-to-cratesio-rust)
3. [Publishing to npm (Node.js)](#3-publishing-to-npm-nodejs)
4. [Alternative Distribution Channels](#4-alternative-distribution-channels)
5. [Automated Release Process](#5-automated-release-process)
6. [Version Management](#6-version-management)
7. [Release Checklist](#7-release-checklist)

---

## 1. Overview

The ARW CLI will be distributed through multiple channels to reach both Rust and Node.js ecosystems. This document outlines the complete publishing workflow for releasing new versions across all platforms.

### Distribution Channels

| Channel             | Target Audience    | Priority | Status     |
| ------------------- | ------------------ | -------- | ---------- |
| **crates.io**       | Rust developers    | P0       | ✅ Planned |
| **npm**             | Node.js developers | P0       | ✅ Planned |
| **GitHub Releases** | All users          | P0       | ✅ Planned |
| **Homebrew**        | macOS/Linux users  | P1       | ✅ Planned |
| **Scoop**           | Windows users      | P2       | ✅ Planned |
| **APT/YUM**         | Linux users        | P3       | 🔄 Future  |
| **Docker Hub**      | Container users    | P1       | ✅ Planned |

---

## 2. Publishing to crates.io (Rust)

### 2.1 Package Configuration

**Cargo.toml setup:**

```toml
[package]
name = "arw-cli"
version = "0.2.0"
edition = "2021"
authors = ["Nolan Dubeau <nolan@example.com>"]
description = "CLI tool for implementing Agent-Ready Web (ARW) specification - make your website accessible to AI agents"
documentation = "https://docs.rs/arw-cli"
homepage = "https://github.com/nolandubeau/agent-ready-web"
repository = "https://github.com/nolandubeau/agent-ready-web"
license = "MIT"
readme = "README.md"
keywords = ["arw", "ai", "agents", "web", "cli"]
categories = ["command-line-utilities", "web-programming"]
exclude = [
    ".github/",
    "tests/fixtures/",
    "*.tar.gz",
    "*.zip"
]
include = [
    "src/**/*",
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSE",
    "CHANGELOG.md"
]

[badges]
maintenance = { status = "actively-developed" }
codecov = { repository = "nolandubeau/agent-ready-web", branch = "main", service = "github" }

[[bin]]
name = "arw"
path = "src/main.rs"

[dependencies]
# ... existing dependencies
```

### 2.2 Pre-publish Checklist

```bash
#!/bin/bash
# scripts/pre-publish-crates.sh

set -e

echo "🔍 Pre-publish checks for crates.io..."

# 1. Check if logged in
if ! cargo login --list &> /dev/null; then
    echo "❌ Not logged in to crates.io"
    echo "Run: cargo login"
    exit 1
fi

# 2. Update version in Cargo.toml
echo "📝 Current version:"
grep "^version = " cli/Cargo.toml

# 3. Run tests
echo "🧪 Running tests..."
cd cli
cargo test --all-features

# 4. Check formatting
echo "🎨 Checking code formatting..."
cargo fmt -- --check

# 5. Run clippy
echo "📎 Running clippy..."
cargo clippy -- -D warnings

# 6. Build release binary
echo "🔨 Building release binary..."
cargo build --release

# 7. Check package contents
echo "📦 Checking package contents..."
cargo package --list --allow-dirty

# 8. Dry run publish
echo "🏃 Dry run publish..."
cargo publish --dry-run --allow-dirty

echo "✅ Pre-publish checks passed!"
echo ""
echo "Next steps:"
echo "  1. Review CHANGELOG.md"
echo "  2. Commit version bump"
echo "  3. Create git tag"
echo "  4. Run: cargo publish"
```

### 2.3 Publishing Process

**Manual publish:**

```bash
# 1. Update version
cd cli
# Edit Cargo.toml version field

# 2. Update CHANGELOG.md
# Add release notes

# 3. Run pre-publish checks
cd .. && ./scripts/pre-publish-crates.sh

# 4. Commit changes
git add cli/Cargo.toml cli/CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"

# 5. Create git tag
git tag -a v0.2.0 -m "Release v0.2.0"

# 6. Publish to crates.io
cd cli
cargo publish

# 7. Push changes and tags
git push origin main
git push origin v0.2.0
```

**Automated publish (GitHub Actions):**

```yaml
# .github/workflows/publish-crates.yml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  publish:
    name: Publish to crates.io
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          components: rustfmt, clippy

      - name: Run tests
        run: cd cli && cargo test --all-features

      - name: Check formatting
        run: cd cli && cargo fmt -- --check

      - name: Run clippy
        run: cd cli && cargo clippy -- -D warnings

      - name: Publish to crates.io
        run: cd cli && cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          draft: false
          prerelease: false
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 2.4 Post-publish Verification

```bash
# Verify package published
cargo search arw-cli

# Install from crates.io
cargo install arw-cli

# Test installed binary
arw --version
arw init --help

# Verify documentation
open https://docs.rs/arw-cli
```

---

## 3. Publishing to npm (Node.js)

### 3.1 NPM Package Structure

```
npm/
├── package.json          # Package manifest
├── README.md             # npm-specific readme
├── LICENSE               # MIT license
├── index.js              # Main entry point
├── bin/
│   └── arw.js           # Executable wrapper
├── scripts/
│   ├── postinstall.js   # Download binary after install
│   ├── download.js      # Binary download logic
│   └── platform.js      # Platform detection
├── test/
│   └── test.js          # Basic tests
└── .npmignore           # Files to exclude from package
```

### 3.2 Package Configuration

**package.json:**

```json
{
  "name": "@agent-ready-web/cli",
  "version": "0.2.0",
  "description": "CLI tool for implementing Agent-Ready Web (ARW) - make your website accessible to AI agents",
  "main": "index.js",
  "bin": {
    "arw": "./bin/arw.js"
  },
  "scripts": {
    "postinstall": "node scripts/postinstall.js",
    "test": "node test/test.js",
    "prepublishOnly": "npm test"
  },
  "keywords": ["arw", "agent-ready-web", "ai", "agents", "llm", "cli", "web", "machine-readable"],
  "author": "Nolan Dubeau <nolan@example.com>",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/nolandubeau/agent-ready-web.git",
    "directory": "npm"
  },
  "bugs": {
    "url": "https://github.com/nolandubeau/agent-ready-web/issues"
  },
  "homepage": "https://github.com/nolandubeau/agent-ready-web#readme",
  "engines": {
    "node": ">=14.0.0"
  },
  "os": ["darwin", "linux", "win32"],
  "cpu": ["x64", "arm64"],
  "dependencies": {
    "node-fetch": "^3.3.2"
  },
  "devDependencies": {
    "jest": "^29.7.0"
  },
  "files": ["index.js", "bin/", "scripts/", "README.md", "LICENSE"]
}
```

### 3.3 Binary Wrapper

**bin/arw.js:**

```javascript
#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Detect platform and architecture
const platform = process.platform;
const arch = process.arch;

// Map to binary path
const binaryName = platform === 'win32' ? 'arw.exe' : 'arw';
const binaryPath = path.join(__dirname, '..', '.bin', `${platform}-${arch}`, binaryName);

// Check if binary exists
if (!fs.existsSync(binaryPath)) {
  console.error(`❌ Binary not found for ${platform}-${arch}`);
  console.error('Supported platforms:');
  console.error('  - macOS: darwin-x64, darwin-arm64');
  console.error('  - Linux: linux-x64, linux-arm64');
  console.error('  - Windows: win32-x64');
  console.error('');
  console.error('Please report this issue:');
  console.error('https://github.com/nolandubeau/agent-ready-web/issues');
  process.exit(1);
}

// Execute the Rust binary with all arguments
const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  windowsHide: true,
});

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code);
  }
});

child.on('error', (error) => {
  console.error('❌ Failed to start ARW CLI:', error.message);
  process.exit(1);
});
```

**scripts/postinstall.js:**

```javascript
const https = require('https');
const fs = require('fs');
const path = require('path');
const { promisify } = require('util');
const stream = require('stream');
const { execSync } = require('child_process');

const pipeline = promisify(stream.pipeline);

const VERSION = require('../package.json').version;
const REPO = 'nolandubeau/agent-ready-web';

async function downloadBinary() {
  const platform = process.platform;
  const arch = process.arch;

  // Platform mapping
  const platformMap = {
    darwin: {
      x64: 'x86_64-apple-darwin',
      arm64: 'aarch64-apple-darwin',
    },
    linux: {
      x64: 'x86_64-unknown-linux-gnu',
      arm64: 'aarch64-unknown-linux-gnu',
    },
    win32: {
      x64: 'x86_64-pc-windows-gnu',
    },
  };

  if (!platformMap[platform] || !platformMap[platform][arch]) {
    console.error(`❌ Unsupported platform: ${platform}-${arch}`);
    process.exit(1);
  }

  const target = platformMap[platform][arch];
  const binaryName = platform === 'win32' ? 'arw.exe' : 'arw';
  const assetName = `arw-${target}${platform === 'win32' ? '.exe' : ''}`;

  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;

  const binDir = path.join(__dirname, '..', '.bin', `${platform}-${arch}`);
  const binaryPath = path.join(binDir, binaryName);

  // Create directory
  fs.mkdirSync(binDir, { recursive: true });

  console.log(`📥 Downloading ARW CLI binary for ${platform}-${arch}...`);
  console.log(`   URL: ${url}`);

  return new Promise((resolve, reject) => {
    https
      .get(
        url,
        {
          headers: {
            'User-Agent': 'arw-npm-installer',
          },
        },
        (response) => {
          // Handle redirects
          if (response.statusCode === 302 || response.statusCode === 301) {
            https
              .get(response.headers.location, (redirectResponse) => {
                if (redirectResponse.statusCode !== 200) {
                  reject(new Error(`Failed to download binary: ${redirectResponse.statusCode}`));
                  return;
                }

                const fileStream = fs.createWriteStream(binaryPath);
                pipeline(redirectResponse, fileStream)
                  .then(() => {
                    // Make executable on Unix
                    if (platform !== 'win32') {
                      fs.chmodSync(binaryPath, 0o755);
                    }
                    console.log('✅ Binary downloaded and installed successfully');
                    resolve();
                  })
                  .catch(reject);
              })
              .on('error', reject);
          } else if (response.statusCode === 200) {
            const fileStream = fs.createWriteStream(binaryPath);
            pipeline(response, fileStream)
              .then(() => {
                if (platform !== 'win32') {
                  fs.chmodSync(binaryPath, 0o755);
                }
                console.log('✅ Binary downloaded and installed successfully');
                resolve();
              })
              .catch(reject);
          } else {
            reject(new Error(`Failed to download binary: ${response.statusCode}`));
          }
        }
      )
      .on('error', reject);
  });
}

// Run download
downloadBinary().catch((err) => {
  console.error('❌ Failed to download binary:', err.message);
  console.error('');
  console.error('You can try:');
  console.error('  1. Install from crates.io: cargo install arw-cli');
  console.error(
    '  2. Download manually from: https://github.com/nolandubeau/agent-ready-web/releases'
  );
  console.error('  3. Report the issue: https://github.com/nolandubeau/agent-ready-web/issues');
  process.exit(1);
});
```

### 3.4 Publishing Process

**Preparation:**

```bash
#!/bin/bash
# scripts/prepare-npm-publish.sh

set -e

echo "📦 Preparing npm package..."

# 1. Ensure binaries are built for all platforms
echo "🔨 Building binaries for all platforms..."

# Note: This should be done in GitHub Actions for cross-compilation
# Manual process requires access to each platform or cross-compilation setup

platforms=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
)

for target in "${platforms[@]}"; do
    echo "Building for $target..."
    cargo build --release --target "$target"
done

# 2. Create GitHub release with binaries
# This step uploads binaries to GitHub Releases
# Done via GitHub Actions workflow

# 3. Test npm package locally
cd npm
npm install
npm test

echo "✅ npm package ready for publishing"
```

**Manual publish:**

```bash
# 1. Update version in package.json
cd npm
# Edit package.json version field

# 2. Test package locally
npm install
npm test

# 3. Login to npm (first time only)
npm login

# 4. Publish (with public access for scoped package)
npm publish --access public

# 5. Verify
npm view @agent-ready-web/cli
```

**Automated publish (GitHub Actions):**

```yaml
# .github/workflows/publish-npm.yml
name: Publish to npm

on:
  release:
    types: [created]

jobs:
  build-binaries:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-gnu

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package binary
        shell: bash
        run: |
          cd cli/target/${{ matrix.target }}/release
          if [ "${{ runner.os }}" = "Windows" ]; then
            7z a arw-${{ matrix.target }}.zip arw.exe
          else
            tar czf arw-${{ matrix.target }}.tar.gz arw
          fi
          mv arw-${{ matrix.target }}.* $GITHUB_WORKSPACE/

      - name: Upload binary to release
        uses: softprops/action-gh-release@v1
        with:
          files: arw-${{ matrix.target }}.*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish-npm:
    name: Publish to npm
    needs: build-binaries
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'

      - name: Install dependencies
        run: cd npm && npm install

      - name: Run tests
        run: cd npm && npm test

      - name: Publish to npm
        run: cd npm && npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

      - name: Verify publish
        run: |
          sleep 30  # Wait for npm to propagate
          npm view @agent-ready-web/cli version
```

### 3.5 Testing npm Package

```bash
# Test installation
npm install -g @agent-ready-web/cli

# Verify binary works
arw --version
arw init --help

# Test in a project
cd /tmp/test-project
npm init -y
npm install @agent-ready-web/cli
npx arw init --yes
```

---

## 4. Alternative Distribution Channels

### 4.1 Homebrew (macOS/Linux)

**Formula: arw.rb**

```ruby
class Arw < Formula
  desc "CLI tool for implementing Agent-Ready Web (ARW)"
  homepage "https://github.com/nolandubeau/agent-ready-web"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/nolandubeau/agent-ready-web/releases/download/v0.2.0/arw-aarch64-apple-darwin.tar.gz"
      sha256 "..." # SHA256 of the file
    else
      url "https://github.com/nolandubeau/agent-ready-web/releases/download/v0.2.0/arw-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/nolandubeau/agent-ready-web/releases/download/v0.2.0/arw-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "..."
    else
      url "https://github.com/nolandubeau/agent-ready-web/releases/download/v0.2.0/arw-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "..."
    end
  end

  def install
    bin.install "arw"
  end

  test do
    assert_match "arw #{version}", shell_output("#{bin}/arw --version")
  end
end
```

**Tap repository setup:**

```bash
# Create tap repository
# Repository: nolandubeau/homebrew-arw

# Formula location:
# Formula/arw.rb

# Installation for users:
brew tap nolandubeau/arw
brew install arw

# Or one-liner:
brew install nolandubeau/arw/arw
```

### 4.2 Scoop (Windows)

**Manifest: arw.json**

```json
{
  "version": "0.2.0",
  "description": "CLI tool for implementing Agent-Ready Web (ARW)",
  "homepage": "https://github.com/nolandubeau/agent-ready-web",
  "license": "MIT",
  "architecture": {
    "64bit": {
      "url": "https://github.com/nolandubeau/agent-ready-web/releases/download/v0.2.0/arw-x86_64-pc-windows-gnu.zip",
      "hash": "...",
      "bin": "arw.exe"
    }
  },
  "checkver": {
    "github": "https://github.com/nolandubeau/agent-ready-web"
  },
  "autoupdate": {
    "architecture": {
      "64bit": {
        "url": "https://github.com/nolandubeau/agent-ready-web/releases/download/v$version/arw-x86_64-pc-windows-gnu.zip"
      }
    }
  }
}
```

**Bucket repository:**

```powershell
# Create bucket repository
# Repository: nolandubeau/scoop-arw

# Manifest location:
# bucket/arw.json

# Installation for users:
scoop bucket add arw https://github.com/nolandubeau/scoop-arw
scoop install arw
```

### 4.3 Docker Image

**Dockerfile:**

```dockerfile
# Multi-stage build for minimal image size

FROM rust:1.75-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy source
COPY cli/ .

# Build release binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime image
FROM alpine:latest

RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/arw /usr/local/bin/arw

# Create working directory
WORKDIR /workspace

# Default command
ENTRYPOINT ["arw"]
CMD ["--help"]
```

**Build and publish:**

```bash
#!/bin/bash
# scripts/publish-docker.sh

set -e

VERSION="0.2.0"
IMAGE="agentreadyweb/arw-cli"

echo "🐳 Building Docker image..."

# Build image
docker build -t "${IMAGE}:${VERSION}" -t "${IMAGE}:latest" -f docker/Dockerfile .

# Test image
docker run --rm "${IMAGE}:${VERSION}" --version

# Login to Docker Hub
docker login

# Push images
docker push "${IMAGE}:${VERSION}"
docker push "${IMAGE}:latest"

echo "✅ Docker images published"
echo "   ${IMAGE}:${VERSION}"
echo "   ${IMAGE}:latest"
```

**Usage:**

```bash
# Run CLI in Docker
docker run --rm agentreadyweb/arw-cli:latest --version

# Mount local directory
docker run --rm -v $(pwd):/workspace agentreadyweb/arw-cli:latest init --yes

# Docker Compose
version: '3.8'
services:
  arw:
    image: agentreadyweb/arw-cli:latest
    volumes:
      - ./:/workspace
    working_dir: /workspace
```

---

## 5. Automated Release Process

### 5.1 Complete Release Workflow

**GitHub Actions: release.yml**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  test:
    name: Run Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd cli && cargo test --all-features

  build-binaries:
    name: Build ${{ matrix.target }}
    needs: test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-gnu

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}

      - name: Build
        run: cd cli && cargo build --release --target ${{ matrix.target }}

      - name: Package
        shell: bash
        run: |
          cd cli/target/${{ matrix.target }}/release
          if [ "${{ runner.os }}" = "Windows" ]; then
            ARCHIVE="arw-${{ matrix.target }}.zip"
            7z a "$ARCHIVE" arw.exe
            echo "ASSET=$ARCHIVE" >> $GITHUB_ENV
          else
            ARCHIVE="arw-${{ matrix.target }}.tar.gz"
            tar czf "$ARCHIVE" arw
            echo "ASSET=$ARCHIVE" >> $GITHUB_ENV
          fi

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: arw-${{ matrix.target }}
          path: cli/target/${{ matrix.target }}/release/${{ env.ASSET }}

  create-release:
    name: Create GitHub Release
    needs: build-binaries
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Download artifacts
        uses: actions/download-artifact@v3
        with:
          path: artifacts

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          draft: false
          generate_release_notes: true
          files: artifacts/**/*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish-crates:
    name: Publish to crates.io
    needs: create-release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd cli && cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  publish-npm:
    name: Publish to npm
    needs: create-release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'
      - run: cd npm && npm install && npm test
      - run: cd npm && npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  publish-docker:
    name: Publish Docker Image
    needs: create-release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Extract version
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          file: docker/Dockerfile
          push: true
          tags: |
            agentreadyweb/arw-cli:${{ steps.version.outputs.VERSION }}
            agentreadyweb/arw-cli:latest
```

---

## 6. Version Management

### 6.1 Semantic Versioning

Follow [SemVer 2.0.0](https://semver.org/):

- **Major (X.0.0)**: Breaking changes
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes, backward compatible

### 6.2 Version Bumping Script

```bash
#!/bin/bash
# scripts/bump-version.sh

set -e

if [ -z "$1" ]; then
    echo "Usage: ./scripts/bump-version.sh <major|minor|patch>"
    exit 1
fi

BUMP_TYPE=$1
CURRENT_VERSION=$(grep '^version = ' cli/Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "Current version: $CURRENT_VERSION"

# Calculate new version
IFS='.' read -r -a version_parts <<< "$CURRENT_VERSION"
MAJOR=${version_parts[0]}
MINOR=${version_parts[1]}
PATCH=${version_parts[2]}

case $BUMP_TYPE in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
    *)
        echo "Invalid bump type: $BUMP_TYPE"
        exit 1
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo "New version: $NEW_VERSION"

# Update Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" cli/Cargo.toml
rm cli/Cargo.toml.bak

# Update package.json
sed -i.bak "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" npm/package.json
rm npm/package.json.bak

echo "✅ Version bumped to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  1. Update CHANGELOG.md"
echo "  2. Commit: git commit -am 'chore: bump version to $NEW_VERSION'"
echo "  3. Tag: git tag -a v$NEW_VERSION -m 'Release v$NEW_VERSION'"
echo "  4. Push: git push origin main && git push origin v$NEW_VERSION"
```

### 6.3 CHANGELOG Management

Use [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New features go here

### Changed

- Changes to existing features

### Deprecated

- Features marked for removal

### Removed

- Removed features

### Fixed

- Bug fixes

### Security

- Security updates

## [0.2.0] - 2025-01-27

### Added

- robots.txt generation and validation
- Standard sitemap.xml support
- Migration from llms.txt command
- Comprehensive test suite
- npm package distribution
- Docker image

### Changed

- Improved chunking algorithms
- Enhanced validation with multiple levels
- Better error messages

### Fixed

- Fixed issue with special characters in YAML
- Corrected sitemap generation for large sites

## [0.1.0] - 2025-01-15

### Added

- Initial release
- Basic ARW initialization
- Machine view generation
- sitemap.llm.json generation
- Basic validation
- Development server
```

---

## 7. Release Checklist

### 7.1 Pre-Release

- [ ] All tests passing
- [ ] Code formatted and linted
- [ ] CHANGELOG.md updated
- [ ] Version bumped in:
  - [ ] cli/Cargo.toml
  - [ ] npm/package.json
- [ ] Documentation updated
- [ ] README examples tested
- [ ] Breaking changes documented

### 7.2 Release

- [ ] Create git tag: `git tag -a v0.2.0 -m "Release v0.2.0"`
- [ ] Push tag: `git push origin v0.2.0`
- [ ] Wait for GitHub Actions to complete
- [ ] Verify GitHub Release created
- [ ] Verify crates.io publish
- [ ] Verify npm publish
- [ ] Verify Docker Hub publish

### 7.3 Post-Release

- [ ] Test installation from crates.io: `cargo install arw-cli`
- [ ] Test installation from npm: `npm install -g @agent-ready-web/cli`
- [ ] Test Docker image: `docker run agentreadyweb/arw-cli:latest --version`
- [ ] Update documentation site
- [ ] Announce release on:
  - [ ] GitHub Discussions
  - [ ] Twitter/X
  - [ ] Reddit (if applicable)
  - [ ] Discord/Community
- [ ] Update Homebrew formula (if version changed significantly)
- [ ] Update Scoop manifest (if version changed significantly)

---

## Conclusion

This comprehensive publishing workflow ensures the ARW CLI reaches developers across multiple ecosystems:

1. **Rust Developers** - via crates.io
2. **Node.js Developers** - via npm
3. **macOS/Linux Users** - via Homebrew
4. **Windows Users** - via Scoop
5. **Container Users** - via Docker Hub
6. **All Users** - via GitHub Releases

The automated release process through GitHub Actions ensures consistent, reliable releases with minimal manual intervention.

---

**Related Documents:**

- CLI-EXPANSION-PLAN.md
- CLI-TESTING-STRATEGY.md
- CLI-STANDARDS-INTEGRATION.md
