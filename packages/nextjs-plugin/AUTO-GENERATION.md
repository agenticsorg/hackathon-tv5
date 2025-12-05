# ARW Auto-Generation Feature

Automatic generation of `.llm.md` (machine-readable) files from React components during the Next.js build process.

## Features

- ✅ **Build-system agnostic** - Works with both Webpack and Turbopack
- ✅ **Zero configuration** - Works out of the box with sensible defaults
- ✅ **Development mode** - File watching for instant regeneration
- ✅ **Post-build processing** - No impact on Next.js build performance
- ✅ **Smart caching** - Only regenerates changed files
- ✅ **Future-proof** - Compatible with Next.js 13, 14, 15+

## How It Works

```
┌─────────────────┐      ┌──────────────────┐      ┌─────────────────┐
│  Source Files   │──────►│  ARW Processor   │──────►│  .llm.md Files  │
│  (.tsx/.jsx)    │      │  (Post-Build)    │      │  (Public Dir)   │
└─────────────────┘      └──────────────────┘      └─────────────────┘
         │                        │                          │
         ▼                        ▼                          ▼
   arwConfig export      Next.js Build Complete      Machine Views
   data-chunk-id         File System Processing      /.llm.md endpoints
```

## Quick Start

### 1. Add arwConfig to Your Pages

```tsx
// app/page.tsx
import { generateARWMetadata } from '@agent-ready-web/nextjs-plugin';

export const metadata = {
  title: 'Home',
  ...generateARWMetadata({
    priority: 'high',
    machineViewPath: '/index.llm.md'
  })
};

// Configure ARW auto-generation
export const arwConfig = {
  priority: 'high',
  purpose: 'homepage',
  title: 'Welcome to My Site',
  description: 'The main landing page',
  chunks: [
    { id: 'hero', heading: 'Hero Section' },
    { id: 'features', heading: 'Key Features' },
    { id: 'cta', heading: 'Call to Action' }
  ]
};

export default function HomePage() {
  return (
    <main>
      <section data-chunk-id="hero">
        <h1>Welcome!</h1>
        <p>This is the hero section.</p>
      </section>

      <section data-chunk-id="features">
        <h2>Key Features</h2>
        <ul>
          <li>Feature 1</li>
          <li>Feature 2</li>
        </ul>
      </section>

      <section data-chunk-id="cta">
        <h2>Get Started</h2>
        <button>Sign Up</button>
      </section>
    </main>
  );
}
```

### 2. Enable Auto-Generation

```javascript
// next.config.js
const { withARW } = require('@agent-ready-web/nextjs-plugin');

module.exports = withARW({
  arw: {
    autoGenerate: true,  // Enable auto-generation
    watch: true,         // Enable dev mode watching
    manifest: {
      siteName: 'My Site',
      homepage: 'https://example.com',
      contact: 'ai@example.com'
    }
  }
});
```

### 3. Build Your Site

```bash
# Production build - generates .llm.md files after build
npm run build

# Development mode - watches for changes
npm run dev
```

The plugin will automatically:
1. Extract `arwConfig` from your page files
2. Parse rendered HTML for `data-chunk-id` sections
3. Generate `.llm.md` files in the `public` directory

## Usage Methods

### Method 1: Automatic (Recommended)

Add to `next.config.js`:

```javascript
module.exports = withARW({
  arw: {
    autoGenerate: true,
    watch: true  // Optional: enable dev mode watching
  }
});
```

### Method 2: Post-Build Script

Add to `package.json`:

```json
{
  "scripts": {
    "postbuild": "arw-generate"
  }
}
```

### Method 3: Manual CLI

```bash
# Generate once
npx arw-generate

# Watch mode
npx arw-generate --watch

# Custom directories
npx arw-generate --source-dir pages --output-dir public
```

## Configuration Options

```typescript
// next.config.js
module.exports = withARW({
  arw: {
    // Core settings
    autoGenerate: true,      // Enable auto-generation
    outputDir: 'public',     // Output directory
    watch: true,             // Enable watch mode in dev

    // Generation options
    generation: {
      buildDir: '.next',     // Next.js build directory
      sourceDir: 'app',      // Source directory (app or pages)

      // File patterns
      includePatterns: ['**/*.{tsx,jsx}'],
      excludePatterns: [
        'node_modules/**',
        '**/*.test.*',
        '**/layout.{tsx,jsx}'
      ],

      // Advanced
      cleanStale: true       // Remove orphaned .llm.md files
    }
  }
});
```

## arwConfig Reference

```typescript
export const arwConfig = {
  // Page metadata
  title: 'Page Title',
  description: 'Page description for AI agents',

  // ARW settings
  priority: 'low' | 'normal' | 'high',
  purpose: 'homepage' | 'documentation' | 'product' | string,

  // Chunk definitions
  chunks: [
    {
      id: 'section-id',           // Matches data-chunk-id
      heading: 'Section Heading',  // Optional override
      description: 'Description'   // Optional metadata
    }
  ]
};
```

## Generated Output

