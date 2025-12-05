# Agent-Ready Web CLI (Alpha)

**Version**: 0.3.0-alpha.1
**Status**: 🧪 Alpha Release - Testing & Feedback Welcome
**Released**: 2025-11-17

---

## ⚠️ Alpha Release Notice

This is an **alpha release** intended for early testing and feedback. While the core functionality is stable, some features are incomplete and breaking changes may occur in future alpha versions.

**Not recommended for production use yet.** Please wait for stable 0.3.0 release.

---

## 📦 Installation

### NPM/NPX (Recommended for Alpha Testing)

```bash
# Install globally
npm install -g @agent-ready-web/cli@alpha

# Verify installation
arw --version
# Output: arw-cli 0.3.0-alpha.1

# Or use with npx (no installation)
npx @agent-ready-web/cli@alpha validate
```

### Cargo (Rust)

```bash
# Install from crates.io
cargo install arw-cli --version 0.3.0-alpha.1

# Verify installation
arw --version
```

### From Source

```bash
# Clone repository
git clone https://github.com/your-org/agent-ready-web.git
cd agent-ready-web/packages/cli

# Build with cargo
cargo build --release

# Binary location
./target/release/arw --version
```

---

## 🚀 Quick Start

```bash
# Initialize a new ARW project
arw init

# Validate your site for ARW conformance
arw validate

# Generate discovery files (llms.txt, sitemap.xml, .llm.md)
arw generate

# Build for production
arw build

# Start development server
arw serve --port 3000
```

---

## ✨ What's New in Alpha 1

### Core Features
✅ **Multi-platform Support** - Linux, macOS, Windows (x64, arm64)
✅ **WASM Distribution** - Run in browser or Node.js
✅ **150+ Tests** - Comprehensive test suite with 90%+ coverage
✅ **10 Commands** - Full CLI interface (10/12 complete)
✅ **8 Schema Types** - Generate schemas for Product, Article, etc.
✅ **Dev Server** - Built-in server with CORS support

### ARW Conformance
- **ARW-1 (Discovery)**: ✅ 100% - Discovery manifest, machine views
- **ARW-2 (Semantic)**: ✅ 95% - Content chunking, semantic metadata
- **ARW-3 (Actions)**: ⚠️ 80% - Action discovery, OAuth flows
- **ARW-4 (Protocol)**: 🚧 30% - Protocol negotiation (in progress)

### Performance
- **40% faster validation** on large sites (100+ pages)
- **60% faster generation** with parallel processing
- **25% reduced memory** footprint

---

## 🎯 Available Commands

### Core Commands

```bash
# Validate ARW conformance
arw validate [path]
arw validate --strict          # Strict mode (fail on warnings)
arw validate -v                # Verbose output

# Generate all discovery files
arw generate
arw generate --sitemap         # Only sitemap
arw generate --llms-txt        # Only llms.txt
arw generate --machine-views   # Only .llm.md files

# Build for production
arw build
arw build --output dist        # Custom output directory

# Development server
arw serve
arw serve --port 8080         # Custom port
arw serve --host 0.0.0.0      # Expose to network
```

### Generation Commands

```bash
# Generate discovery manifest
arw generate-manifest

# Generate JSON schemas
arw generate-schema --type product
arw generate-schema --type article --output schema.json

# Generate sitemaps
arw generate-sitemap
arw generate-sitemap --format json

# Generate robots.txt
arw generate-robots
```

### Utility Commands

```bash
# Initialize new project
arw init
arw init --template blog      # Use template

# Verify conformance
arw verify
arw verify --report json      # JSON output

# Get help
arw --help
arw validate --help           # Command-specific help

# Version info
arw --version
```

---

## 📋 Known Issues (Alpha 1)

### High Priority
🔴 **ARW-4 (Protocol)** - Only 30% implemented (planned for beta)
🔴 **Action Validation** - Some edge cases need refinement
🔴 **WASM Bundle Size** - Currently ~2MB uncompressed (optimizing)

### Medium Priority
🟡 **Large Sites** - Performance optimization ongoing for 1000+ pages
🟡 **Memory Usage** - Spikes on concurrent generation of 100+ files
🟡 **Windows Paths** - Minor edge cases with UNC paths

### Low Priority
🟢 **CLI Formatting** - Output styling inconsistent across commands
🟢 **Progress Bars** - Not shown for some long-running operations
🟢 **Help Text** - Could be more detailed for complex commands

---

## 🧪 Testing & Feedback

We need your help to make this stable!

### What to Test
1. **Installation** - Does it install cleanly on your platform?
2. **Validation** - Does it correctly validate your ARW sites?
3. **Generation** - Are generated files correct and complete?
4. **Performance** - How does it perform on your site size?
5. **Errors** - Are error messages helpful and actionable?

### How to Report Issues

**GitHub Issues**: https://github.com/your-org/agent-ready-web/issues

**Label your issue**: `alpha-0.3.0`

**Include**:
- Platform (OS, architecture)
- Installation method (npm, cargo, source)
- Command being run
- Error output (if any)
- Site size (number of pages)

### Example Issue

```markdown
**Title**: Validation fails on large site with 500+ pages

**Platform**: macOS 14.1 (arm64)
**Installation**: npm install -g @agent-ready-web/cli@alpha
**Version**: 0.3.0-alpha.1

**Command**:
arw validate /path/to/site

**Error**:
[error output here]

**Site Info**:
- 523 pages
- 1.2GB total size
- Multiple languages

**Expected**: Validation should complete
**Actual**: Crashes after 200 pages
```

---

## 📖 Documentation

