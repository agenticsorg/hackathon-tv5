# ARW Inspector Guide

**Visual explorer for Agent-Ready Web capabilities**

The ARW Inspector is a browser-based tool for visually exploring, validating, and understanding ARW implementations on any website.

## Overview

The inspector provides a comprehensive view of how AI agents discover and interact with ARW-enabled websites. It's designed as a companion tool to `arw serve` and helps developers, publishers, and AI agent creators understand ARW implementations.

## Key Features

### 🔍 Discovery & Parsing

- Automatically fetches and parses `llms.txt` from any URL
- Validates YAML syntax and ARW schema compliance
- Reports errors and warnings with clear descriptions
- Handles CORS issues gracefully

### 📄 Content Inspection

- Lists all content entries from the discovery file
- Shows priorities, purposes, and metadata
- Links to machine views
- Displays content descriptions and context

### ⚡ Actions Analysis

- Displays OAuth-protected actions
- Shows HTTP methods and endpoints
- Lists authentication requirements and scopes
- Renders JSON schemas for request bodies

### 📋 Policy Visualization

- Clear display of training and inference policies
- Attribution requirements and templates
- Rate limits (authenticated vs unauthenticated)
- Policy explanations and context

### 📝 Machine View Comparison

- Side-by-side raw Markdown and rendered preview
- Syntax highlighting for Markdown source
- Content chunk identification
- Size and token statistics

### 🔌 Protocol Support

- Lists all protocol endpoints (MCP, ACP, A2A)
- Shows protocol types and descriptions
- Schema file references

## Getting Started

### Installation

```bash
cd examples/arw-inspector
npm install
```

### Running the Inspector

```bash
npm run dev
```

Visit http://localhost:5174 to use the inspector.

### Basic Usage

1. **Enter URL**: Type or paste the URL of an ARW-enabled website
2. **Click Inspect**: The inspector fetches and parses the ARW capabilities
3. **Browse Tabs**: Navigate through Overview, Content, Actions, Policies, and Machine Views
4. **Explore Details**: Click on items to see full details, schemas, and metadata

## User Interface

### Tabs

#### 1. Overview

The Overview tab provides a high-level summary:

- **Site Information**: Name, homepage, contact, ARW version
- **Capability Summary**: Content count, action count, protocol count, machine views count
- **Policy Summary**: Quick view of training/inference/attribution status
- **Protocol List**: All supported protocols with endpoints

**Use this tab to:**

- Get a quick sense of what the site offers
- Verify basic site information
- Check protocol support at a glance

#### 2. Content

The Content tab lists all content entries:

- **Stats**: Total entries, entries with machine views, high priority entries
- **Entry List**: Each entry shows:
  - URL and priority badge
  - Description
  - Purpose and machine view link
  - Metadata (expandable)

**Use this tab to:**

- Understand the site's content structure
- Find specific pages or resources
- Verify machine views are declared
- Check metadata completeness

#### 3. Actions

The Actions tab displays OAuth-protected operations:

- **Action Cards** showing:
  - Name and HTTP method badge
  - Endpoint URL
  - Description
  - Authentication type
  - Required scopes
  - Request schema (expandable)

**Use this tab to:**

- Discover what operations agents can perform
- Understand authentication requirements
- Review request schemas for implementation
- Validate action declarations

#### 4. Policies

The Policies tab shows usage policies:

- **Training Policy**: Allowed or restricted
- **Inference Policy**: Allowed or restricted, with restrictions list
- **Attribution Policy**: Required or optional, with format and template
- **Rate Limits**: Per-minute limits, authenticated vs unauthenticated

**Use this tab to:**

- Understand publisher terms
- Respect content usage guidelines
- Implement proper attribution
- Stay within rate limits

#### 5. Machine Views

The Machine Views tab provides detailed view inspection:

- **Selector**: List of all available machine views
- **Side-by-Side Display**:
  - Raw Markdown source (with syntax highlighting)
  - Rendered preview (as agents would see it)
- **Chunks**: List of identified content chunks
- **Statistics**: File size, line count, chunk count

**Use this tab to:**

- Compare HTML vs machine view content
- Verify semantic structure
- Check chunk declarations
- Validate Markdown formatting

## Workflows

### For Publishers

**Validating Your Implementation:**

1. Run your site locally: `arw serve --port 3000`
2. Open inspector: http://localhost:5174
3. Inspect: `http://localhost:3000`
4. Check for errors/warnings in each tab
5. Verify machine views render correctly
6. Ensure policies match your intent

**Iterating on Machine Views:**

1. Edit your `.llm.md` files
2. Refresh the inspector (re-inspect the URL)
3. Check the rendered preview
4. Verify chunk IDs are correct
5. Review size/token statistics

### For Developers

**Learning ARW by Example:**

1. Inspect well-known ARW sites:
   - `https://arw.dev`
   - Your own examples
2. Study their content structure in Content tab
3. Examine machine views in Machine Views tab
4. Note patterns in actions and policies
5. Use as templates for your implementation

