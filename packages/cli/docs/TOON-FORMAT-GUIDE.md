# TOON Format Guide

**Status:** ⚡ Experimental Feature (v0.1.0)

TOON (Tree Object Notation) is an experimental machine-readable format for Agent-Ready Web content. It provides an alternative to Markdown with explicit structure and semantic annotations.

## What is TOON?

TOON is a structured format that represents web content as a tree of typed objects. Unlike Markdown (which is presentation-focused), TOON emphasizes semantic meaning and machine parseability.

**Key Benefits:**
- **Explicit structure** - No parsing ambiguity
- **Type safety** - Each element has a defined type
- **Semantic annotations** - Metadata embedded in structure
- **Efficient parsing** - Direct tree traversal vs regex parsing
- **Chunk addressability** - Navigate by semantic IDs

## When to Use TOON vs Markdown

### Use Markdown (.llm.md) When:
✅ Content is primarily text and headings\
✅ Human readability is important\
✅ Content authors edit manually\
✅ Simple blog posts or documentation\
✅ **RECOMMENDED for most ARW implementations**

### Use TOON (.llm.toon) When:
✅ Content has complex nested structure\
✅ Rich semantic metadata is needed\
✅ Generated programmatically from databases\
✅ Complex product catalogs or APIs\
✅ **EXPERIMENTAL - use for testing only**

## TOON Structure

### Basic Document

```toon
MachineView {
  version: "1.0"
  title: "Page Title"

  content: [
    Heading { level: 1, text: "Main Title" }
    Paragraph {
      content: [Text("Hello, world!")]
    }
  ]

  metadata: {
    source: "/page.html"
    generated_at: "2025-11-19T23:52:00Z"
    format: "arw-machine-view"
  }
}
```

### Chunk-Based Structure

```toon
MachineView {
  version: "1.0"
  title: "Product Page"

  chunks: [
    Chunk {
      id: "overview"
      title: "Product Overview"
      blocks: [
        Heading { level: 2, text: "Premium Keyboard" }
        Paragraph {
          content: [
            Text("Price: "),
            Strong { content: [Text("$129.99")] }
          ]
        }
      ]
    }

    Chunk {
      id: "specs"
      title: "Specifications"
      blocks: [
        Heading { level: 2, text: "Technical Specs" }
        Table {
          headers: ["Feature", "Value"]
          rows: [
            ["Battery Life", "40 hours"],
            ["Connectivity", "Bluetooth 5.0"]
          ]
        }
      ]
    }
  ]

  metadata: {
    source: "/products/keyboard.html"
    generated_at: "2025-11-19T23:52:00Z"
  }
}
```

## CLI Commands for TOON

### Generate TOON from HTML

```bash
# Generate TOON format (experimental)
arw generate index.html --format toon --output index.llm.toon

# Generate from directory
arw generate ./pages --recursive --format toon

# Force overwrite existing TOON files
arw generate ./pages -r --format toon -f
```

### Validate TOON Files

```bash
# Validate TOON structure
arw validate --path ./public --toon

# Strict validation
arw validate --strict --toon
```

### Convert Between Formats

```bash
# Convert TOON to Markdown (planned)
arw convert index.llm.toon --to markdown --output index.llm.md

# Convert Markdown to TOON (planned)
arw convert index.llm.md --to toon --output index.llm.toon
```

## TOON Element Types

### Content Elements

**Heading**
```toon
Heading { level: 1, text: "Title" }
Heading { level: 2, text: "Subtitle" }
```

**Paragraph**
```toon
Paragraph {
  content: [
    Text("Plain text "),
    Strong { content: [Text("bold text")] },
    Text(" and "),
    Em { content: [Text("italic text")] }
  ]
}
```

**Link**
```toon
Link {
  href: "/page"
  text: "Click here"
}

# Link in paragraph
Paragraph {
  content: [
    Text("Visit "),
    Link { href: "/about", text: "about page" }
  ]
}
```

