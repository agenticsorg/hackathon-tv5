# Agent-Ready Web (ARW) — Specification v0.1-draft

**Specification ID:** ARW-0.1-draft\
**Status:** 🧩 Editor's Draft (Work in Progress)\
**Stage:** Pre-Standard Proposal\
**Date:** 2025-11-12\
**Editor & Maintainer:** Nolan Dubeau (hello@nolandubeau.com)

**License:** Apache 2.0

**Repository:** [github.com/agent-ready-web](https://github.com/agent-ready-web)

**Feedback:** Open for community discussion via GitHub Issues & Pull Requests

> ⚠️ **Editor's Draft Notice**
> This document is an **early-stage working draft** published for transparency and public feedback.
> It **does not yet represent a finalized standard** and may change without notice.
> Implementers are encouraged to **experiment, test, and share input**,
> but should not rely on version stability or backward compatibility.

---

## Part I: Normative Specification

### 1. Overview

**Agent-Ready Web (ARW)** is a specification for making websites natively discoverable, interpretable, and operable to AI agents and autonomous systems - while preserving the human web experience and SEO integrity.

ARW provides the **infrastructure layer for agent-web interoperability**, combining the discoverability of llms.txt, the access control of robots.txt, and the semantics of Schema.org into a unified framework.

**Key Benefits:**

- **85% token reduction** – Machine views average 8KB vs 55KB HTML
- **10x faster discovery** – Structured manifests vs crawling
- **Native actions** – OAuth-enforced transactions
- **Observability** – Track agent behavior via AI-\* headers
- **SEO preserved** – Parallel views, no human impact

#### 1.1 Design Philosophy

ARW **extends** existing web standards—it does not replace them.

| Existing Standard  | ARW Role                                            |
| ------------------ | --------------------------------------------------- |
| **sitemap.xml**    | Source of content freshness & priority              |
| **robots.txt**     | Controls crawl access                               |
| **llms.txt/.json** | Declares agent-readable content, actions & policies |
| **HTTP Headers**   | Real-time observability & analytics                 |
| **Schema.org**     | Declarative semantics for actions                   |
| **OAuth 2.0**      | Secure user consent for agent-initiated operations  |

**Four Core Principles:**

1. **Web-native first** – Works over standard HTTP, no proprietary SDKs
2. **Progressive enhancement** – Implement incrementally without breakage
3. **Human parity** – Never degrade accessibility or SEO
4. **Efficiency & interoperability** – Optimize for token usage and agent performance

#### 1.2 The Agent-Web Interoperability Gap

AI agents are accessing the web at unprecedented scale:

- **ChatGPT Atlas:** 100M+ weekly users
- **Claude, Perplexity, Gemini:** Growing rapidly
- **Expected:** 40%+ of website traffic by 2025, 70% by 2027

**The Problem: No Standard Infrastructure**

Current state:

- ❌ Agents scrape inefficient HTML (55KB vs 8KB for Markdown)
- ❌ No structured discovery (agents must crawl entire sites)
- ❌ No action capabilities (agents can't complete transactions)
- ❌ No observability (publishers can't track agent behavior)
- ❌ No clear policies (ambiguity around training/inference)

**ARW's Solution: Infrastructure for the Agent Web**

ARW provides:

1. **Efficient Content Layer**

   - Machine views (`.llm.md`) reduce tokens by 85%
   - Semantic chunking for precise citations
   - No HTML parsing ambiguity

2. **Structured Discovery**

   - Single manifest (`/llms.txt`) lists all capabilities
   - 10x faster than crawling
   - Hierarchical organization

3. **Action Layer**

   - OAuth-enforced transactions (technically protected)
   - Schema.org semantics
   - Idempotent operations

4. **Observability Layer**

   - AI-\* headers for analytics
   - Agent identification and tracking
   - Performance monitoring

5. **Policy Declaration**
   - Machine-readable terms (advisory, like robots.txt)
   - Legal foundation and accountability
   - Basis for AI company commitments

**Two Enforcement Models:**

- **Technical enforcement** (OAuth actions): Cannot be bypassed
- **Advisory declarations** (policies): Provide observability, legal basis, and platform leverage

This dual approach recognizes that:

- Transactional operations can be technically protected
- Content access benefits from transparency and standardization
- Observability enables analytics and accountability
- Machine-readable policies provide legal clarity

#### 1.3 Example Scenario: CloudCart E-commerce Platform

Throughout this specification, we use **CloudCart** (https://cloudcart.example.com), a fictitious e-commerce platform, to demonstrate ARW implementation across different page types:

- **Product pages** (`/products/wireless-keyboard`)
- **Documentation** (`/docs/api/authentication`)
- **Support center** (`/support/shipping-returns`)
- **API endpoints** for actions like creating orders, submitting tickets

CloudCart wants to enable AI agents to:

1. Help users find and compare products efficiently
2. Answer technical questions from documentation
3. Submit support tickets on behalf of users (with OAuth consent)
4. Generate order summaries and recommendations

**Without ARW:**

- Agents scrape 55KB HTML per page (wasteful)
- No structured way to discover all products
- Cannot complete transactions (no OAuth actions)
- Zero visibility into agent traffic

**With ARW:**

- Agents fetch 8KB Markdown (85% reduction)
- Single `/llms.txt` manifest lists all capabilities
- OAuth actions enable transactions
- Full observability via AI-\* headers

---

### 2. Conformance Levels

Implementations MUST declare their conformance level in `/llms.txt` under `version` and `profile`.

| Profile             | Capabilities                                                          | Requirements                                          |
| ------------------- | --------------------------------------------------------------------- | ----------------------------------------------------- |
| **ARW-1 Discovery** | Basic discovery via `/llms.txt` + at least one `.llm.md`              | llms.txt, sitemap.xml, AI-Attribution header          |
| **ARW-2 Semantic**  | Adds chunking, link relations, rate limits, and attribution templates | All ARW-1 + chunk mapping + full AI-\* header suite   |
| **ARW-3 Actions**   | Adds OAuth actions + Schema.org `potentialAction`                     | All ARW-2 + OAuth + secure endpoints + idempotency    |
| **ARW-4 Protocol**  | Adds MCP/ACP/A2A discovery                                            | All ARW-3 + protocol section + advanced observability |

Implementers SHOULD expose a badge or header `AI-ARW-Level: n` where n is 1-4.

**Badge System:**

- Level 1: 🟢 Discovery Ready - `/llms.txt` + machine views
- Level 2: 🟢 Semantic Ready - Adds chunks & headers
- Level 3: 🟢 Action Ready - OAuth actions
- Level 4: 🟢 Protocol Ready - MCP/ACP/A2A interop

**Implementation Path:**

Most sites should start with **ARW-1** (2 hours implementation) and progressively enhance:

- ARW-1: Basic interoperability (80% of value)
- ARW-2: Analytics and optimization (15% of value)
- ARW-3: Transactional capabilities (4% of value)
- ARW-4: Advanced protocols (1% of value)

---

### 3. Discovery Architecture

#### 3.1 Overview: Layered Discovery System

ARW uses a **layered discovery system** built on RFC 8615 web standards to enable efficient agent-web interaction.

**Discovery Layers:**

1. **Primary Entrypoint** – `/.well-known/arw-manifest.json` (RFC 8615 standard location)
2. **Dual Canonical Formats** – `/llms.json` (machine parsing) and `/llms.txt` (human editing)
3. **Discovery Hints** – `robots.txt` hints for non-standard architectures

**Agent Discovery Flow:**

```
┌─────────────────────────────────────────────────────────┐
│ Step 1: Check RFC 8615 Standard Location               │
│ GET /.well-known/arw-manifest.json                      │
│ ✓ Found → Use as primary manifest                      │
│ ✗ Not Found → Continue to Step 2                       │
└─────────────────────────────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────┐
│ Step 2: Check Dual Canonical Formats                   │
│ GET /llms.json (preferred)                              │
│ GET /llms.txt (fallback)                                │
│ ✓ Found → Use either format                            │
│ ✗ Not Found → Continue to Step 3                       │
└─────────────────────────────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────┐
│ Step 3: Check Discovery Hints                          │
│ GET /robots.txt (check for arw-manifest: hint)         │
│ ✓ Found → Follow hint to custom location               │
│ ✗ Not Found → Site not ARW-enabled                     │
└─────────────────────────────────────────────────────────┘
```

**Manifest Purpose:**

All discovery formats declare:

- Machine-readable endpoints (`.llm.md`)
- Available actions & protocols
- Site policies & contact
- Conformance profile

**Why a layered system?**

- **RFC 8615 compliance** – Standard location for site metadata
- **Format flexibility** – Both JSON (machine) and YAML (human) are first-class
- **10x faster** than crawling entire site
- **Single source of truth** for agent capabilities
- **Progressive disclosure** – agents load only what they need
- **Graceful fallback** – works across different hosting architectures

#### 3.2 Primary Entrypoint: `/.well-known/arw-manifest.json`

Sites implementing ARW **SHOULD** expose the primary machine entrypoint:

- **Location:** `/.well-known/arw-manifest.json`
- **Standard:** RFC 8615 (Well-Known URIs for site-wide metadata)
- **Content-Type:** `application/json; charset=utf-8`
- **Format:** JSON representation of ARW manifest
- **Purpose:** Primary entrypoint for agent discovery

**Why RFC 8615 .well-known?**

- **Standardization** – RFC 8615 establishes `.well-known/` as the standard location for site metadata
- **Discoverability** – Agents know where to check first across all sites
- **Separation** – No routing conflicts with application paths
- **Security** – Can apply different security policies to `.well-known/`
- **Caching** – Can cache separately from application content
- **Future-proof** – Extensible for future ARW features

**Relationship to Other Formats:**

- `/.well-known/arw-manifest.json` → **Primary** canonical machine endpoint (RECOMMENDED)
- `/llms.json` → Alternative JSON format (backwards compatible)
- `/llms.txt` → YAML format for human editing (source of truth for YAML-first workflows)
- `/.well-known/llms` → Legacy fallback (301/302 redirect to implementer choice)

**Implementation Recommendations:**

- **JSON-first sites:** Generate `/llms.json`, copy to `/.well-known/arw-manifest.json`
- **YAML-first sites:** Write `/llms.txt`, generate both `/llms.json` and `/.well-known/arw-manifest.json`
- **Large sites:** Expose all three formats for maximum compatibility
- **Dynamic sites:** Generate JSON on-demand for `.well-known/` path

#### 3.3 Dual Canonical Formats

#### 3.4 Discovery Hints via `robots.txt`

**Optional Robots.txt Hints:**

For sites with non-standard architectures, ARW **MAY** be advertised in `robots.txt` using custom directives:

```
# robots.txt
User-agent: *
Disallow: /admin

# ARW Discovery Hints (optional)
arw-manifest: /llms.json
arw-level: 3
arw-contact: ai@example.com
```

**Directive Semantics:**

- `arw-manifest: <url>` - Location of ARW manifest (JSON or YAML)
- `arw-level: <1-4>` - Conformance level (ARW-1 through ARW-4)
- `arw-contact: <email>` - Contact for agent-related questions

**Agent Processing:**

1. Agents **SHOULD** check `robots.txt` for ARW hints during crawl
2. If `arw-manifest` found, use specified URL instead of default paths
3. Agents **MUST** still respect standard robots.txt crawl rules
4. Hints are **advisory** - agents SHOULD verify by fetching manifest

**Rationale:**

- Passive discovery (no extra requests for compliant crawlers)
- Consistent with robots.txt as "website metadata file"
- Helps large sites with custom manifest locations
- Optional - not required for conformance

**Examples:**

```
# Standard location
arw-manifest: /llms.json

# Custom location (large site with /ai/ subdirectory)
arw-manifest: /ai/manifest.json

# International site with language-specific manifests
arw-manifest: /en/llms.json
arw-manifest-es: /es/llms.json
arw-manifest-fr: /fr/llms.json
```

**Format Specifications:**

ARW manifests **MUST** be available in at least one canonical format. Both JSON and YAML have equal status as first-class formats:

1. **JSON Format**:
   - **Content-Type:** `application/json; charset=utf-8`
   - **Locations:**
     - `/.well-known/arw-manifest.json` (PRIMARY, RFC 8615 standard)
     - `/llms.json` (alternative root location)
   - **Use case:** Machine parsing, strict validation, agent compatibility
   - **When to use:** JSON-first workflows, automated generation, strict validation

2. **YAML Format** (`/llms.txt`):

   - **Content-Type:** `text/plain; charset=utf-8` (REQUIRED for compatibility)
   - **Location:** `/llms.txt` at domain root
   - **Use case:** Human editing, comment support, version control friendly
   - **Alternative:** `application/yaml; charset=utf-8` (MAY be used if Accept header explicitly requests it)
   - **Rationale:** `text/plain` prevents binary data corruption in AI agent web fetch tools
   - **When to use:** YAML-first workflows, manual editing by content teams

**Implementation Requirements:**

- Sites **MUST** expose at least one canonical format
- Sites **SHOULD** expose `/.well-known/arw-manifest.json` as primary entrypoint (RFC 8615)
- Sites **SHOULD** expose multiple formats for maximum compatibility
- All formats **MUST** contain semantically identical data (if multiple present)
- Sites **MAY** generate formats from a single source at build time or request time
- Sites **MAY** maintain formats manually if preferred

**Implementation Approaches:**

**Approach A: JSON-First (Recommended for new sites)**
1. Generate `/llms.json` from database or CMS
2. Copy to `/.well-known/arw-manifest.json` (primary)
3. Optionally generate `/llms.txt` for human readers

**Approach B: YAML-First (Recommended for manual editing)**
1. Manually edit `/llms.txt` (human-editable source)
2. Generate `/llms.json` and `/.well-known/arw-manifest.json`
3. Validate consistency across formats

**Discovery Priority for Agents:**

1. Try `/.well-known/arw-manifest.json` (RFC 8615 standard - RECOMMENDED)
2. Try `/llms.json` (alternative JSON location)
3. Try `/llms.txt` (YAML format)
4. Try `robots.txt` for hints (custom locations - see Section 3.4)

**Well-Known Fallback:**

- `/.well-known/llms` → 301/302 redirect to `/llms.txt` OR `/llms.json` (implementer choice)

**Format Design Rationale:**

- **RFC 8615** provides standard location across all websites
- **Dual canonical** status ensures format flexibility (no "legacy" format)
- **`text/plain`** for YAML prevents binary corruption in AI agent tools
- **`/llms.txt`** follows web conventions (robots.txt, security.txt, humans.txt)
- **YAML** supports comments and is human-friendly for version control
- **JSON** provides strict parsing and is preferred by agent frameworks
- **No forced tooling** - sites choose workflow that fits their team

#### 3.5 Complete Example (CloudCart)

```yaml
# Agent-Ready Web Discovery Manifest
# https://cloudcart.example.com/llms.txt

version: 0.1
profile: ARW-3

# Site Information
site:
  name: 'CloudCart'
  description: 'Modern e-commerce platform for electronics'
  homepage: 'https://cloudcart.example.com'
  contact: 'ai@cloudcart.example.com'

# Machine-Readable Content Endpoints
# Note: Last-modified dates are in sitemap.xml per existing standards
content:
  # Product catalog
  - url: /products/wireless-keyboard
    machine_view: /products/wireless-keyboard.llm.md
    purpose: product_information
    priority: high
    chunks:
      - id: product-summary
        heading: 'Product Overview'
        description: 'Name, price, SKU, availability'
      - id: product-specs
        heading: 'Technical Specifications'
        description: 'Detailed technical specs'
      - id: product-reviews
        heading: 'Customer Reviews'
        description: 'User ratings and feedback'

  # Documentation
  - url: /docs/api/authentication
    machine_view: /docs/api/authentication.llm.md
    purpose: technical_documentation
    priority: high
    chunks:
      - id: auth-overview
        heading: 'Authentication Overview'
      - id: oauth-flow
        heading: 'OAuth 2.0 Flow'
      - id: api-keys
        heading: 'API Key Management'

  # Support content
  - url: /support/shipping-returns
    machine_view: /support/shipping-returns.llm.md
    purpose: customer_support
    priority: medium
    chunks:
      - id: shipping-overview
        heading: 'Shipping Overview'
      - id: return-policy
        heading: 'Return Policy'

# Actions: OAuth-Protected Operations
actions:
  - id: add_to_cart
    name: 'Add to Cart'
    description: 'Add a product to the shopping cart'
    endpoint: /api/actions/add-to-cart
    method: POST
    auth: oauth2
    scopes: ['cart:write']
    schema: https://schema.org/BuyAction

  - id: create_order
    name: 'Create Order'
    description: 'Place a new order with products and shipping info'
    endpoint: /api/actions/create-order
    method: POST
    auth: oauth2
    scopes: ['orders:write']
    schema: https://schema.org/OrderAction

  - id: create_support_ticket
    name: 'Create Support Ticket'
    description: 'Submit a customer support request'
    endpoint: /api/actions/create-ticket
    method: POST
    auth: oauth2
    scopes: ['support:write']
    schema: https://schema.org/CreateAction

# Protocol Interoperability
protocols:
  - name: 'Model Context Protocol'
    type: mcp
    endpoint: /api/mcp/v1
    description: 'MCP server for product data and order management'

  - name: 'Agentic Commerce Protocol'
    type: acp
    endpoint: /api/acp/checkout
    description: 'ACP-compliant checkout flow'

  - name: 'Agent2Agent Protocol'
    type: a2a
    endpoint: /.well-known/agent
    description: 'Agent card for inter-agent collaboration'

# OAuth Configuration
oauth:
  authorization_url: /oauth/authorize
  token_url: /oauth/token
  scopes:
    - name: 'cart:write'
      description: 'Add and remove items from cart'
    - name: 'orders:write'
      description: 'Create and modify orders'
    - name: 'support:write'
      description: 'Create support tickets'

# Usage Policies (Advisory - provides observability and legal clarity)
policies:
  training:
    allowed: false
    note: 'Content may not be used for model training without license'
  inference:
    allowed: true
    restrictions: ['attribution_required', 'rate_limited']
  attribution:
    required: true
    format: link
    template: 'Information from CloudCart ({path})'
  rate_limits:
    authenticated: '100/min'
    unauthenticated: '20/min'
```

#### 3.6 Minimal Implementation (ARW-1)

Small sites can start with a minimal llms.txt:

```yaml
version: 0.1
profile: ARW-1

site:
  name: 'My Blog'
  homepage: 'https://myblog.example.com'
  # contact: 'ai@myblog.example.com'  # Optional

content:
  - url: /
    machine_view: /index.llm.md
    purpose: homepage

  - url: /about
    machine_view: /about.llm.md
    purpose: about_page

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
```

**Implementation time:** 2 hours for basic blog or documentation site

#### 3.7 JSON Format Example

The same minimal manifest in JSON format (can be auto-generated from `/llms.txt` or maintained directly):

```json
{
  "version": "0.1",
  "profile": "ARW-1",
  "site": {
    "name": "My Blog",
    "homepage": "https://myblog.example.com"
  },
  "content": [
    {
      "url": "/",
      "machine_view": "/index.llm.md",
      "purpose": "homepage"
    },
    {
      "url": "/about",
      "machine_view": "/about.llm.md",
      "purpose": "about_page"
    }
  ],
  "policies": {
    "training": {
      "allowed": false
    },
    "inference": {
      "allowed": true
    },
    "attribution": {
      "required": true
    }
  }
}
```

**Key Points:**

- Exact structural mirror of `/llms.txt` (if using YAML-first approach)
- Can be generated automatically by ARW CLI (`arw build`)
- Can be maintained directly (if using JSON-first approach)
- Should be served at both `/.well-known/arw-manifest.json` (primary) and `/llms.json`
- Agents can fetch from either location
- Same schema validation applies to both formats

#### 3.8 Platform Compatibility Considerations

ARW is designed to work across all hosting platforms and AI agent tools. However, implementers should be aware of platform-specific behaviors:

##### 3.8.1 Hosting Platform Quirks

**Vercel:**

- Serves custom mime types correctly via `vercel.json` headers config
- No automatic `.txt` → `text/plain` transformation (must configure)
- Supports dynamic routes for format negotiation

**Netlify:**

- Requires `_headers` file or `netlify.toml` for custom mime types
- May cache mime types aggressively (test after deployment)
- Supports redirect rules for well-known paths

**Cloudflare Pages:**

- Automatic mime type detection can override custom headers
- Use `_headers` file to force correct Content-Type
- Edge caching requires explicit cache headers

**GitHub Pages:**

- Limited mime type control (uses file extension only)
- **Recommendation:** Use `.json` extension for JSON manifests
- Static file serving only (no dynamic format negotiation)

**Apache:**

- Use `.htaccess` for mime type configuration
- AddType directive: `AddType text/plain .txt`
- Supports content negotiation via MultiViews

**Nginx:**

- Configure via `mime.types` or `location` blocks
- Example: `location /llms.txt { types { text/plain txt; } }`
- Supports `try_files` for format negotiation

##### 3.8.2 AI Agent Tool Limitations

**Known Compatibility Issues:**

| Tool/Platform     | Issue                           | Workaround                    |
| ----------------- | ------------------------------- | ----------------------------- |
| Claude WebFetch   | `application/yaml` → binary     | Use `text/plain` or JSON      |
| ChatGPT Browser   | Custom mime types trigger CORS  | Include proper CORS headers   |
| Perplexity        | Aggressive caching              | Use versioned URLs or ETags   |
| Generic HTTP libs | Custom `text/x-*` types flagged | Fallback to `text/plain`      |
| Proxied requests  | Content-Type may be stripped    | Include format in URL (.json) |

**Recommendations for Maximum Compatibility:**

1. **Prefer standard mime types:**

   - `/llms.txt` → `text/plain; charset=utf-8`
   - `/llms.json` → `application/json; charset=utf-8`
   - `/*.llm.md` → `text/markdown; charset=utf-8`

2. **Include format in URL when possible:**

   - `/.well-known/arw-manifest.json` (explicit format)
   - Better than relying on mime type alone

3. **Set CORS headers for agent access:**

   ```
   Access-Control-Allow-Origin: *
   Access-Control-Allow-Methods: GET, OPTIONS
   Access-Control-Expose-Headers: AI-*, Link
   ```

4. **Use robust caching strategy:**

   ```
   Cache-Control: public, max-age=3600, must-revalidate
   ETag: "manifest-v2"
   ```

5. **Test with multiple agents:**
   - Use `npx arw@alpha test-agent-compat <url>` (proposed tool)
   - Verify with Claude, ChatGPT, Perplexity simulators

##### 3.8.3 Mime Type Compatibility Matrix

**Verified Compatible Combinations:**

| File            | Mime Type             | Claude | ChatGPT | Perplexity | Gemini |
| --------------- | --------------------- | ------ | ------- | ---------- | ------ |
| `/llms.json`    | `application/json`    | ✅     | ✅      | ✅         | ✅     |
| `/llms.txt`     | `text/plain`          | ✅     | ✅      | ✅         | ✅     |
| `/llms.txt`     | `application/yaml`    | ❌     | ⚠️      | ✅         | ⚠️     |
| `/*.llm.md`     | `text/markdown`       | ✅     | ✅      | ✅         | ✅     |
| `/*.llm.md`     | `text/x-llm+markdown` | ⚠️     | ⚠️      | ⚠️         | ⚠️     |
| `/.well-known/` | `application/json`    | ✅     | ✅      | ✅         | ✅     |

Legend:

- ✅ Verified working
- ⚠️ Works but not recommended
- ❌ Confirmed broken

**Testing Methodology:**
To verify compatibility on your site:

1. Deploy ARW manifest
2. Test with multiple agent simulators
3. Check for binary data corruption
4. Verify JSON/YAML parsing
5. Monitor error logs for agent access issues

##### 3.8.4 Content Delivery Networks (CDNs)

**CDN Considerations:**

**Cloudflare CDN:**

- Cache HTML and JSON separately (use Vary header)
- Page Rules for custom caching on `/llms.*`
- May modify Content-Type (verify after setup)

**AWS CloudFront:**

- Requires explicit mime type mapping in S3 or Lambda@Edge
- Use metadata on S3 objects
- Custom error pages for well-known paths

**Fastly:**

- VCL snippets for dynamic Content-Type
- Edge logic for format negotiation
- Good support for custom headers

**Akamai:**

- Property Manager for mime type rules
- EdgeWorkers for dynamic responses
- Header modification may affect AI-\* headers

#### 3.9 Schema Definition

```yaml
version:
  type: string
  enum: ['0.1']  # Will become ['1.0'] when spec graduates from draft
profile:
  type: string
  enum: ['ARW-1', 'ARW-2', 'ARW-3', 'ARW-4']
site:
  type: object
  required: [name, homepage]
  properties:
    name: string
    description: string
    homepage: string (URL)
    contact: string (email, optional)
content:
  type: array
  items:
    required: [url, machine_view]
    properties:
      url: string (path or URL)
      machine_view: string (URL to .llm.md)
      purpose: string
      priority: enum [high, medium, low]
      chunks: array of chunk objects
actions:
  type: array
  items:
    required: [id, name, endpoint, method, auth]
    properties:
      id: string
      name: string
      description: string
      endpoint: string (URL)
      method: enum [GET, POST, PUT, PATCH, DELETE]
      auth: enum [oauth2, api_key, none]
      scopes: array of strings
      schema: string (Schema.org URL)
policies:
  type: object
  required: [training, inference, attribution]
  properties:
    training: object {allowed: boolean, note: string}
    inference: object {allowed: boolean, restrictions: array}
    attribution: object {required: boolean, format: string, template: string}
```

---

### 4. Machine Views (`.llm.md`)

#### 4.1 Purpose & Benefits

Markdown representation of content optimized for LLM ingestion. Each section is a _chunk_ annotated with `<!-- chunk:id -->`.

**Efficiency Benefits:**

- **85% token reduction** vs HTML (55KB HTML → 8KB Markdown typical)
- **3-5x faster** agent processing (no HTML parsing)
- **Semantic structure** for better understanding
- **No parsing ambiguity** - clean, structured format
- **Chunk addressability** for precise citations (40% token savings)
- **No clutter** - pure content, no navigation/ads

**Example Savings:**

- Product page: 55KB HTML → 8KB Markdown (85% reduction)
- Documentation: 120KB HTML → 18KB Markdown (85% reduction)
- Blog post: 38KB HTML → 6KB Markdown (84% reduction)

**Why this matters:**

- Lower costs for AI companies (fewer tokens)
- Faster responses for users (less processing)
- Better accuracy (structured content)
- Precise attribution (chunk-level citations)

#### 4.2 Normative Requirements

Machine views MUST use one of these Content-Type values:

**Recommended (priority order):**

1. `text/markdown; charset=utf-8` (RECOMMENDED - maximum compatibility)
2. `text/x-llm+markdown; charset=utf-8` (ACCEPTABLE - ARW-specific signal)
3. `text/plain; charset=utf-8` (ACCEPTABLE - universal fallback)

**Content Negotiation:**

- Servers **MAY** return different mime types based on `Accept` header
- If `Accept: text/markdown` → return `text/markdown`
- If `Accept: text/x-llm+markdown` → return `text/x-llm+markdown`
- If `Accept: */*` → return `text/markdown` (safest default)

**Additional Requirements:**

- Include `AI-Attribution` and `AI-Inference` headers
- Match chunk IDs between HTML (`data-chunk-id`) and Markdown (`<!-- chunk:id -->`)
- Support `ETag` and caching (`max-age >= 900`)

Machine views MAY:

- Expose fragments: `GET /page.llm.md#chunk-id` → returns that chunk only
- Support query parameter: `GET /page.llm.md?chunk=chunk-id`

#### 4.3 Examples

##### Product Page Example

**Machine View** (`/products/wireless-keyboard.llm.md`):

```markdown
<!-- chunk: product-summary -->

# Wireless Mechanical Keyboard

**Price:** $129.99
**Stock:** In Stock (47 units)
**Rating:** 4.9/5 stars (324 reviews)
**SKU:** KB-WL-001

Premium wireless mechanical keyboard with hot-swappable Cherry MX Brown switches, RGB backlighting, and dual connectivity (Bluetooth 5.0 + USB-C).

<!-- chunk: product-specs -->

## Technical Specifications

| Specification | Details                                           |
| ------------- | ------------------------------------------------- |
| Connection    | Bluetooth 5.0 & USB-C wired                       |
| Battery Life  | Up to 40 hours (backlight off), 20 hours (RGB on) |
| Switch Type   | Cherry MX Brown (tactile, non-clicky)             |
| Hot-Swappable | Yes - tool-free switch replacement                |
| Backlight     | RGB per-key programmable (16.8M colors)           |
| Layout        | Full-size 104-key (US ANSI)                       |
| Dimensions    | 440mm × 135mm × 38mm                              |
| Weight        | 980g                                              |

<!-- chunk: product-reviews -->

## Customer Reviews

**Overall Rating:** 4.9/5 stars (324 reviews)

**Top Positive Themes:**

- Excellent build quality and premium feel
- Smooth, responsive Cherry MX switches
- Long battery life exceeds expectations
- Beautiful RGB lighting with easy customization

**Common Concerns:**

- Premium price point
- Slightly heavy for travel
```

**Token comparison:**

- HTML: ~18,000 tokens (55KB)
- Machine view: ~2,700 tokens (8KB)
- **Savings: 85%**

**Required HTTP Headers:**

```http
Content-Type: text/x-llm+markdown; charset=utf-8
AI-Attribution: required; format=link; url=https://cloudcart.example.com/products/wireless-keyboard
AI-Rate-Limit: 100;window=60
AI-Training: disallowed
AI-Inference: allowed
Cache-Control: public, max-age=3600
ETag: "product-kb-wl-001-v5"
```

##### Documentation Page Example

**Machine View** (`/docs/api/authentication.llm.md`):

````markdown
<!-- chunk: auth-overview -->

# API Authentication

CloudCart API uses OAuth 2.0 for secure authentication and authorization. All API requests require a valid access token.

**Supported Flows:**

- Authorization Code Flow (recommended for web apps)
- Client Credentials Flow (for server-to-server)

**Token Lifetime:**

- Access tokens: 1 hour
- Refresh tokens: 30 days

<!-- chunk: oauth-flow -->

## OAuth 2.0 Authorization Flow

### Step 1: Authorization Request

Direct users to the authorization endpoint:

```http
GET /oauth/authorize?
  client_id=YOUR_CLIENT_ID&
  response_type=code&
  redirect_uri=YOUR_REDIRECT_URI&
  scope=orders:read orders:write&
  state=RANDOM_STATE_STRING
```

**Parameters:**

- `client_id`: Your application's client ID
- `response_type`: Always use `code`
- `redirect_uri`: Must match registered URI exactly
- `scope`: Space-separated list of requested scopes
- `state`: Random string to prevent CSRF attacks

### Step 2: Token Exchange

Exchange authorization code for access token:

```http
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=AUTHORIZATION_CODE&
client_id=YOUR_CLIENT_ID&
client_secret=YOUR_CLIENT_SECRET&
redirect_uri=YOUR_REDIRECT_URI
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "def50200b1c4d9e...",
  "scope": "orders:read orders:write"
}
```
````

#### 4.4 Chunk Addressability

Agents can request specific chunks via URL fragments or query parameters:

```http
# Request full machine view
GET /docs/api/authentication.llm.md
→ Returns entire document (~8KB)

# Request specific chunk (fragment)
GET /docs/api/authentication.llm.md#oauth-flow
→ Returns only the OAuth flow section (~2KB, 75% savings)

# Request specific chunk (query parameter)
GET /docs/api/authentication.llm.md?chunk=oauth-flow
→ Returns only the OAuth flow section (~2KB, 75% savings)
```

**Benefits:**

- **75% additional token savings** when agent needs specific section
- **Faster processing** (less content to parse)
- **Precise citations** (chunk-level attribution)
- **Better caching** (chunk responses can be cached separately)

**Server Implementation:**

```typescript
export async function GET(request: Request) {
  const url = new URL(request.url);
  const chunkId = url.hash.slice(1) || url.searchParams.get('chunk');

  const fullContent = await getFullMarkdown();

  if (chunkId) {
    const chunk = extractChunk(fullContent, chunkId);
    return new Response(chunk, {
      headers: {
        'Content-Type': 'text/x-llm+markdown; charset=utf-8',
        'AI-Chunk-Id': chunkId,
        'AI-Attribution':
          'required; format=link; url=https://cloudcart.example.com/docs/api/authentication',
        'Cache-Control': 'public, max-age=3600',
      },
    });
  }

  return new Response(fullContent, {
    headers: {
      'Content-Type': 'text/x-llm+markdown; charset=utf-8',
      'AI-Attribution':
        'required; format=link; url=https://cloudcart.example.com/docs/api/authentication',
      'Cache-Control': 'public, max-age=3600',
    },
  });
}
```

---

### 5. AI-Header Namespace

#### 5.1 Response Headers (Observability & Analytics)

| Header           | Purpose              | Example                                  |
| ---------------- | -------------------- | ---------------------------------------- |
| `AI-Attribution` | Attribution rules    | `required; format=link; url=https://...` |
| `AI-Training`    | Training permission  | `disallowed`                             |
| `AI-Inference`   | Inference permission | `allowed`                                |
| `AI-Rate-Limit`  | Rate window          | `100;window=60`                          |
| `AI-Chunk-Id`    | Current chunk        | `product-specs`                          |
| `AI-ARW-Level`   | Conformance level    | `3` (for ARW-3)                          |

**Purpose:**

These headers enable:

- **Analytics** – Track which agents access which content
- **Performance monitoring** – Measure response times, bandwidth
- **Policy enforcement** – Agents can respect declared policies
- **Attribution tracking** – Verify if agents provide proper attribution
- **Observability** – Publisher dashboards showing agent behavior

**Note on enforcement:**

These headers are **advisory** (like `robots.txt`). They provide:

- ✓ Observability data for analytics platforms
- ✓ Legal foundation (machine-readable ToS)
- ✓ Accountability (identify violators)
- ✓ Basis for AI company commitments

OAuth actions (Section 6) provide **technical enforcement** where needed.

**Usage Notes:**

- Headers MUST be present on all machine view responses
- Headers SHOULD be present on HTML responses (for observability)
- Values are semicolon-separated key-value pairs

#### 5.2 Request Headers (Agent Identification)

| Header            | Example           | Meaning            |
| ----------------- | ----------------- | ------------------ |
| `AI-Agent`        | `ClaudeAgent/1.0` | Agent identifier   |
| `AI-Purpose`      | `user_query`      | Request intent     |
| `AI-Request-ID`   | `req_abc123`      | Trace ID           |
| `AI-User-Consent` | `granted`         | User authorization |

**Observability Benefits:**

Agents SHOULD include these headers to enable:

- **Agent analytics** – Which AI companies access your site
- **Usage tracking** – Measure agent traffic separately from human traffic
- **Performance optimization** – A/B test content for different agents
- **Policy monitoring** – Identify agents respecting vs violating policies
- **Attribution verification** – Cross-reference requests with citations

**Example Request:**

```http
GET /products/wireless-keyboard.llm.md
Host: cloudcart.example.com
AI-Agent: ChatGPT-Bot/1.0
AI-Purpose: user_query
AI-Request-ID: 550e8400-e29b-41d4-a716-446655440000
AI-User-Consent: granted
```

**Example Analytics Use Cases:**

- "ChatGPT accounts for 60% of our agent traffic"
- "Claude agents prefer documentation over product pages"
- "Perplexity respects rate limits, Unknown-Bot-XYZ does not"
- "Agent traffic converts 15% higher on pages with machine views"

---

### 6. Declarative Actions

Actions enable agents to perform operations on behalf of users with explicit OAuth consent.

**Why actions are different:**

Actions are **technically enforced** via OAuth. Unlike content access (which is advisory), transactional operations require:

- User consent (OAuth authorization screen)
- Valid access token (short-lived, scoped)
- Server-side validation

This means actions **cannot be bypassed** – providing true publisher control over transactions.

#### 6.1 Schema.org Integration

**HTML on Product Page:**

```html
<script type="application/ld+json">
  {
    "@context": "https://schema.org",
    "@type": "Product",
    "name": "Wireless Mechanical Keyboard",
    "sku": "KB-WL-001",
    "offers": {
      "@type": "Offer",
      "price": "129.99",
      "priceCurrency": "USD",
      "availability": "https://schema.org/InStock"
    },
    "potentialAction": {
      "@type": "BuyAction",
      "name": "Add to Cart",
      "description": "Add this product to your shopping cart",
      "target": {
        "@type": "EntryPoint",
        "urlTemplate": "https://cloudcart.example.com/api/actions/add-to-cart",
        "httpMethod": "POST",
        "contentType": "application/json"
      },
      "object": {
        "@type": "Product",
        "sku": "KB-WL-001"
      },
      "instrument": {
        "@type": "OAuth2",
        "name": "CloudCart OAuth",
        "authorizationUrl": "https://cloudcart.example.com/oauth/authorize",
        "tokenUrl": "https://cloudcart.example.com/oauth/token",
        "scope": "cart:write"
      },
      "result": {
        "@type": "Order",
        "orderStatus": "https://schema.org/OrderProcessing"
      }
    }
  }
</script>
```

#### 6.2 OAuth 2.0 Requirements

Actions MUST:

- Use OAuth 2.0 for user consent
- Issue short-lived tokens (≤ 1 hour)
- Implement PKCE for public clients
- Support token refresh
- Log all action executions

Actions SHOULD:

- Support `Idempotency-Key` header
- Return clear error messages
- Provide action status endpoints
- Support webhook notifications

**Security Requirements:**

- All endpoints MUST use HTTPS
- Tokens MUST be short-lived (≤ 1 hour)
- Agents MUST obtain user consent before invocation
- Servers SHOULD validate request origin
- Rate limiting MUST be enforced

#### 6.3 Action Execution Flow

```
User → Agent: "Add the wireless keyboard to my cart"
Agent → Site: GET /llms.json or /llms.txt (discover actions)
Agent → User: Request authorization for cart:write scope
User → OAuth: Approve on consent screen
OAuth → Agent: Authorization code
Agent → OAuth: Exchange code for access token
Agent → API: POST /api/actions/add-to-cart (with token)
API → Agent: Success response with cart URL
Agent → User: "Added to cart. Ready to checkout?"
```

**Example Action Request:**

```http
POST /api/actions/add-to-cart HTTP/1.1
Host: cloudcart.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
Content-Type: application/json
Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000

{
  "product_id": "KB-WL-001",
  "quantity": 1
}
```

**Example Action Response:**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "success": true,
  "cart_id": "cart_abc123",
  "item_count": 3,
  "total": "$329.97",
  "checkout_url": "https://cloudcart.example.com/checkout?cart=cart_abc123"
}
```

---

### 7. Protocol Interoperability

ARW operates at the **web/HTTP presentation layer** and complements specialized agent protocols.

| Protocol | Purpose                                    | Example Endpoint     |
| -------- | ------------------------------------------ | -------------------- |
| **MCP**  | Model Context Protocol for structured data | `/api/mcp/v1`        |
| **ACP**  | Agentic Commerce Protocol for transactions | `/api/acp/checkout`  |
| **A2A**  | Agent-to-Agent cards for identity          | `/.well-known/agent` |

**Layered Architecture:**

```
┌─────────────────────────────────────────────┐
│  Specialized Protocols (MCP, ACP, A2A)      │  ← Workflow-specific
├─────────────────────────────────────────────┤
│  ARW (HTTP/Web Discovery Layer)             │  ← Universal discovery
├─────────────────────────────────────────────┤
│  HTTP/HTTPS                                  │  ← Transport
└─────────────────────────────────────────────┘
```

**Key Principle:** ARW provides universal HTTP/web discovery. Specialized protocols handle specific workflows.

Agents discovering these in `protocols` MAY negotiate directly after verifying policies.

#### 7.1 Model Context Protocol (MCP)

ARW advertises MCP servers via discovery files:

```yaml
protocols:
  - name: 'Product Catalog MCP Server'
    type: mcp
    endpoint: /api/mcp/v1
    schema: /api/mcp/schema.json
    description: 'Real-time product data and inventory'
```

#### 7.2 Agentic Commerce Protocol (ACP)

ARW provides product discovery; ACP handles checkout:

```yaml
protocols:
  - name: 'CloudCart Checkout'
    type: acp
    endpoint: /api/acp/checkout
    description: 'ACP-compliant checkout flow'
```

#### 7.3 Agent2Agent Protocol (A2A)

ARW enables HTTP-based discovery of agent capabilities:

```yaml
protocols:
  - name: 'Customer Support Agent'
    type: a2a
    endpoint: /.well-known/agent
    description: 'Agent card for support automation'
```

---

### 8. Security Considerations

- All endpoints MUST use HTTPS
- Sanitize Markdown to prevent injection attacks
- No PII in machine views unless authenticated
- Log agent requests for analytics and security
- Use Content Security Policy: `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'`
- Validate OAuth tokens on every action request
- Implement rate limiting per agent identifier
- Monitor for abuse patterns via observability headers

---

### 9. Link Relations & Discovery Headers

#### 9.1 HTML Head Links

```html
<link rel="llms-manifest" href="/llms.txt" />
<link rel="alternate" type="text/x-llm+markdown" href="/products/wireless-keyboard.llm.md" />
```

#### 9.2 HTTP Link Header

```
Link: </llms.txt>; rel="llms-manifest"
Link: </products/wireless-keyboard.llm.md>; rel="alternate"; type="text/x-llm+markdown"
```

#### 9.3 Well-Known Fallback

`/.well-known/llms` → 302 to `/llms.txt`

---

### 10. Validation & Badges

#### 10.1 CLI Validator (Reference)

```bash
npx arw@alpha validate https://example.com
```

Performs:

- YAML schema check
- Header verification
- Chunk ID integrity
- OAuth endpoint presence
- Conformance level validation

#### 10.2 Badges

Publishers MAY display compliance badges:

```markdown
![ARW-3 Action Ready](https://arw.dev/badges/arw-3-action-ready.svg)
```

| Level | Badge Label        | Meaning                     |
| ----- | ------------------ | --------------------------- |
| ARW-1 | 🟢 Discovery Ready | `/llms.txt` + machine views |
| ARW-2 | 🟢 Semantic Ready  | Adds chunks & headers       |
| ARW-3 | 🟢 Action Ready    | OAuth actions               |
| ARW-4 | 🟢 Protocol Ready  | MCP/ACP/A2A interop         |

---

### 11. Versioning & Governance

- **Current version:** v0.1-draft
- **Specification maintainer:** Nolan Dubeau
- **Governance:** Open specification with community input via GitHub
- **Feedback:** GitHub Discussions and Pull Requests welcome
- **License:** Apache 2.0

#### 11.1 Version Numbering

**Specification Version vs. Manifest Version:**

- **Specification version** (in header): `v0.1-draft`, `v1.0`, etc.
- **Manifest version** (in files): `version: 0.1`, `version: 1.0`, etc.
- Both MUST match during implementation

**Version Lifecycle:**

- `v0.x-draft` - Pre-release, breaking changes allowed
- `v1.0` - First stable release, semantic versioning begins
- `v1.x` - Backward-compatible additions only
- `v2.0` - Next major version (if breaking changes needed)

**Current Status:**

- Specification: `v0.1-draft` (this document)
- Manifest version: `0.1` (use in implementations)
- Upgrade path: Will become `1.0` when editor's draft graduates

**Semantic Versioning for Specifications:**

- **Major (2.0.0)** - Breaking changes requiring implementation updates
- **Minor (1.1.0)** - New features, backwards compatible
- **Patch (1.0.1)** - Clarifications, typos, non-breaking improvements

---

### 12. Normative Keywords

The key words **MUST**, **SHOULD**, **MAY**, **RECOMMENDED**, and **OPTIONAL** are to be interpreted as in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

---

## Part II: Implementation Guide

### 13. Understanding the Value Proposition

#### 13.1 Efficiency Gains

**For AI Companies:**

- 85% token reduction → Lower API costs
- 10x faster discovery → Faster responses to users
- No HTML parsing → Better accuracy
- Structured data → Improved understanding

**For Publishers:**

- Observability → Analytics on agent traffic
- Optimization → A/B test content for agents
- Monetization → Track conversions from agents
- Control → Technical enforcement of transactions

**For Users:**

- Faster responses (less processing time)
- More accurate answers (structured data)
- Transaction capabilities (OAuth actions)
- Better attribution (chunk-level citations)

#### 13.2 Comparison to Alternatives

| Approach        | Discovery      | Efficiency | Actions | Observability | SEO Impact |
| --------------- | -------------- | ---------- | ------- | ------------- | ---------- |
| **HTML only**   | Crawl required | 55KB avg   | None    | None          | N/A        |
| **llms.txt v0** | Basic manifest | Manual     | None    | None          | None       |
| **ARW**         | Full manifest  | 8KB avg    | OAuth   | Full headers  | None       |

**ARW is not competing with llms.txt** – it extends and enhances it with:

- Structured manifest (vs free-form text)
- Chunk-level addressability
- OAuth actions
- Observability headers
- Protocol interoperability

---

### 14. Migration Strategies

#### 14.1 From Static HTML

**Phase 1: Minimal ARW-1 (2 hours)**

1. Create `/llms.txt` with basic site info
2. Convert 3-5 key pages to `.llm.md` format
3. Add `AI-Attribution` headers

**Phase 2: Enhanced ARW-2 (1 week)**

4. Add chunk annotations to HTML and Markdown
5. Implement full `AI-*` header suite
6. Create complete `sitemap.xml`

**Phase 3: Actions ARW-3 (2-4 weeks)**

7. Implement OAuth 2.0 server
8. Create action endpoints
9. Add Schema.org `potentialAction` to HTML

**Phase 4: Protocols ARW-4 (ongoing)**

10. Integrate MCP/ACP/A2A as needed
11. Advanced observability and analytics

#### 14.2 From llms.txt v0

If you already have a basic llms.txt file:

1. **Add profile declaration:** `profile: ARW-1`
2. **Enhance with metadata:** Add chunks array to content entries
3. **Add policies:** Define training/inference/attribution rules
4. **Progressive enhancement:** Move to ARW-2, ARW-3, ARW-4 as needed

**Migration time:** 1-2 hours for existing llms.txt sites

---

### 15. Testing & Validation

#### 15.1 Tools

- **CLI Validator:** `npx arw@alpha validate https://example.com`
- **Inspector Web App:** Visual debugging of ARW implementations
- **DevTools Extension:** Chrome extension for live inspection

#### 15.2 Validation Checklist

**ARW-1 Discovery:**

- [ ] `/llms.txt` exists and is valid YAML
- [ ] At least one `.llm.md` file accessible
- [ ] `AI-Attribution` header present
- [ ] Profile declared in llms.txt

**ARW-2 Semantic:**

- [ ] All ARW-1 requirements met
- [ ] Content chunks properly annotated
- [ ] Full `AI-*` header suite implemented
- [ ] Rate limiting configured

**ARW-3 Actions:**

- [ ] All ARW-2 requirements met
- [ ] OAuth 2.0 endpoints functional
- [ ] At least one action endpoint
- [ ] Schema.org `potentialAction` in HTML

**ARW-4 Protocol:**

- [ ] All ARW-3 requirements met
- [ ] At least one protocol endpoint advertised
- [ ] Protocol endpoints functional

---

### 16. Best Practices

#### 16.1 Machine View Creation

- **Keep it semantic:** Use proper Markdown hierarchy
- **Stay focused:** Remove navigation, ads, irrelevant content
- **Be consistent:** Use same chunk IDs across pages of similar type
- **Think real-time:** Update machine views when HTML changes
- **Cache wisely:** Set appropriate cache headers (900-3600s)

#### 16.2 Policy Declaration

- **Be explicit:** Clearly state training vs. inference permissions
- **Be reasonable:** Balance protection with utility
- **Be transparent:** Explain your reasoning in notes
- **Be contactable:** Provide email for policy questions

**Note:** Policies are advisory (like robots.txt). They provide:

- Observability data for analytics
- Legal foundation (machine-readable ToS)
- Accountability when violations occur
- Basis for AI company commitments

#### 16.3 Action Design

- **User-centric:** Always require OAuth consent
- **Idempotent:** Support `Idempotency-Key` header
- **Clear errors:** Return helpful error messages
- **Status updates:** Provide endpoints to check action status

---

## Appendices

### Appendix A: Complete CloudCart Reference

See Part I sections for complete CloudCart implementation across all conformance levels.

### Appendix B: Schema Reference

See Section 3.9 for complete YAML schema definition.

### Appendix C: Header Reference

See Section 5 for complete `AI-*` header documentation.

### Appendix D: Glossary

| Term                 | Definition                                                                 |
| -------------------- | -------------------------------------------------------------------------- |
| **Agent**            | AI system that browses websites on behalf of users (ChatGPT, Claude, etc.) |
| **Machine View**     | Markdown representation optimized for LLM parsing (`.llm.md` files)        |
| **Chunk**            | Addressable content segment with unique ID                                 |
| **Action**           | OAuth-protected operation agents can invoke (technically enforced)         |
| **Profile**          | Conformance level (ARW-1 through ARW-4)                                    |
| **Policy**           | Machine-readable usage terms (advisory, provides observability)            |
| **Observability**    | Tracking agent traffic via `AI-*` headers for analytics                    |
| **Interoperability** | Ability for agents and websites to work together efficiently               |

### Appendix E: Enforcement Models

**Technical Enforcement (Actions):**

- OAuth 2.0 required
- Cannot be bypassed
- Server validates all requests
- Short-lived tokens
- User consent required

**Advisory Declaration (Policies):**

- Like robots.txt
- Provides observability
- Legal foundation
- Accountability
- Platform leverage

---

### Appendix F: Changelog

#### v0.1-draft Updates (2025-11-12)

This version incorporates critical amendments to improve agent compatibility and implementation clarity while maintaining backward compatibility.

**Amendment 1: Primary MIME Type Change (P0 - CRITICAL)**

- **Changed:** Primary MIME type for `/llms.txt` from `application/yaml` to `text/plain; charset=utf-8`
- **Reason:** Prevents binary data corruption in AI agent web fetch tools (confirmed in Claude WebFetch)
- **Location:** Section 3.3 (Dual Canonical Formats)
- **Impact:** REQUIRED for maximum compatibility
- **Alternative:** `application/yaml` MAY be used if Accept header explicitly requests it

**Amendment 2: Dual Canonical Status for /llms.json (P0 - CRITICAL)**

- **Status:** Already present in specification
- **Confirmed:** `/llms.json` has dual canonical status with `/llms.txt`
- **Location:** Section 3.3 (Dual Canonical Formats)
- **Both formats:** MUST contain semantically identical data if both present
- **Discovery priority:** JSON first for best compatibility

**Amendment 3: Primary Machine Entrypoint (P1 - HIGH)**

- **Status:** Already present in specification
- **Added:** `/.well-known/arw-manifest.json` as primary machine entrypoint
- **Location:** Section 3.2 (Primary Entrypoint)
- **Purpose:** Canonical "start here" URL for agent discovery
- **Discovery flow:** Well-known JSON → `/llms.json` → `/llms.txt` → robots.txt hints

**Amendment 4: Robots.txt Discovery Integration (P1 - HIGH)**

- **Status:** Already present in specification
- **Added:** Optional robots.txt hints specification
- **Location:** Section 3.4 (Discovery Hints via robots.txt)
- **Directives:** `arw-manifest`, `arw-level`, `arw-contact`
- **Purpose:** Passive discovery without additional HTTP requests

**Amendment 5: Version Reference Fixes (P1 - HIGH)**

- **Changed:** All examples from `version: 1.0` to `version: 0.1`
- **Locations:**
  - Section 3.5 (CloudCart complete example)
  - Section 3.6 (Minimal implementation)
  - Section 3.7 (JSON format example)
  - Section 3.9 (Schema definition)
  - Section 11.1 (Version numbering clarification)
- **Clarified:** Specification version (v0.1-draft) vs manifest version (0.1) distinction

**Amendment 6: Fallback MIME Types for .llm.md (P1)**

- **Changed:** Added flexible MIME type options for machine views
- **Location:** Section 4.2 (Normative Requirements)
- **Priority order:**
  1. `text/markdown; charset=utf-8` (RECOMMENDED)
  2. `text/x-llm+markdown; charset=utf-8` (ACCEPTABLE)
  3. `text/plain; charset=utf-8` (ACCEPTABLE)
- **Added:** Content negotiation based on Accept header
- **Reason:** Prevent compatibility issues with custom MIME types

**Amendment 7: Platform Compatibility Considerations (P1)**

- **Status:** Already present in specification
- **Added:** Comprehensive platform compatibility section
- **Location:** Section 3.8 (Platform Compatibility Considerations)
- **Includes:**
  - Hosting platform quirks (Vercel, Netlify, Cloudflare, GitHub Pages, Apache, Nginx)
  - AI agent tool limitations and compatibility matrix
  - CDN considerations (Cloudflare, AWS CloudFront, Fastly, Akamai)
  - Testing methodology and recommendations

**Version Numbering Clarification:**

- Enhanced Section 11 with detailed version lifecycle explanation
- Specification version: `v0.1-draft` (this document)
- Manifest version: `0.1` (use in implementations)
- Clear upgrade path to v1.0 when specification graduates from draft

**Backward Compatibility:**

- All changes are non-breaking or provide clear migration paths
- Existing `/llms.txt` implementations remain valid
- New MIME types and endpoints are additive enhancements
- Version 0.1 aligns with draft status (will become 1.0 at stable release)

**Implementation Impact:**

- **Critical (immediate):** Update MIME type from `application/yaml` to `text/plain`
- **Recommended:** Add `/llms.json` for maximum compatibility
- **Recommended:** Expose `/.well-known/arw-manifest.json` as primary machine endpoint
- **Optional:** Add robots.txt hints for enhanced discovery
- **Required:** Update version fields from 1.0 to 0.1 in all manifests

**Testing Checklist for v0.1-draft Compliance:**

- [ ] `/llms.txt` serves `text/plain; charset=utf-8` (not `application/yaml`)
- [ ] Version field is `0.1` (not `1.0`)
- [ ] `.llm.md` files use `text/markdown` or `text/plain` MIME types
- [ ] No binary data corruption when fetched by AI agents
- [ ] Both `/llms.txt` and `/llms.json` contain identical data (if both present)
- [ ] `/.well-known/arw-manifest.json` present (recommended for ARW-2+)
- [ ] Platform-specific MIME type configuration verified

---

**End of Document**
