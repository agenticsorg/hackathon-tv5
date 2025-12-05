# ARW Next.js Integration Guide

Complete guide for integrating Agent-Ready Web into Next.js applications with all available options and recommendations.

## Table of Contents

- [Integration Options](#integration-options)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Usage Patterns](#usage-patterns)
- [Best Practices](#best-practices)
- [Migration](#migration)
- [Troubleshooting](#troubleshooting)

## Integration Options

### 1. CLI-Based Build Time Integration

**When to use:** Existing Next.js apps, minimal changes required

**Setup:**
```bash
npm install @agent-ready-web/cli
```

**Configuration:**
```json
{
  "scripts": {
    "build": "next build && arw generate ./out --recursive",
    "postbuild": "arw validate --strict"
  }
}
```

**Pros:**
- ✅ Zero runtime overhead
- ✅ Works with existing setup
- ✅ No code changes

**Cons:**
- ❌ Separate build step
- ❌ No component-level integration
- ❌ Manual synchronization

---

### 2. Next.js Plugin (Recommended)

**When to use:** New projects, full ARW adoption

**Setup:**
```bash
npm install @agent-ready-web/nextjs-plugin
```

**Configuration:**
```javascript
// next.config.js
const { withARW } = require('@agent-ready-web/nextjs-plugin');

module.exports = withARW({
  arw: {
    autoGenerate: true,
    manifest: {
      siteName: 'My Site',
      homepage: 'https://example.com',
      contact: 'ai@example.com'
    }
  }
});
```

**Pros:**
- ✅ Best developer experience
- ✅ Automatic synchronization
- ✅ Type-safe integration
- ✅ Hot reload in development

**Cons:**
- ❌ Build complexity
- ❌ Next.js version dependency

---

### 3. NAPI-RS Native Bindings

**When to use:** Performance-critical server-side generation

**Setup:**
```bash
npm install @agent-ready-web/cli
```

**Usage:**
```typescript
import { generateMachineView } from '@agent-ready-web/cli';

export async function generateMetadata({ params }) {
  const html = await renderToString(<MyComponent />);
  const machineView = await generateMachineView(html, {
    format: 'markdown',
    priority: 'high'
  });

  return {
    alternates: {
      types: {
        'text/x-llm+markdown': `/page.llm.md`
      }
    }
  };
}
```

**Pros:**
- ✅ Native Rust performance
- ✅ Direct API access
- ✅ Server-side rendering compatible

**Cons:**
- ❌ Platform-specific binaries
- ❌ Larger package size
- ❌ Complex deployment

---

### 4. WASM Universal Integration

**When to use:** Edge runtime, client-side generation

**Setup:**
```bash
npm install @agent-ready-web/cli-wasm
```

**Usage:**
```typescript
import init, { generateMachineView } from '@agent-ready-web/cli-wasm';

export async function MyComponent() {
  await init();
  const machineView = await generateMachineView(content);
  // ...
}
```

**Pros:**
- ✅ Universal (browser + server)
- ✅ Edge runtime compatible
- ✅ No native dependencies

**Cons:**
- ❌ Initialization overhead
- ❌ Slower than native
- ❌ Bundle size

---

### 5. Runtime React Integration

**When to use:** Dynamic content, CMS integrations

**Setup:**
```bash
npm install @agent-ready-web/nextjs-plugin
```

**Usage:**
```tsx
import { ARWProvider, useARW } from '@agent-ready-web/nextjs-plugin/runtime';

function MyApp({ children }) {
  return (
    <ARWProvider config={{ priority: 'high' }}>
      {children}
    </ARWProvider>
  );
}

function MyComponent() {
  const { manifest } = useARW();
  return <div>{manifest?.site.name}</div>;
}
```

**Pros:**
- ✅ Dynamic content support
- ✅ Component-level control
- ✅ React ecosystem

**Cons:**
- ❌ Runtime overhead
- ❌ SEO considerations
- ❌ Client-side computation

---

## Quick Start

### App Router (Next.js 13+)

**1. Install:**
```bash
npm install @agent-ready-web/nextjs-plugin
```

**2. Configure:**
```javascript
// next.config.js
const { withARW } = require('@agent-ready-web/nextjs-plugin');

module.exports = withARW({
  arw: {
    manifest: {
      siteName: 'My Site',
      homepage: 'https://example.com',
      contact: 'ai@example.com'
    }
  }
});
```

**3. Add Provider:**
```tsx
// app/layout.tsx
import { ARWProvider } from '@agent-ready-web/nextjs-plugin/components';

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        <ARWProvider>{children}</ARWProvider>
      </body>
    </html>
  );
}
```

**4. Add Metadata:**
```tsx
// app/page.tsx
import { generateARWMetadata } from '@agent-ready-web/nextjs-plugin/components';

export const metadata = {
  title: 'Home',
  ...generateARWMetadata({ priority: 'high' })
};
```

### Pages Router (Next.js 12 and below)

**1. Configure:**
```javascript
// next.config.js
const { withARW } = require('@agent-ready-web/nextjs-plugin');

module.exports = withARW({
  arw: {
    manifest: {
      siteName: 'My Site',
      homepage: 'https://example.com',
      contact: 'ai@example.com'
    }
  }
});
```

**2. Add Provider:**
```tsx
// pages/_app.tsx
import { ARWProvider } from '@agent-ready-web/nextjs-plugin/components';

export default function App({ Component, pageProps }) {
  return (
    <ARWProvider>
      <Component {...pageProps} />
    </ARWProvider>
  );
}
```

**3. Add Head:**
```tsx
// pages/index.tsx
import Head from 'next/head';
import { ARWHead } from '@agent-ready-web/nextjs-plugin/components';

export default function Home() {
  return (
    <>
      <Head>
        <ARWHead priority="high" />
      </Head>
      <main>Content</main>
    </>
  );
}
```

---

## Configuration

### Minimal Configuration

```javascript
module.exports = withARW({
  arw: {
    manifest: {
      siteName: 'My Site',
      homepage: 'https://example.com',
      contact: 'ai@example.com'
    }
  }
});
```

### Full Configuration

```javascript
module.exports = withARW({
  arw: {
    // Auto-generate machine views
    autoGenerate: true,

    // Output directory
    outputDir: 'public/.well-known',

    // Watch mode (dev only)
    watch: process.env.NODE_ENV === 'development',

    // Manifest (REQUIRED)
    manifest: {
      siteName: 'My Website',
      description: 'Site description',
      homepage: 'https://example.com',
      contact: 'ai@example.com',

      policies: {
        training: {
          allowed: false,
          note: 'Content not licensed for training'
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
      ttl: 3600000
    }
  }
});
```

---

## Usage Patterns

### Pattern 1: Static Pages with Chunks

```tsx
// app/page.tsx
export const arwConfig = {
  priority: 'high',
  purpose: 'homepage',
  chunks: [
    { id: 'hero', heading: 'Welcome' },
    { id: 'features', heading: 'Features' }
  ]
};

export default function HomePage() {
  return (
    <main>
      <section data-chunk-id="hero">
        <h1>Welcome</h1>
      </section>
      <section data-chunk-id="features">
        <h2>Features</h2>
      </section>
    </main>
  );
}
```

### Pattern 2: Dynamic Routes

```tsx
// app/posts/[slug]/page.tsx
import { generateARWMetadata } from '@agent-ready-web/nextjs-plugin/components';

export async function generateMetadata({ params }) {
  const post = await getPost(params.slug);

  return {
    title: post.title,
    ...generateARWMetadata({
      priority: 'normal',
      machineViewPath: `/posts/${params.slug}.llm.md`
    })
  };
}
```

### Pattern 3: API Routes

```typescript
// app/api/arw/manifest/route.ts
import { NextResponse } from 'next/server';

export async function GET() {
  const manifest = {
    version: '0.1',
    profile: 'ARW-1',
    // ... manifest data
  };

  return NextResponse.json(manifest);
}
```

---

## Best Practices

### 1. Co-locate Machine Views

Keep `.llm.md` files next to their components:

```
app/
├── about/
│   ├── page.tsx
│   └── page.llm.md
├── products/
│   ├── page.tsx
│   └── page.llm.md
```

### 2. Use Build-Time Generation

For static content, prefer build-time:

```javascript
arw: {
  autoGenerate: true,
  generation: {
    includePatterns: ['app/**/*.tsx']
  }
}
```

### 3. Implement Caching

For dynamic content, use caching:

```javascript
runtime: {
  cacheStrategy: 'memory',
  ttl: 3600000 // 1 hour
}
```

### 4. Monitor Performance

Track build time impact:

```bash
ANALYZE=true npm run build
```

### 5. Version Lock ARW Spec

Lock to specific ARW profile:

```javascript
manifest: {
  // ... other config
  profile: 'ARW-1' // Lock to specific version
}
```

---

## Migration

### From Manual Implementation

**Before:**
```tsx
<Head>
  <link rel="alternate" type="text/x-llm+markdown" href="/page.llm.md" />
</Head>
```

**After:**
```tsx
import { ARWHead } from '@agent-ready-web/nextjs-plugin/components';

<ARWHead priority="high" />
```

### From llms.txt

**Before:**
```
# llms.txt in public/
version: 0.1
profile: ARW-1
...
```

**After:**
```javascript
// next.config.js
module.exports = withARW({
  arw: {
    manifest: {
      siteName: 'My Site',
      // ... from llms.txt
    }
  }
});
```

---

## Troubleshooting

### Build Errors

**Error:** `Missing required manifest fields`

**Solution:** Ensure all required fields are present:
```javascript
manifest: {
  siteName: 'Required',
  homepage: 'https://required.com',
  contact: 'required@email.com'
}
```

### Runtime Issues

**Error:** `useARW must be used within ARWProvider`

**Solution:** Wrap app with `<ARWProvider>`:
```tsx
<ARWProvider>
  {children}
</ARWProvider>
```

### Performance Issues

**Issue:** Slow build times

**Solution:** Disable auto-generation or use selective patterns:
```javascript
generation: {
  includePatterns: ['app/public/**/*.tsx']
}
```

---

## Support

- [GitHub Issues](https://github.com/nolandubeau/agent-ready-web/issues)
- [Documentation](https://arw.dev)
- [Discussions](https://github.com/nolandubeau/agent-ready-web/discussions)

---

*Made with ❤️ for the Agent-Ready Web*