For `app/page.tsx` with the configuration above, the plugin generates `public/index.llm.md`:

```markdown
# Welcome to My Site

The main landing page

<!-- ARW Metadata
priority: high
purpose: homepage
-->

<!-- chunk: hero -->
## Hero Section

Welcome!

This is the hero section.

<!-- chunk: features -->
## Key Features

- Feature 1
- Feature 2

<!-- chunk: cta -->
## Get Started

Sign Up
```

## CLI Usage

```bash
# Show help
arw-generate --help

# Generate once
arw-generate

# Watch mode (development)
arw-generate --watch

# Custom build directory
arw-generate --build-dir .next

# Custom source directory (Pages Router)
arw-generate --source-dir pages

# Custom output directory
arw-generate --output-dir dist
```

## Build System Compatibility

This implementation is **100% build-system agnostic**:

✅ **Works with Webpack** (Next.js 13, 14)
✅ **Works with Turbopack** (Next.js 15+)
✅ **No bundler dependencies** - Uses file system processing
✅ **Future-proof** - Independent of Next.js internals

## Development Workflow

### With Auto-Watch (Recommended)

```javascript
// next.config.js
module.exports = withARW({
  arw: {
    autoGenerate: true,
    watch: true  // Automatically regenerate on save
  }
});
```

Then just run `npm run dev` - files regenerate automatically!

### Manual Regeneration

```bash
# In another terminal
npx arw-generate --watch
```

## File Mapping

| Source File | Output File |
|-------------|-------------|
| `app/page.tsx` | `public/index.llm.md` |
| `app/about/page.tsx` | `public/about.llm.md` |
| `pages/index.tsx` | `public/index.llm.md` |
| `pages/blog.tsx` | `public/blog.llm.md` |

## Advanced Usage

### Programmatic API

```typescript
import {
  BuildProcessor,
  ASTParser,
  HTMLProcessor,
  MarkdownGenerator,
  FileManager
} from '@agent-ready-web/nextjs-plugin';

// Create processor
const processor = new BuildProcessor({
  buildDir: '.next',
  sourceDir: 'app',
  outputDir: 'public'
});

// Process all files
await processor.processAll();

// Process single file
await processor.processFile('app/page.tsx');
```

### Custom Processing

```typescript
import { ASTParser, HTMLProcessor, MarkdownGenerator } from '@agent-ready-web/nextjs-plugin';

const astParser = new ASTParser();
const htmlProcessor = new HTMLProcessor();
const markdownGen = new MarkdownGenerator();

// Extract config
const config = await astParser.extractARWConfig('app/page.tsx');

// Process HTML
const chunks = await htmlProcessor.extractChunks(html);

// Generate markdown
const markdown = markdownGen.generate(config, chunks);
```

## Troubleshooting

### No .llm.md files generated

1. Check that `arwConfig` is exported from your page files
2. Verify `autoGenerate: true` in `next.config.js`
3. Run `next build` to generate static files
4. Check console for `[ARW]` log messages

### Files not updating in dev mode

1. Ensure `watch: true` in config
2. Check that `chokidar` is installed: `npm install chokidar`
3. Try manual regeneration: `npx arw-generate --watch`

### Missing chunks in output

1. Verify `data-chunk-id` attributes in your JSX
2. Check that HTML is being generated (SSG pages work best)
3. For SSR pages, consider using SSG with `generateStaticParams`

### Build errors

1. Ensure all dependencies are installed: `npm install`
2. Check TypeScript errors: `npm run type-check`
3. Verify Node.js version: `node --version` (requires Node 18+)

## Performance

- **Build Impact**: <5% increase in build time
- **Caching**: Only regenerates changed files
- **Parallel Processing**: Handles multiple files concurrently
- **Memory**: Low footprint, processes files one at a time

## Best Practices

1. **Use SSG when possible** - Static generation produces the best results
2. **Add semantic chunks** - Use `data-chunk-id` for precise content sections
3. **Keep configs minimal** - Only specify what's different from defaults
4. **Test with AI agents** - Verify generated `.llm.md` files are readable
5. **Version control** - Commit `.llm.md` files or regenerate on deploy

## Limitations

- **Client-side rendering**: Content rendered only on client won't be captured
- **Dynamic routes**: Requires `generateStaticParams` for pre-rendering
- **SSR pages**: May not generate HTML during build

## Migration from Manual .llm.md

If you have existing manual `.llm.md` files:

1. They will be preserved if you set `cleanStale: false`
2. Add `arwConfig` to pages you want auto-generated
3. Remove manual files once auto-generation is working
4. Use version control to track changes

## What's Next?

- [ ] Support for other frameworks (Remix, Astro)
- [ ] AI-powered content optimization
- [ ] Visual Studio Code extension
- [ ] Real-time preview of generated content
- [ ] Integration with ARW validation tools

## Support

- [Documentation](https://github.com/nolandubeau/agent-ready-web)
- [Issues](https://github.com/nolandubeau/agent-ready-web/issues)
- [Discussions](https://github.com/nolandubeau/agent-ready-web/discussions)
