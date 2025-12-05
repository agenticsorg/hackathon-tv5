# ARW CLI - Complete Reference

> **Agent-Ready Web CLI** - Make your website accessible to AI agents

Version: 0.1.0
License: MIT
Repository: https://github.com/nolandubeau/agent-ready-web

---

## Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Commands](#commands)
  - [arw init](#arw-init)
  - [arw build](#arw-build)
  - [arw generate](#arw-generate)
  - [arw sitemap](#arw-sitemap)
  - [arw robots](#arw-robots)
  - [arw validate](#arw-validate)
  - [arw watch](#arw-watch)
  - [Other Commands](#other-commands)
- [Workflows](#workflows)
- [File Architecture](#file-architecture)

---

## Quick Start

```bash
# 1. Initialize ARW in your site directory
cd my-website
arw init

# 2. Build all ARW files from llms.txt
arw build

# 3. Validate your implementation
arw validate --strict
```

**That's it!** You now have:
- `.well-known/arw-manifest.json` (discovery router)
- `.well-known/arw-policies.json` (usage policies)
- `.well-known/arw-content-index.json` (content catalog)
- `sitemap.xml` (search engine sitemap)
- `llms.txt` (primary source of truth)

---

## Architecture

### Single Source of Truth

ARW follows the **"llms.txt is the single source of truth"** principle:

```
llms.txt (YAML - human-editable, canonical source)
   ↓
   ├─→ llms.json (JSON mirror - exact structural match)
   ├─→ .well-known/arw-manifest.json (JSON discovery router)
   ├─→ .well-known/arw-policies.json (JSON policies)
   ├─→ .well-known/arw-content-index.json (JSON content catalog)
   ├─→ sitemap.xml (XML sitemap with priorities from llms.txt)
   └─→ robots.txt (crawl rules from llms.txt policies)

.arw/config.yaml (CLI preferences only - NOT site info)
```

**Key principle**: Edit `llms.txt`, run `arw build`, done!

**Why both YAML and JSON?**
- `llms.txt` (YAML): Human-friendly editing, comments allowed, follows robots.txt convention
- `llms.json` (JSON): Machine-friendly parsing, strict typing, agent preference
- **Zero sync risk**: `llms.json` is auto-generated from `llms.txt` by `arw build`

### Discovery Flow

1. **AI agent discovers site**: `GET /.well-known/arw-manifest.json`
2. **Manifest provides links**:
   - `llms_txt`: `/llms.txt` (YAML canonical source)
   - `content_index`: `/.well-known/arw-content-index.json`
   - `policies`: `/.well-known/arw-policies.json`
   - `sitemap`: `/sitemap.xml`
3. **Agent loads resources** based on needs:
   - Agents preferring JSON may try `/llms.json` first
   - Agents supporting YAML can use `/llms.txt` directly
   - Both are structurally identical (auto-synced by CLI)

---

## Commands

### arw init

Initialize ARW structure for your site.

**Usage:**
```bash
arw init [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Site root directory (default: `public`)
- `-y, --yes` - Skip interactive prompts and use defaults

**What it does:**
1. Prompts for site information (name, description, homepage, contact)
2. Prompts for content policies (training, inference, attribution)
3. Creates `.arw/config.yaml` in application root (CLI preferences only)
4. Creates `public/llms.txt` as primary source of truth
5. Creates `public/index.llm.md` example machine view

**Example:**
```bash
# Interactive setup
arw init

# Quick setup with defaults
arw init --yes

# Initialize in specific directory (overriding default)
arw init --path ./custom-dir
```

**Output:**
```
.arw/config.yaml          # In application root
public/llms.txt           # In public directory
public/index.llm.md       # In public directory
```

---

### arw build

**⭐ NEW!** Build all ARW files from `llms.txt` in one command.

**Usage:**
```bash
arw build [OPTIONS]
```

**Options:**
- `-s, --source <SOURCE>` - Site root directory containing llms.txt (default: `public`)
- `-b, --base-url <BASE_URL>` - Base URL for the site (defaults to homepage in llms.txt)

**What it does:**
1. Reads `llms.txt` manifest (YAML)
2. Generates `llms.json` (JSON mirror of llms.txt)
3. Generates `.well-known/arw-manifest.json` (discovery router)
4. Generates `.well-known/arw-policies.json` (from policies section)
5. Generates `.well-known/arw-content-index.json` (from content section)
6. Generates `sitemap.xml` (with priorities from llms.txt)

**Example:**
```bash
# Build all files
arw build

# Build with custom base URL
arw build --base-url https://mysite.com

# Build from specific directory (overriding default)
arw build --source ./custom-dir
```

**Output:**
```
llms.json                            (JSON mirror)
.well-known/arw-manifest.json        (723 bytes)
.well-known/arw-policies.json        (376 bytes)
.well-known/arw-content-index.json   (253 bytes)
sitemap.xml                          (varies)
```

**Why use this?**
- **One command** instead of running sitemap, robots, etc. separately
- **Consistency**: All files generated from same source
- **Fast**: Generates everything in ~1 second
- **Links files**: Manifest includes correct URLs to all generated files

---

### arw generate

Generate machine views (`.llm.md`) from HTML files.

**Usage:**
```bash
arw generate <SOURCE> [OPTIONS]
```

**Options:**
- `-o, --output <OUTPUT>` - Output directory for machine views
- `-r, --recursive` - Process directories recursively
- `-f, --format <FORMAT>` - Input format: html, markdown, auto (default: auto)
- `--force` - Force overwrite existing files

**Example:**
```bash
# Convert single file
arw generate index.html

# Convert entire directory
arw generate ./pages --recursive

# Custom output directory
arw generate ./src --output ./public --recursive
```

---

### arw sitemap

Generate `sitemap.xml` from llms.txt.

**Usage:**
```bash
arw sitemap [OPTIONS]
```

**Options:**
- `--source <SOURCE>` - Site URL or local path (default: `public`)
- `-o, --output <OUTPUT>` - Output file path (default: `sitemap.llm.json`)
- `-d, --depth <DEPTH>` - Maximum crawl depth (default: 5)
- `-b, --base-url <BASE_URL>` - Base URL for the site

**How it works:**
1. Looks for `llms.txt` in source directory
2. If found: Uses `content[].priority` values (high → 1.0, medium → 0.8, low → 0.5)
3. If not found: Fallback to file scanning with default priority 0.5

**Example:**
```bash
# Generate from llms.txt
arw sitemap --output sitemap.xml --base-url https://example.com

# Fallback to file scanning
arw sitemap --output sitemap.xml
```

---

### arw robots

Generate `robots.txt` from llms.txt policies.

**Usage:**
```bash
arw robots [OPTIONS]
```

**Options:**
- `-m, --manifest <MANIFEST>` - Path to llms.txt manifest (default: `public/llms.txt`)
- `-o, --output <OUTPUT>` - Output file path (default: `public/robots.txt`)

**How it works:**
- Reads `policies.training.allowed` from llms.txt
- If `false`: Blocks AI training bots (GPTBot, CCBot, etc.)
- If `true`: Allows AI training bots
- Always allows standard web crawlers (Googlebot, Bingbot, etc.)

**Example:**
```bash
# Generate robots.txt
arw robots

# Custom paths
arw robots --manifest ./public/llms.txt --output ./public/robots.txt
```

---

### arw validate

Validate ARW implementation for compliance.

**Usage:**
```bash
arw validate [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Site root directory (default: `public`)
- `-s, --strict` - Enable strict validation mode
- `-f, --fix` - Attempt to auto-fix common issues

**What it validates:**
- `llms.txt` syntax and schema
- Required fields present
- Valid email addresses
- Priority values (high/medium/low)
- Chunk IDs uniqueness
- File references exist

**Example:**
```bash
# Basic validation
arw validate

# Strict validation
arw validate --strict

# Validate and auto-fix
arw validate --fix
```

---

### arw watch

Watch for file changes and auto-regenerate.

**Usage:**
```bash
arw watch [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Directory to watch (default: `public`)
- `-g, --generate` - Auto-generate machine views from HTML changes
- `-V, --validate` - Auto-validate llms.txt on changes

**Example:**
```bash
# Watch and auto-generate
arw watch --generate

# Watch and validate
arw watch --validate

# Both
arw watch --generate --validate
```

---

### Other Commands

#### arw serve
Start development server for testing.

```bash
arw serve [--port 3000] [--watch] [--open]
```

#### arw scan
Scan and analyze a website for ARW implementation.

```bash
arw scan <URL> [--depth 2] [--output report.json]
```

#### arw policy
Manage policy.json configuration.

```bash
arw policy [--template ecommerce] [--edit]
```

#### arw actions
Manage and test actions (ARW-3).

```bash
arw actions [--test] [--action-id create-order]
```

---

## Workflows

### Initial Setup

```bash
# 1. Initialize
arw init

# 2. Edit llms.txt (add your pages)
nano public/llms.txt

# 3. Build all files
arw build

# 4. Validate
arw validate --strict
```

### Adding New Pages

```bash
# 1. Edit llms.txt (add new content item with priority)
nano public/llms.txt

# 2. Generate machine view
arw generate public/new-page.html --output public/

# 3. Rebuild all files
arw build

# 4. Validate
arw validate
```

### Development Workflow

```bash
# Terminal 1: Watch for changes
arw watch --generate --validate

# Terminal 2: Edit your files
# ... make changes ...

# Files auto-regenerate on save!
```

### Deployment Workflow

```bash
# 1. Update llms.txt with latest content
nano public/llms.txt

# 2. Build all ARW files
arw build

# 3. Validate before deploying
arw validate --strict

# 4. Deploy to production
# ... your deployment process ...
```

---

## File Architecture

### Generated Files

| File | Generated By | Source | Purpose |
|------|-------------|--------|---------|
| `llms.txt` | `arw init` | User input | Primary source of truth (YAML) |
| `llms.json` | `arw build` | llms.txt | JSON mirror (exact structural match) |
| `.well-known/arw-manifest.json` | `arw build` | llms.txt | Discovery router (JSON) |
| `.well-known/arw-policies.json` | `arw build` | llms.txt policies | Usage policies (JSON) |
| `.well-known/arw-content-index.json` | `arw build` | llms.txt content | Content catalog (JSON) |
| `sitemap.xml` | `arw build` | llms.txt content | Search engine sitemap |
| `robots.txt` | `arw robots` | llms.txt policies | Crawl rules |
| `*.llm.md` | `arw generate` | HTML files | Machine views |

### llms.txt Structure

```yaml
version: 1.0
profile: ARW-1

site:
  name: "My Website"
  description: "Website description"
  homepage: "https://example.com"
  contact: "ai@example.com"

content:
  - url: /
    machine_view: /index.llm.md
    purpose: homepage
    priority: high         # ← Used by sitemap!

  - url: /about
    machine_view: /about.llm.md
    purpose: information
    priority: medium

policies:
  training:
    allowed: false         # ← Used by robots.txt!
  inference:
    allowed: true
  attribution:
    required: true
```

### .well-known/arw-manifest.json Structure

```json
{
  "$schema": "https://arw.dev/schemas/arw-manifest.schema.json",
  "version": "1.0",
  "profile": "ARW-1",
  "site": {
    "name": "My Website",
    "description": "...",
    "homepage": "https://example.com",
    "contact": "ai@example.com"
  },
  "discovery": {
    "llms_txt": "/llms.txt",
    "content_index": "/.well-known/arw-content-index.json",
    "policies": "/.well-known/arw-policies.json",
    "sitemap": "/sitemap.xml"
  },
  "capabilities": {
    "machine_views": true,
    "chunking": true,
    "actions": false,
    "oauth": false,
    "protocols": []
  },
  "metadata": {
    "last_updated": "2025-01-08T...",
    "generator": "arw-cli",
    "spec_version": "1.0"
  }
}
```

---

## Best Practices

### 1. Edit llms.txt, Not Generated Files

❌ **Wrong:**
```bash
nano public/.well-known/arw-manifest.json  # Manual editing
```

✅ **Right:**
```bash
nano public/llms.txt  # Edit source
arw build             # Regenerate all files
```

### 2. Use Priorities Consistently

```yaml
content:
  - url: /
    priority: high      # Homepage

  - url: /docs
    priority: high      # Important documentation

  - url: /blog
    priority: medium    # Regular content

  - url: /archive
    priority: low       # Old content
```

### 3. Validate Before Deploying

```bash
# In CI/CD pipeline
arw validate --strict || exit 1
```

### 4. Version Control

**.gitignore:**
```
.arw/                               # CLI config (optional, user-specific)
public/llms.json                    # Generated JSON mirror
public/*.llm.md                     # Generated machine views
public/sitemap.xml                  # Generated sitemap
public/.well-known/*.json           # Generated discovery files
```

**Committed:**
```
public/llms.txt                     # Source of truth - ALWAYS commit!
```

**Note**: `llms.json` is gitignored because it's auto-generated from `llms.txt`. Only commit the canonical YAML source.

**Note**: The `.arw/` directory contains CLI preferences that are user-specific. You may choose to commit it if you want to share CLI configuration across your team, or gitignore it for per-developer settings.

---

## Troubleshooting

### "llms.txt not found"

```bash
# Run init first
arw init
```

### "Validation errors"

```bash
# Check errors with strict mode
arw validate --strict

# Auto-fix common issues
arw validate --fix
```

### "Priority not working in sitemap"

Make sure you:
1. Set `priority` in llms.txt content items
2. Run `arw build` (not `arw sitemap` alone)
3. Use valid priority values: `high`, `medium`, `low`

---

## Support

- **Documentation**: https://github.com/nolandubeau/agent-ready-web
- **Spec**: `spec/ARW-v1.0.md`
- **Issues**: https://github.com/nolandubeau/agent-ready-web/issues

---

**🎉 You're ready to make your website Agent-Ready!**