### Quick Links
- [Full README](./README.md) - Complete documentation (5,300+ lines)
- [CLI Guide](./CLI.md) - All commands and options
- [Testing Guide](./TESTING.md) - Run tests locally
- [Changelog](./CHANGELOG.md) - What's new
- [WASM Guide](./WASM.md) - WebAssembly usage

### External Resources
- [ARW Specification](https://github.com/agent-ready-web/spec)
- [Examples](../../examples/) - Example sites
- [Community Forum](https://github.com/your-org/agent-ready-web/discussions)

---

## 🔄 Roadmap

### Beta Release (0.3.0-beta.1) - Target: 4-6 weeks
- [ ] Complete ARW-4 implementation (70% remaining)
- [ ] Refine action validation edge cases
- [ ] Optimize WASM bundle size (target: <1MB)
- [ ] Add progress indicators for all commands
- [ ] Implement streaming for large file generation
- [ ] Add configuration file support (`.arwrc`)

### Stable Release (0.3.0) - Target: 8-12 weeks
- [ ] 100% test coverage for critical paths
- [ ] Performance benchmarks for 10,000+ page sites
- [ ] Comprehensive error recovery
- [ ] Full documentation with examples
- [ ] Tutorial and getting started guide
- [ ] API stability guarantees

---

## 🏗️ Platform Support

### Operating Systems
✅ Linux (glibc 2.17+, musl)
✅ macOS 10.15+ (Catalina and later)
✅ Windows 10+ (64-bit)

### Architectures
✅ x86_64 (x64)
✅ aarch64 (arm64)
✅ armv7 (Linux only)

### Node.js Versions
✅ Node.js 16.x
✅ Node.js 18.x (LTS)
✅ Node.js 20.x (LTS)
✅ Node.js 22.x

### Rust MSRV
Minimum Supported Rust Version: **1.70.0**

---

## 🔐 Security

This alpha release includes:

- ✅ Input validation on all user inputs
- ✅ Path traversal prevention
- ✅ XSS prevention in HTML parsing
- ✅ Dependency security audit (cargo audit, npm audit)
- ✅ No unsafe Rust code in core logic
- ✅ Sandboxed execution (no elevated privileges)

**Security Issues**: Please report to security@your-org.com

---

## 🤝 Contributing

We welcome contributions! Here's how to help:

### Bug Reports
1. Check existing issues
2. Create new issue with `alpha-0.3.0` label
3. Include reproduction steps

### Feature Requests
1. Open discussion first
2. Get community feedback
3. Submit PR with tests

### Code Contributions
1. Fork repository
2. Create feature branch
3. Write tests (required)
4. Submit PR with description

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for details.

---

## 📊 Usage Examples

### Example 1: Validate Blog

```bash
# Navigate to your blog directory
cd ~/my-blog

# Run validation
npx @agent-ready-web/cli@alpha validate

# Output:
# ✓ Discovery manifest valid
# ✓ 12 machine views (.llm.md) valid
# ✓ Semantic chunking present
# ✓ ARW-1 conformance: 100%
# ✓ ARW-2 conformance: 95%
# ⚠ ARW-3 conformance: 80% (some actions incomplete)
#
# Overall: PASS (with warnings)
```

### Example 2: Generate Discovery Files

```bash
# Generate all files
npx @agent-ready-web/cli@alpha generate

# Output:
# ✓ llms.txt created (12 pages listed)
# ✓ sitemap.xml generated
# ✓ sitemap.llm.json generated
# ✓ 12 .llm.md files created
# ✓ robots.txt generated
#
# Discovery files ready in current directory
```

### Example 3: Development Server

```bash
# Start dev server
npx @agent-ready-web/cli@alpha serve --port 3000

# Output:
# Server running at:
#   Local:   http://localhost:3000
#   Network: http://192.168.1.100:3000
#
# CORS enabled for all origins
# Press Ctrl+C to stop

# Test endpoints:
curl http://localhost:3000/llms.txt
curl http://localhost:3000/sitemap.xml
curl http://localhost:3000/index.html
```

---

## 🆚 Comparison: Alpha vs Stable

| Feature | Alpha 0.3.0 | Stable 0.3.0 (Target) |
|---------|-------------|------------------------|
| ARW-1 (Discovery) | ✅ 100% | ✅ 100% |
| ARW-2 (Semantic) | ✅ 95% | ✅ 100% |
| ARW-3 (Actions) | ⚠️ 80% | ✅ 100% |
| ARW-4 (Protocol) | 🚧 30% | ✅ 100% |
| Test Coverage | 90%/80% | 100%/95% |
| Performance | Good | Excellent |
| Documentation | Complete | Enhanced |
| Breaking Changes | Possible | Guaranteed stable |
| Production Ready | ❌ No | ✅ Yes |

---

## 💬 Community & Support

### Get Help
- **GitHub Discussions**: Ask questions, share ideas
- **GitHub Issues**: Report bugs, request features
- **Documentation**: Comprehensive guides available

### Stay Updated
- **Changelog**: See [CHANGELOG.md](./CHANGELOG.md)
- **Releases**: Watch GitHub releases
- **Blog**: Development updates (coming soon)

### Feedback Channels
- Alpha testing feedback: alpha-feedback@your-org.com
- General questions: support@your-org.com
- Security issues: security@your-org.com

---

## 📜 License

MIT License - See [LICENSE](../../LICENSE) for details

---

## 🙏 Acknowledgments

Thanks to all alpha testers and contributors!

Special thanks to:
- Early adopters who provided feedback
- Contributors who submitted PRs
- Community members who helped with testing

---

## 🎉 Happy Testing!

This is just the beginning. Your feedback will shape the stable release.

**Let's make the web agent-ready together!** 🤖🌐

---

*Last updated: 2025-11-17*
*Next alpha: TBD based on feedback*
*Beta target: 4-6 weeks*
*Stable target: 8-12 weeks*