**List**
```toon
List {
  ordered: false
  items: [
    "First item",
    "Second item",
    "Third item"
  ]
}

List {
  ordered: true
  items: [
    "Step 1",
    "Step 2",
    "Step 3"
  ]
}
```

**Table**
```toon
Table {
  headers: ["Name", "Price", "Stock"]
  rows: [
    ["Keyboard", "$129.99", "47"],
    ["Mouse", "$49.99", "156"]
  ]
}
```

**Code**
```toon
CodeBlock {
  language: "javascript"
  code: "console.log('Hello, world!');"
}

InlineCode { text: "npm install" }
```

**Image**
```toon
Image {
  src: "/images/product.jpg"
  alt: "Product photo"
  width: 800
  height: 600
}
```

### Structural Elements

**Chunk**
```toon
Chunk {
  id: "unique-chunk-id"
  title: "Chunk Title"
  description: "Optional description"
  blocks: [
    # Content elements go here
  ]
}
```

**Section**
```toon
Section {
  heading: "Section Title"
  content: [
    # Content elements go here
  ]
}
```

### Metadata

```toon
metadata: {
  source: "/path/to/source.html"
  generated_at: "2025-11-19T23:52:00Z"
  format: "arw-machine-view"
  version: "1.0"
  author: "Content Team"
  last_updated: "2025-11-19"
  tags: ["product", "electronics"]
}
```

## Comparison: Markdown vs TOON

### Example Content

**Markdown (.llm.md)**
```markdown
<!-- chunk: overview -->

# Premium Wireless Keyboard

**Price:** $129.99
**Stock:** In Stock

Premium wireless mechanical keyboard with hot-swappable switches.

[View specifications](#specs)

<!-- chunk: specs -->

## Technical Specifications

| Feature | Value |
|---------|-------|
| Battery Life | 40 hours |
| Connectivity | Bluetooth 5.0 |
```

**TOON (.llm.toon)**
```toon
MachineView {
  version: "1.0"
  title: "Premium Wireless Keyboard"

  chunks: [
    Chunk {
      id: "overview"
      title: "Product Overview"
      blocks: [
        Heading { level: 1, text: "Premium Wireless Keyboard" }

        Paragraph {
          content: [
            Strong { content: [Text("Price:")] },
            Text(" $129.99")
          ]
        }

        Paragraph {
          content: [
            Strong { content: [Text("Stock:")] },
            Text(" In Stock")
          ]
        }

        Paragraph {
          content: [
            Text("Premium wireless mechanical keyboard with hot-swappable switches.")
          ]
        }

        Link { href: "#specs", text: "View specifications" }
      ]
    }

    Chunk {
      id: "specs"
      title: "Technical Specifications"
      blocks: [
        Heading { level: 2, text: "Technical Specifications" }

        Table {
          headers: ["Feature", "Value"]
          rows: [
            ["Battery Life", "40 hours"],
            ["Connectivity", "Bluetooth 5.0"]
          ]
        }
      ]
    }
  ]

  metadata: {
    source: "/products/keyboard.html"
    generated_at: "2025-11-19T23:52:00Z"
  }
}
```

### Token Comparison

| Format | Size | Tokens (est.) | Savings |
|--------|------|---------------|---------|
| HTML | 55KB | 18,000 | - |
| Markdown | 8KB | 2,700 | 85% vs HTML |
| TOON | 12KB | 3,800 | 79% vs HTML |

**Analysis:**
- Markdown is most compact
- TOON is more structured but larger
- Both are vastly better than HTML

## Migration Guide

### Adding TOON to Existing ARW Site

**Step 1: Generate TOON files alongside Markdown**

```bash
# Generate both formats
arw generate ./pages --recursive --format markdown
arw generate ./pages --recursive --format toon
```

**Step 2: Update llms.txt manifest**

