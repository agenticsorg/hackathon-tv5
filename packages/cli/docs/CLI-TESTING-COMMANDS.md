# ARW CLI Testing Commands Reference

**Version:** 0.1.0 with TOON Support
**Last Updated:** 2025-11-19
**Purpose:** Complete command reference for testing ARW CLI with TOON integration

---

## Quick Test Workflow

```bash
# 1. Build the CLI
cd packages/cli
cargo build --release

# 2. Run basic init
./target/release/arw init --path ./test-site --yes

# 3. Test TOON generation
./target/release/arw generate ./test-site/index.html --output-format toon

# 4. Test validation
./target/release/arw validate ./test-site

# 5. Run tests
cargo test --lib
```

---

## Core Commands

### 1. `arw init` - Initialize ARW Structure

**Synopsis:**
```bash
arw init [OPTIONS]
```

**Arguments:**
- `--path <PATH>` - Site root directory (default: `public`)
- `-y, --yes` - Skip prompts, use defaults

**Test Commands:**
```bash
# Basic initialization
arw init

# Initialize specific path with auto-yes
arw init --path ./my-site --yes

# Initialize with prompts
arw init --path ./test-project
```

**Expected Output:**
- Creates `.arw/config.yaml`
- Creates `public/llms.txt` (YAML manifest)
- Creates `public/index.llm.md` (example machine view)

---

### 2. `arw generate` - Generate Machine Views

**Synopsis:**
```bash
arw generate <SOURCE> [OPTIONS]
```

**Arguments:**
- `<SOURCE>` - Input file or directory (required)
- `--output <DIR>` - Output directory
- `-r, --recursive` - Process directories recursively
- `-i, --input-format <FORMAT>` - Input format (html, markdown, auto) [default: auto]
- `--output-format <FORMAT>` - Output format (markdown, toon) [default: markdown] **[NEW]**
- `-f, --force` - Overwrite existing files

**Test Commands (Existing - Markdown):**
```bash
# Generate single file (Markdown - default)
arw generate index.html

# Generate from directory recursively (Markdown)
arw generate ./pages --recursive --output ./generated

# Force overwrite existing files
arw generate ./pages -r -f

# With custom output directory
arw generate ./src/pages -r --output ./public
```

**Test Commands (NEW - TOON Format):**
```bash
# Generate single file as TOON
arw generate index.html --output-format toon

# Generate directory as TOON
arw generate ./pages --recursive --output-format toon

# Generate with both formats (run twice)
arw generate index.html --output-format markdown
arw generate index.html --output-format toon

# TOON with custom output
arw generate ./pages -r --output-format toon --output ./toon-output

# Force overwrite TOON files
arw generate ./pages -r --output-format toon -f
```

**Expected Output:**
- **Markdown:** Creates `.llm.md` files
- **TOON:** Creates `.llm.toon` files
- Console output showing generation progress

---

### 3. `arw validate` - Validate Implementation

**Synopsis:**
```bash
arw validate [OPTIONS]
```

**Arguments:**
- `--path <PATH>` - Site root directory
- `--strict` - Enable deep consistency validation
- `-f, --fix` - Auto-fix issues (planned)

**Test Commands:**
```bash
# Basic validation
arw validate

# Validate specific path
arw validate --path ./my-site

# Strict mode with deep consistency checks
arw validate --strict

# Validate path with strict mode
arw validate --path ./test-site --strict
```

**Expected Output:**
- Validation status for llms.txt
- Validation status for robots.txt
- Validation status for sitemap.xml
- Chunk consistency checks (if --strict)
- Machine view file existence checks

---

### 4. `arw robots` - Generate robots.txt

**Synopsis:**
```bash
arw robots [OPTIONS]
```

**Arguments:**
- `--manifest <FILE>` - Input manifest file (default: `llms.txt`)
- `--output <FILE>` - Output file (default: `robots.txt`)

**Test Commands:**
```bash
# Generate from default llms.txt
arw robots

# Specify manifest and output
arw robots --manifest ./public/llms.txt --output ./public/robots.txt

# Generate to custom location
arw robots --output ./custom-robots.txt
```

