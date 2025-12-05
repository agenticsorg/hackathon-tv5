# Releasing ARW CLI

This guide covers the complete release process for the ARW CLI package.

## Overview

The ARW CLI uses a multi-step release process:

1. Version bumping
2. Binary compilation for all platforms
3. Binary packaging and checksum generation
4. GitHub release with binary artifacts
5. NPM package publication

## Prerequisites

Before releasing:

- [ ] All tests passing (`cargo test` and `npm test`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated with release notes
- [ ] No uncommitted changes
- [ ] GitHub personal access token configured
- [ ] NPM authentication configured (`npm login`)

## Version Bumping

### 1. Update Version Numbers

Update version in **both** files to keep them in sync:

#### Cargo.toml

```toml
[package]
name = "arw-cli"
version = "0.2.0"  # Update this
```

#### npm/package.json

```json
{
  "name": "@agent-ready-web/cli",
  "version": "0.2.0"  # Update this
}
```

### 2. Update CHANGELOG.md

Add release notes:

```markdown
## [0.2.0] - 2024-01-15

### Added
- New feature X
- Command Y support

### Fixed
- Bug Z resolution

### Changed
- Breaking change description
```

### 3. Commit Changes

```bash
git add Cargo.toml npm/package.json CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git push origin main
```

## Building Binaries

### Option 1: Automated (GitHub Actions)

The easiest way is to let GitHub Actions handle everything:

```bash
# Create and push tag
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions will:
1. Build binaries for all platforms
2. Create GitHub release
3. Upload binary artifacts
4. Publish to npm

### Option 2: Manual Build

For manual releases or testing:

#### 1. Build All Platform Binaries

```bash
cd packages/cli
./scripts/build-npm-package.sh
```

This creates binaries for:
- linux-x64
- linux-arm64
- darwin-x64
- darwin-arm64
- win32-x64

#### 2. Package Binaries

```bash
./scripts/package-binaries.sh
```

This creates:
- `arw-{platform}.tar.gz` (Unix)
- `arw-windows-x64.zip` (Windows)
- `.sha256` checksum files

#### 3. Test Local Binaries

```bash
./scripts/test-npm-package.sh
```

## Creating GitHub Release

### Automated (GitHub Actions)

If you pushed a tag, GitHub Actions handles this automatically.

### Manual

1. Go to https://github.com/agent-ready-web/agent-ready-web/releases/new

2. Fill in release details:
   - **Tag**: `v0.2.0`
   - **Title**: `ARW CLI v0.2.0`
   - **Description**: Copy from CHANGELOG.md

3. Upload all artifacts from `dist/`:
   ```
   arw-darwin-arm64.tar.gz
   arw-darwin-arm64.tar.gz.sha256
   arw-darwin-x64.tar.gz
   arw-darwin-x64.tar.gz.sha256
   arw-linux-arm64.tar.gz
   arw-linux-arm64.tar.gz.sha256
   arw-linux-x64.tar.gz
   arw-linux-x64.tar.gz.sha256
   arw-windows-x64.zip
   arw-windows-x64.zip.sha256
   ```

4. Click "Publish release"

## Publishing to NPM

### Automated (GitHub Actions)

If using GitHub Actions, npm publish happens automatically after release creation.

### Manual

#### 1. Verify You're Logged In

```bash
npm whoami
# Should show your npm username
```

If not logged in:

```bash
npm login
```

#### 2. Run Publish Script

```bash
cd packages/cli
./scripts/publish-npm.sh
```

This will:
1. Run pre-publish checks
2. Build WASM fallback
3. Prompt for confirmation
4. Publish to npm

#### 3. Verify Publication

```bash
# Check package page
npm view @agent-ready-web/cli

# Test installation
npx @agent-ready-web/cli@0.2.0 --help
```

## Post-Release Tasks

### 1. Verify Installation on All Platforms

Test on each platform:

```bash
# Linux x64
npx @agent-ready-web/cli@0.2.0 --help

# macOS Intel
npx @agent-ready-web/cli@0.2.0 --help

# macOS Apple Silicon
npx @agent-ready-web/cli@0.2.0 --help

# Windows
npx @agent-ready-web/cli@0.2.0 --help
```

### 2. Update Documentation

- [ ] Update README with new features
- [ ] Update API documentation
- [ ] Update examples

### 3. Announce Release

- [ ] Tweet/post on social media
- [ ] Update project website
- [ ] Notify users in Discord/Slack
- [ ] Send email to mailing list

### 4. Monitor Issues

Watch for bug reports in the first 24-48 hours:
- GitHub Issues
- npm package page
- Support channels

## Rollback Process

If critical issues are found:

### 1. Deprecate Bad Version

```bash
npm deprecate @agent-ready-web/cli@0.2.0 "Critical bug, use 0.2.1 instead"
```

### 2. Release Patch Version

Follow the standard release process with a patch version:

```bash
# Bump to 0.2.1
# Fix the issue
# Release normally
```

### 3. Delete GitHub Release (if necessary)

Only for severe issues:

```bash
# Delete the tag
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0

# Delete the release on GitHub UI
```

## Troubleshooting

### Binary Build Fails

**Problem**: Cross-compilation fails for a platform

**Solution**:
```bash
# Install cross
cargo install cross --git https://github.com/cross-rs/cross

# Try building specific target
cross build --release --target aarch64-unknown-linux-gnu
```

### NPM Publish Fails

**Problem**: "You must verify your email address"

**Solution**:
```bash
# Check npm profile
npm profile get

# Verify email if needed
npm profile set email your@email.com
```

**Problem**: "You cannot publish over the previously published versions"

**Solution**:
```bash
# Version already exists, bump the version
# Edit Cargo.toml and package.json
# Commit and try again
```

### Binary Download Fails After Release

**Problem**: Users report binary download errors

**Solution**:
1. Check GitHub release has all artifacts
2. Verify artifact names match `platforms.js`
3. Check network/CDN issues
4. Verify checksums are correct

### WASM Fallback Not Working

**Problem**: WASM module fails to load

**Solution**:
```bash
# Rebuild WASM module
cd packages/cli
npm run build:wasm

# Test locally
node -e "require('./npm/index.js').validateManifest('test')"
```

## Release Checklist

Use this checklist for each release:

### Pre-Release
- [ ] All tests passing
- [ ] Version bumped in Cargo.toml and package.json
- [ ] CHANGELOG.md updated
- [ ] Documentation updated
- [ ] No uncommitted changes
- [ ] Branch is up to date with main

### Build
- [ ] Binaries built for all platforms
- [ ] Binaries packaged with checksums
- [ ] Local package tests passing
- [ ] WASM fallback built and tested

### Release
- [ ] GitHub release created
- [ ] All binary artifacts uploaded
- [ ] Release notes published
- [ ] NPM package published
- [ ] Package version visible on npm

### Post-Release
- [ ] Installation verified on all platforms
- [ ] Documentation updated
- [ ] Release announced
- [ ] Issues monitoring active

## Automated Release Workflow

For convenience, you can create a release script:

```bash
#!/bin/bash
# release.sh - Automated release workflow

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./release.sh <version>"
    exit 1
fi

echo "🚀 Releasing version $VERSION"

# Update versions
sed -i '' "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml
sed -i '' "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" npm/package.json

# Commit
git add Cargo.toml npm/package.json
git commit -m "chore: bump version to $VERSION"

# Tag
git tag "v$VERSION"

# Push
git push origin main
git push origin "v$VERSION"

echo "✅ Released! GitHub Actions will handle the rest."
```

## Questions?

If you have questions about the release process:

1. Check existing [GitHub Releases](https://github.com/agent-ready-web/agent-ready-web/releases)
2. Review [GitHub Actions logs](https://github.com/agent-ready-web/agent-ready-web/actions)
3. Open an issue for release-related questions