```yaml
# llms.txt
version: 0.1
profile: ARW-2

content:
  - url: /about
    machine_view: /about.llm.md  # Primary (Markdown)
    machine_view_toon: /about.llm.toon  # Alternative (TOON)
    purpose: about_page
```

**Step 3: Serve both formats with correct headers**

```javascript
// Markdown (primary)
app.get('/*.llm.md', (req, res) => {
  res.setHeader('Content-Type', 'text/markdown; charset=utf-8');
  res.setHeader('AI-Format', 'arw-markdown');
  // ... serve file
});

// TOON (alternative)
app.get('/*.llm.toon', (req, res) => {
  res.setHeader('Content-Type', 'text/plain; charset=utf-8');
  res.setHeader('AI-Format', 'arw-toon');
  // ... serve file
});
```

**Step 4: Test with ARW validator**

```bash
arw validate --strict --toon
```

### Content Negotiation (Advanced)

Allow agents to request preferred format:

```javascript
app.get('/about.llm', (req, res) => {
  const accept = req.headers['accept'];

  if (accept?.includes('text/x-toon')) {
    res.redirect(302, '/about.llm.toon');
  } else {
    res.redirect(302, '/about.llm.md');
  }
});
```

## Best Practices

### When to Use TOON

✅ **DO use TOON when:**
- Generating content programmatically from databases
- Working with complex nested structures
- Need precise semantic annotations
- Building product catalogs with rich metadata

❌ **DON'T use TOON when:**
- Content is primarily text (use Markdown)
- Manual editing by content teams (use Markdown)
- Simple blog posts or documentation (use Markdown)
- Just getting started with ARW (use Markdown)

### TOON File Naming

- Use `.llm.toon` extension (matches `.llm.md` pattern)
- Place alongside corresponding `.llm.md` files
- Same base name: `about.llm.md` + `about.llm.toon`

### TOON in llms.txt

```yaml
content:
  - url: /products/keyboard
    machine_view: /products/keyboard.llm.md  # Primary
    formats:
      - type: markdown
        url: /products/keyboard.llm.md
      - type: toon
        url: /products/keyboard.llm.toon  # Alternative
```

## Experimental Status

⚠️ **IMPORTANT:** TOON is an experimental feature.

**What this means:**
- Format may change in future versions
- Not all ARW tools support TOON yet
- Limited agent support (most agents expect Markdown)
- Syntax may evolve based on feedback

**Current limitations:**
- No official TOON parser library yet
- Limited tooling support
- Not in ARW specification v0.1-draft
- May be refined or removed in future

**Use TOON for:**
- Testing and experimentation
- Providing feedback on the format
- Building proof-of-concept implementations
- Research into structured content formats

**Stick with Markdown for:**
- Production ARW sites
- Maximum compatibility
- Proven agent support
- Stable, well-documented format

## Examples & Resources

See the `/examples/toon-format/` directory for:
- Complete TOON examples
- Side-by-side Markdown/TOON comparisons
- Integration examples
- Testing templates

## Feedback & Discussion

TOON is an experimental format. We welcome feedback:

- GitHub Issues: Report bugs or suggest improvements
- GitHub Discussions: Share use cases and experiences
- Example PRs: Contribute TOON examples

**Questions to explore:**
- Is TOON more useful than Markdown for certain content types?
- What syntax improvements would help?
- Should TOON support additional semantic elements?
- How can we reduce TOON file size while maintaining structure?

## Future Directions

**Potential enhancements:**
- Schema validation for TOON structure
- TOON parser libraries (JavaScript, Python, Rust)
- TOON-to-Markdown bidirectional conversion
- TOON editor with live preview
- Integration with CMS platforms
- Formal TOON specification

---

**Remember:** Markdown is the recommended format for most ARW implementations. Use TOON only for experimentation and feedback.

For questions or feedback, see: [GitHub Discussions](https://github.com/agent-ready-web/agent-ready-web/discussions)