**Expected Output:**
- `robots.txt` file with AI-agent awareness
- Policy-driven crawler rules
- ARW discovery hints

---

### 5. `arw sitemap` - Generate Sitemaps

**Synopsis:**
```bash
arw sitemap [OPTIONS]
```

**Arguments:**
- `--output <FILE>` - Output file (auto-detects format from extension)
- `--depth <N>` - Maximum crawl depth (default: 5)
- `--base-url <URL>` - Base URL for the site

**Test Commands:**
```bash
# Generate XML sitemap
arw sitemap --output sitemap.xml --base-url https://example.com

# Generate JSON sitemap
arw sitemap --output sitemap.llm.json --base-url https://example.com

# Custom depth
arw sitemap --output sitemap.xml --base-url https://example.com --depth 10
```

**Expected Output:**
- **XML:** Standard sitemap.xml format
- **JSON:** ARW sitemap.llm.json format with chunk metadata

---

### 6. `arw watch` - Watch Mode

**Synopsis:**
```bash
arw watch [OPTIONS]
```

**Arguments:**
- `--path <PATH>` - Directory to watch (default: `.`)
- `--generate` - Auto-generate machine views on HTML changes
- `--validate` - Auto-validate on llms.txt changes

**Test Commands:**
```bash
# Basic watch mode
arw watch --path .

# Watch and auto-generate
arw watch --generate

# Watch with both features
arw watch --generate --validate

# Watch specific directory
arw watch --path ./public --generate --validate
```

**Expected Output:**
- Real-time file system monitoring
- Auto-regeneration notifications
- Console status updates

---

### 7. `arw actions` - Manage Actions

**Synopsis:**
```bash
arw actions [OPTIONS]
```

**Arguments:**
- `--test` - Test action endpoints
- `--action-id <ID>` - Filter by specific action ID

**Test Commands:**
```bash
# List all actions
arw actions

# Test all action endpoints
arw actions --test

# Test specific action
arw actions --test --action-id create_order
```

**Expected Output:**
- List of actions from manifest
- Endpoint details (URL, method, auth, scopes)
- Test results (if --test flag used)

---

### 8. `arw serve` - Development Server

**Synopsis:**
```bash
arw serve [OPTIONS]
```

**Arguments:**
- `--path <PATH>` - Site root directory
- `-p, --port <PORT>` - Server port (default: 3000)
- `--watch` - Enable hot reload
- `--open` - Open browser automatically

**Test Commands:**
```bash
# Start server on default port
arw serve

# Custom port with hot reload
arw serve --port 8080 --watch

# Auto-open browser
arw serve --open

# All options
arw serve --path ./public --port 3000 --watch --open
```

**Expected Output:**
- Server running on specified port
- Hot reload status (if enabled)
- URL to access server

---

### 9. `arw scan` - Scan Website

**Synopsis:**
```bash
arw scan <URL> [OPTIONS]
```

**Arguments:**
- `<URL>` - Website URL to scan (required)
- `--depth <N>` - Maximum crawl depth (default: 3)
- `--output <DIR>` - Output directory for generated files
- `-n, --dry-run` - Don't generate files, analysis only

**Test Commands:**
```bash
# Basic scan
arw scan https://example.com

# Scan with custom depth
arw scan https://example.com --depth 5

# Scan with output directory
arw scan https://example.com --output ./arw-files

# Dry run (analysis only)
arw scan https://example.com --dry-run
```

**Expected Output:**
- Site analysis report
- Discovered pages and structure
- Generated ARW files (unless --dry-run)

---

### 10. `arw policy` - Manage Policies

**Synopsis:**
```bash
arw policy [OPTIONS]
```

**Arguments:**
- `--path <PATH>` - Site root directory
- `--template <NAME>` - Use template (ecommerce, documentation, blog)
- `--edit` - Edit existing policy interactively

**Test Commands:**
```bash
# Create policy from template
arw policy --template ecommerce

# Create documentation policy
arw policy --template documentation

# Edit existing policy
arw policy --edit

# Custom path with template
arw policy --path ./my-site --template blog
```

