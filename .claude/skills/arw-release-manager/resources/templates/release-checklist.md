# ARW CLI Release Checklist

Version: ____.____.____
Release Date: ____-____-____
Release Manager: ________________

## Pre-Release

### Code Quality

- [ ] All tests passing (`npm test` / `cargo test`)
- [ ] Linting clean (`npm run lint` / `cargo clippy`)
- [ ] Type checking clean (`npm run typecheck` / `cargo check`)
- [ ] No compiler warnings
- [ ] Code reviewed and approved

### Version Management

- [ ] Version bumped in `package.json`
- [ ] Version bumped in `Cargo.toml`
- [ ] Versions match between npm and cargo
- [ ] CHANGELOG.md updated with release notes
- [ ] All new features documented

### Security

- [ ] No hardcoded secrets or API keys
- [ ] Dependencies audited (`npm audit` / `cargo audit`)
- [ ] Security vulnerabilities addressed
- [ ] .npmignore and .gitignore properly configured

### Build Verification

- [ ] TypeScript builds successfully (`npm run build`)
- [ ] Rust release builds successfully (`cargo build --release`)
- [ ] CLI binary works (`./target/release/arw-cli --version`)
- [ ] Package size is reasonable (`npm pack --dry-run`)

## Local Testing

### TypeScript/npm

- [ ] `npm link` installs globally
- [ ] `arw-cli --version` shows correct version
- [ ] `arw-cli --help` displays help
- [ ] Core commands work as expected
- [ ] `npm unlink` cleans up

### Rust/Cargo

- [ ] `cargo install --path .` works
- [ ] Binary executes correctly
- [ ] All subcommands functional

## Publishing

### npm

- [ ] Logged in (`npm whoami`)
- [ ] Dry run successful (`npm publish --dry-run`)
- [ ] Published (`npm publish --access public`)
- [ ] Verified on npmjs.com

### crates.io

- [ ] Logged in (`cargo login`)
- [ ] Dry run successful (`cargo publish --dry-run`)
- [ ] Published (`cargo publish`)
- [ ] Verified on crates.io

## Post-Release

### Git

- [ ] All changes committed
- [ ] Tag created (`git tag v{VERSION}`)
- [ ] Tag pushed (`git push origin v{VERSION}`)
- [ ] GitHub release created with notes

### Verification

- [ ] Fresh install works (`npm install -g arw-cli@{VERSION}`)
- [ ] Fresh install works (`cargo install arw-cli@{VERSION}`)
- [ ] Version command shows new version
- [ ] No regressions in core functionality

### Communication

- [ ] Release notes published
- [ ] Team notified
- [ ] Documentation updated (if needed)
- [ ] Breaking changes communicated

## Rollback Plan (if needed)

If critical issues are found:

1. **npm**: `npm deprecate arw-cli@{VERSION} "Critical bug, use {PREV_VERSION}"`
2. **crates.io**: Cannot unpublish, but can yank: `cargo yank --vers {VERSION}`
3. Tag previous version as latest
4. Communicate issue to users

## Notes

_Add any release-specific notes here_

---

**Sign-off**

Release Manager: ________________ Date: ________
Reviewer: ________________ Date: ________
