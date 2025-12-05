# @agent-ready-web/nextjs-plugin

Next.js plugin for seamless [Agent-Ready Web (ARW)](https://arw.dev) integration. Automatically generate machine-readable content alongside your React components.

[![npm version](https://img.shields.io/npm/v/@agent-ready-web/nextjs-plugin.svg)](https://www.npmjs.com/package/@agent-ready-web/nextjs-plugin)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- ✅ **Zero-config setup** - Add ARW in minutes
- ✅ **App Router support** - First-class Next.js 13+ support
- ✅ **Pages Router compatible** - Works with legacy routing
- ✅ **Type-safe** - Full TypeScript support
- ✅ **Auto-generation** - Optional build-time machine view generation
- ✅ **React hooks** - `useARW()`, `useARWEnabled()`, `useMachineViewUrl()`
- ✅ **Components** - `<ARWHead />`, `<ARWProvider />`
- ✅ **Flexible** - Multiple integration patterns

## Installation

```bash
npm install @agent-ready-web/nextjs-plugin
# or
yarn add @agent-ready-web/nextjs-plugin
# or
pnpm add @agent-ready-web/nextjs-plugin
```

## Quick Start

### 1. Update `next.config.js`

```javascript
const { withARW } = require('@agent-ready-web/nextjs-plugin');

module.exports = withARW({
  arw: {
    manifest: {
      siteName: 'My Website',
      homepage: 'https://example.com',
      contact: 'ai@example.com'
    }
  }
});
```

### 2. Add ARW Provider (App Router)

```tsx
// app/layout.tsx
import { ARWProvider } from '@agent-ready-web/nextjs-plugin/components';

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>
        <ARWProvider>
          {children}
        </ARWProvider>
      </body>
    </html>
  );
}
```

### 3. Add ARW Metadata to Pages

```tsx
// app/page.tsx
import { generateARWMetadata } from '@agent-ready-web/nextjs-plugin/components';

export const metadata = {
  title: 'Home',
  ...generateARWMetadata({
    priority: 'high',
    machineViewPath: '/index.llm.md'
  })
};

export default function HomePage() {
  return (
    <main>
      <section data-chunk-id="hero">
        <h1>Welcome</h1>
      </section>
    </main>
  );
}
```

That's it! Your Next.js app is now ARW-enabled.

## Integration Options

This plugin supports **5 integration patterns** with different trade-offs:

### Option 1: CLI Build Time (Minimal)

Add ARW generation as a build step:

```json
{
  "scripts": {
    "build": "next build && arw generate ./out --recursive"
  }
}
```

**Best for:** Existing apps wanting minimal changes

### Option 2: Next.js Plugin (Recommended)

Use the plugin for automatic integration:

```javascript
// next.config.js
const { withARW } = require('@agent-ready-web/nextjs-plugin');

module.exports = withARW({
  arw: {
    autoGenerate: true, // Enable auto-generation
    manifest: {
      siteName: 'My Site',
      homepage: 'https://example.com',
      contact: 'ai@example.com'
    }
  }
});
```

**Best for:** New projects with full ARW adoption

### Option 3: Runtime Integration

Use React components and hooks for dynamic content:

```tsx
import { useARW } from '@agent-ready-web/nextjs-plugin/runtime';

function MyComponent() {
  const { manifest, loading } = useARW();

  if (loading) return <div>Loading...</div>;

  return <div>{manifest?.site.name}</div>;
}
```

**Best for:** Dynamic content sites

## Configuration

### Full Configuration Example

```javascript
// next.config.js
const { withARW } = require('@agent-ready-web/nextjs-plugin');

module.exports = withARW({
  arw: {
    // Auto-generate machine views during build
    autoGenerate: true,

    // Output directory
    outputDir: 'public/.well-known',

    // Watch mode (development only)
    watch: process.env.NODE_ENV === 'development',

    // Manifest configuration (REQUIRED)
    manifest: {
      siteName: 'My Website',
      description: 'Website description',
      homepage: 'https://example.com',
      contact: 'ai@example.com',

      // Usage policies
      policies: {
        training: {
          allowed: false,
          note: 'Content not licensed for model training'
        },
        inference: {
          allowed: true,
          restrictions: ['attribution_required']
        },
        attribution: {
          required: true,
          format: 'link'
        }
      }
    },

    // Generation options
    generation: {
      format: 'markdown', // or 'toon'
      chunkStrategy: 'semantic',
      includePatterns: ['app/**/*.tsx'],
      excludePatterns: ['app/admin/**']
    },

    // Runtime options
    runtime: {
      enableClientGeneration: false,
      cacheStrategy: 'memory',
      ttl: 3600000 // 1 hour
    }
  }
});
```

## Components

### ARWHead

Add ARW discovery metadata to your page:

```tsx
import { ARWHead } from '@agent-ready-web/nextjs-plugin/components';

export default function Page() {
  return (
    <>
      <ARWHead priority="high" />
      <main>Content</main>
    </>
  );
}
```

### ARWProvider

Provide ARW context to your app:

```tsx
import { ARWProvider } from '@agent-ready-web/nextjs-plugin/components';

export default function RootLayout({ children }) {
  return (
    <ARWProvider config={{ priority: 'high' }}>
      {children}
    </ARWProvider>
  );
}
```

### generateARWMetadata

Generate metadata for App Router:

```tsx
import { generateARWMetadata } from '@agent-ready-web/nextjs-plugin/components';

export const metadata = {
  title: 'My Page',
  ...generateARWMetadata({
    priority: 'high',
    machineViewPath: '/page.llm.md'
  })
};
```

## Hooks

### useARW

Access ARW manifest data:

```tsx
import { useARW } from '@agent-ready-web/nextjs-plugin/runtime';

function MyComponent() {
  const { manifest, loading, error } = useARW();

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return <div>{manifest?.site.name}</div>;
}
```

### useARWEnabled

Check if ARW is enabled:

```tsx
import { useARWEnabled } from '@agent-ready-web/nextjs-plugin/runtime';

function MyComponent() {
  const isEnabled = useARWEnabled();

  return <div>ARW: {isEnabled ? 'Enabled' : 'Disabled'}</div>;
}
```

### useMachineViewUrl

Get machine view URL for current page:

```tsx
import { useMachineViewUrl } from '@agent-ready-web/nextjs-plugin/runtime';

function MyComponent() {
  const machineViewUrl = useMachineViewUrl();

  return <a href={machineViewUrl}>Machine View</a>;
}
```

## Examples

### App Router (Basic)

See [examples/basic](./examples/basic) for a minimal App Router integration.

### App Router (Advanced)

See [examples/advanced](./examples/advanced) for advanced features with auto-generation.

### Pages Router

See [examples/pages-router](./examples/pages-router) for Pages Router integration.

## Migration Guide

### From llms.txt

If you're using the standalone `llms.txt` format:

1. Install the plugin
2. Update `next.config.js`
3. Add manifest configuration
4. (Optional) Enable auto-generation

### From Manual Implementation

If you manually implemented ARW:

1. Install the plugin
2. Replace manual headers with `<ARWHead />`
3. Move manifest to `next.config.js`
4. (Optional) Use hooks and components

## Trade-offs

| Approach | Performance | DX | Flexibility | Bundle Size | Complexity |
|----------|------------|-----|------------|-------------|-----------|
| CLI Build | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| Plugin | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Runtime | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ |

## Performance

- **Build time impact**: < 30%
- **Runtime overhead**: Minimal (metadata only)
- **Bundle size**: ~50KB (gzipped)

## TypeScript

Full TypeScript support included:

```typescript
import type {
  ARWNextConfig,
  ARWManifest,
  ARWPolicies,
  PageMetadata
} from '@agent-ready-web/nextjs-plugin';
```

## Requirements

- Next.js >= 13.0.0
- React >= 18.0.0
- Node.js >= 16.0.0

## Contributing

Contributions welcome! Please see the [main repository](https://github.com/nolandubeau/agent-ready-web) for guidelines.

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Links

- [ARW Specification](https://github.com/nolandubeau/agent-ready-web/blob/main/spec/ARW-0.1-draft.md)
- [ARW CLI](https://github.com/nolandubeau/agent-ready-web/tree/main/packages/cli)
- [ARW Website](https://arw.dev)
- [GitHub](https://github.com/nolandubeau/agent-ready-web)

## Support

- [GitHub Issues](https://github.com/nolandubeau/agent-ready-web/issues)
- [GitHub Discussions](https://github.com/nolandubeau/agent-ready-web/discussions)

---

Made with ❤️ for the Agent-Ready Web | [arw.dev](https://arw.dev)