**Expected Output:**
- `policy.json` file created/modified
- Template-based policy configuration

---

## TOON-Specific Testing Scenarios

### Scenario 1: Basic TOON Generation
```bash
# Create test HTML file
echo '<h1>Test Page</h1><p>This is a test.</p>' > test.html

# Generate TOON
arw generate test.html --output-format toon

# Verify output
cat test.llm.toon
```

**Expected:** Valid TOON format with MachineView structure

---

### Scenario 2: TOON vs Markdown Comparison
```bash
# Generate both formats
arw generate index.html --output-format markdown
arw generate index.html --output-format toon

# Compare file sizes
ls -lh index.llm.md index.llm.toon

# Compare content
cat index.llm.md
cat index.llm.toon
```

**Expected:** Both files created, TOON should be 15-20% smaller

---

### Scenario 3: Recursive TOON Generation
```bash
# Create test directory structure
mkdir -p test-pages/{products,blog,docs}
echo '<h1>Product</h1>' > test-pages/products/keyboard.html
echo '<h1>Blog Post</h1>' > test-pages/blog/article.html
echo '<h1>Documentation</h1>' > test-pages/docs/guide.html

# Generate all as TOON
arw generate ./test-pages --recursive --output-format toon

# Verify all TOON files created
find test-pages -name "*.llm.toon"
```

**Expected:** 3 TOON files created in respective directories

---

### Scenario 4: TOON with Chunks
```bash
# Create HTML with chunk markers
cat > chunked.html << 'EOF'
<h1>Main Title</h1>
<section data-chunk-id="intro">
  <h2>Introduction</h2>
  <p>This is the intro.</p>
</section>
<section data-chunk-id="details">
  <h2>Details</h2>
  <p>More details here.</p>
</section>
EOF

# Generate TOON
arw generate chunked.html --output-format toon

# Verify chunks in output
grep -A 5 "chunks:" chunked.llm.toon
```

**Expected:** TOON file with separate chunks for intro and details

---

### Scenario 5: TOON Validation
```bash
# Generate TOON files
arw generate ./pages --recursive --output-format toon

# Run validation
arw validate --strict

# Check for errors
echo $?
```

**Expected:** Validation passes, exit code 0

---

## Test Matrix

### Format Combinations
| Input Format | Output Format | Command | Expected Output File |
|-------------|---------------|---------|---------------------|
| HTML | Markdown | `arw generate test.html` | `test.llm.md` |
| HTML | TOON | `arw generate test.html --output-format toon` | `test.llm.toon` |
| Markdown | Markdown | `arw generate test.md -i markdown` | `test.llm.md` |

### Flag Combinations
```bash
# All generate flags together
arw generate ./src -r -f --output-format toon --output ./dist

# Minimal command
arw generate index.html

# Maximum verbosity (with watch)
arw serve --path ./public --port 8080 --watch --open
```

---

## Testing Checklist

### Basic Functionality
- [ ] `arw init` creates proper directory structure
- [ ] `arw generate` creates `.llm.md` files (markdown)
- [ ] `arw generate --output-format toon` creates `.llm.toon` files
- [ ] `arw validate` reports validation status
- [ ] `arw robots` generates robots.txt
- [ ] `arw sitemap` generates XML and JSON sitemaps

### TOON-Specific
- [ ] TOON output has valid MachineView structure
- [ ] TOON version field is "1.0"
- [ ] TOON title extraction works
- [ ] TOON content blocks (Heading, Paragraph, List, Code)
- [ ] TOON chunks generated from headings
- [ ] TOON metadata includes source, timestamp, format
- [ ] TOON files are 15-20% smaller than Markdown

### Edge Cases
- [ ] Empty HTML generates valid TOON
- [ ] HTML with special characters (quotes, backslashes)
- [ ] Unicode and emoji support (🚀, ©, ®)
- [ ] Nested HTML structures
- [ ] Multiple headings create multiple chunks
- [ ] Missing title handles gracefully

### Integration
- [ ] Recursive generation works with TOON
- [ ] Force overwrite works with TOON files
- [ ] Custom output directory works
- [ ] Watch mode detects TOON file changes
- [ ] Validation detects TOON files