**Debugging Issues:**

1. Inspect your site
2. Check Overview for immediate errors
3. Review warnings in each tab
4. Verify llms.txt syntax
5. Test machine view accessibility
6. Validate schema compliance

### For AI Agent Developers

**Understanding Discovery:**

1. Inspect target site
2. Study content catalog structure
3. Identify available actions
4. Review authentication requirements
5. Note policy restrictions
6. Plan agent behavior accordingly

**Testing Discovery Logic:**

1. Use inspector as reference implementation
2. Compare your agent's parsing with inspector
3. Verify you handle all content types
4. Test action discovery
5. Respect declared policies

## Advanced Features

### CORS Handling

The inspector handles CORS issues in several ways:

1. **Direct Fetch**: Tries to fetch directly (works if CORS enabled)
2. **Proxy Mode** (planned): Route through backend proxy
3. **Error Messages**: Clear explanation when CORS blocks request

For local development, ensure your server enables CORS:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, OPTIONS
```

The `arw serve` command enables CORS automatically.

### Validation

The inspector validates:

- ✅ YAML syntax in llms.txt
- ✅ Required fields (version, site)
- ✅ Content entry structure
- ✅ Action schema validity
- ✅ Machine view accessibility
- ⚠️ Missing optional fields (warnings, not errors)

### Performance

- **Parallel Fetching**: Machine views fetched concurrently
- **Caching**: Results cached until re-inspection
- **Lazy Loading**: Tabs load content only when viewed
- **Size Limits**: Handles large machine views efficiently

## Integration with ARW CLI

The inspector complements the ARW CLI tool:

### Current Integration

```bash
# Serve site with CORS (works with inspector)
arw serve --port 3000

# In another terminal, run inspector
cd examples/arw-inspector && npm run dev

# Inspect via browser at http://localhost:5174
```

### Planned Integration

Future releases will provide tighter integration:

```bash
# Launch server + inspector in one command
arw serve --inspect

# Auto-open inspector at http://localhost:8080/inspector
# Auto-refresh on file changes
# Embedded validation
```

See [CLI Roadmap](../cli/specs/CLI-EXPANSION-PLAN.md) for details.

## Troubleshooting

### "No ARW Discovery File Found"

**Cause**: Inspector couldn't fetch `llms.txt`

**Solutions:**

- Verify the URL is correct
- Check that `/llms.txt` exists at the root
- Ensure CORS is enabled on the server
- Try with `http://` instead of `https://` for local development

### "Failed to parse llms.txt as YAML"

**Cause**: Syntax error in YAML file

**Solutions:**

- Check YAML syntax (indentation, colons, quotes)
- Validate with an online YAML validator
- Review examples in `/examples/`
- Use `arw validate` command

### "Failed to fetch machine view"

**Cause**: Machine view URL is incorrect or inaccessible

**Solutions:**

- Verify machine view path in llms.txt
- Check that file exists at declared path
- Ensure path starts with `/` (absolute) or matches base URL
- Verify CORS headers on machine view responses

### Inspector not loading / blank screen

**Cause**: Build or runtime error

**Solutions:**

- Check browser console for errors
- Try clearing browser cache
- Rebuild: `npm run build && npm run dev`
- Check for JavaScript errors in console

## Development

### Building from Source

```bash
cd examples/arw-inspector
npm install
npm run typecheck  # Verify TypeScript
npm run build      # Production build
npm run preview    # Preview production build
```

### Modifying the Inspector

The inspector is built with React and TypeScript:

- **Components**: `src/components/` - React UI components
- **Utils**: `src/utils/` - Inspection logic
- **Types**: `src/types.ts` - TypeScript type definitions
- **Styles**: Component-scoped CSS files

To add a new panel:

1. Create component in `src/components/YourPanel.tsx`
2. Add styles in `src/components/YourPanel.css`
3. Import and add tab in `src/components/Inspector.tsx`

### Contributing

See [Contributing Guidelines](../README.md#contributing) for how to submit improvements.

## Future Roadmap

Planned features:

- **Agent Simulation**: Simulate how different agents discover/use the site
- **Validation Reports**: Comprehensive compliance checking with export
- **Comparison Mode**: Side-by-side comparison of multiple sites
- **Browser Extension**: Inspect any page directly from browser toolbar
- **CLI Mode**: Headless inspection for CI/CD pipelines
- **Performance Analysis**: Token cost estimation and optimization suggestions
- **Real-time Updates**: Live reload when ARW files change
- **Export**: Generate documentation from inspection results

## Resources

- **Inspector README**: [examples/arw-inspector/README.md](../examples/arw-inspector/README.md)
- **ARW Specification**: [spec/ARW-v0.1-DRAFT.md](../spec/ARW-v0.1-DRAFT.md)
- **CLI Documentation**: [cli/README.md](../cli/README.md)
- **Example Sites**: [examples/README.md](../examples/README.md)

---

**Part of the [Agent-Ready Web](https://arw.dev) specification**