---

## Performance Testing

### File Size Comparison
```bash
# Generate both formats for same file
arw generate large-page.html --output-format markdown
arw generate large-page.html --output-format toon

# Compare sizes
ls -lh large-page.llm.md large-page.llm.toon
```

### Generation Speed
```bash
# Time markdown generation
time arw generate ./pages -r --output-format markdown

# Time TOON generation
time arw generate ./pages -r --output-format toon
```

**Expected:** TOON should be within 10-15% of Markdown generation time

---

## Error Cases to Test

### Invalid Commands
```bash
# Invalid output format
arw generate test.html --output-format invalid
# Expected: Error message about valid formats

# Missing required argument
arw generate
# Expected: Error about missing SOURCE argument

# Invalid path
arw validate --path /nonexistent/path
# Expected: Error about path not found
```

### Malformed HTML
```bash
# Test with broken HTML
echo '<h1>Unclosed heading' > broken.html
arw generate broken.html --output-format toon
# Expected: Should handle gracefully, generate valid TOON
```

---

## Unit Test Commands

```bash
# Run all library tests
cargo test --lib

# Run TOON-specific tests
cargo test --lib test_to_toon

# Run with verbose output
cargo test --lib -- --nocapture

# Run specific test
cargo test --lib test_to_toon_basic_html

# Run integration tests
cargo test --test toon_workflow_test
```

---

## Example Test Session

```bash
# 1. Build
cd packages/cli
cargo build --release

# 2. Create test environment
mkdir -p /tmp/arw-test
cd /tmp/arw-test

# 3. Initialize
arw init --yes

# 4. Create test HTML
cat > test.html << 'EOF'
<h1>Product Page</h1>
<section data-chunk-id="specs">
  <h2>Specifications</h2>
  <ul>
    <li>Price: $79.99</li>
    <li>Stock: In Stock</li>
  </ul>
</section>
EOF

# 5. Generate both formats
arw generate test.html --output-format markdown
arw generate test.html --output-format toon

# 6. Compare outputs
echo "=== MARKDOWN ==="
cat test.llm.md
echo ""
echo "=== TOON ==="
cat test.llm.toon
echo ""

# 7. Validate
arw validate --strict

# 8. Check file sizes
ls -lh test.llm.*

# 9. Clean up
cd -
rm -rf /tmp/arw-test
```

---

## Quick Copy-Paste Test Suite

```bash
#!/bin/bash
# Quick TOON integration test

# Setup
cd packages/cli
cargo build --release
mkdir -p /tmp/toon-test && cd /tmp/toon-test

# Test 1: Basic TOON generation
echo '<h1>Test</h1><p>Content</p>' > test.html
arw generate test.html --output-format toon
test -f test.llm.toon && echo "✓ Test 1 passed" || echo "✗ Test 1 failed"

# Test 2: Markdown generation (default)
arw generate test.html
test -f test.llm.md && echo "✓ Test 2 passed" || echo "✗ Test 2 failed"

# Test 3: Recursive TOON generation
mkdir pages && echo '<h1>Page 1</h1>' > pages/page1.html
arw generate pages -r --output-format toon
test -f pages/page1.llm.toon && echo "✓ Test 3 passed" || echo "✗ Test 3 failed"

# Test 4: TOON with chunks
cat > chunked.html << 'EOF'
<h1>Main</h1>
<section data-chunk-id="intro"><h2>Intro</h2></section>
EOF
arw generate chunked.html --output-format toon
grep -q "chunks:" chunked.llm.toon && echo "✓ Test 4 passed" || echo "✗ Test 4 failed"

# Cleanup
cd - && rm -rf /tmp/toon-test
echo "Test suite complete"
```

---

## Help Commands

```bash
# Main help
arw --help

# Command-specific help
arw generate --help
arw validate --help
arw init --help

# Version
arw --version
```

---

**Notes:**
- All commands assume you're in the project root or have `arw` in PATH
- Use `./target/release/arw` if running from packages/cli directory
- Replace `arw` with `cargo run --` for development testing
- All file paths should use absolute paths or be relative to current directory
