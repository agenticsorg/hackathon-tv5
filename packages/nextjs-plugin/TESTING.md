# Testing ARW Next.js Plugin Examples

This guide shows how to test the ARW Next.js plugin examples locally.

## Prerequisites

- Node.js >= 16.0.0
- npm, yarn, or pnpm
- Git

## Quick Start

### Option 1: Test Basic Example (Recommended)

```bash
# Navigate to example
cd packages/nextjs-plugin/examples/basic

# Install dependencies
npm install

# Start development server
npm run dev

# Open browser to http://localhost:3000
```

**Note:** The examples use `file:../../` to reference the local plugin, so they work with standard npm.

### Option 2: Test All Examples

```bash
# From repository root
cd packages/nextjs-plugin

# Install dependencies for all examples
npm run test:examples

# Or test individually
cd examples/basic && npm run dev
cd examples/advanced && npm run dev
cd examples/pages-router && npm run dev
```

## What to Test

### 1. Basic Example (App Router)

**Location:** `packages/nextjs-plugin/examples/basic`

**What to verify:**
- ✅ Homepage loads at http://localhost:3000
- ✅ ARW manifest at http://localhost:3000/.well-known/arw-manifest.json
- ✅ Machine view at http://localhost:3000/index.llm.md
- ✅ Page includes ARW metadata in HTML `<head>`
- ✅ No console errors

**Test commands:**
```bash
cd examples/basic

# Development
npm run dev

# Production build
npm run build
npm run start

# Check files
curl http://localhost:3000/.well-known/arw-manifest.json
curl http://localhost:3000/index.llm.md
```

### 2. Advanced Example

**Location:** `packages/nextjs-plugin/examples/advanced`

**What to verify:**
- ✅ Auto-generation enabled
- ✅ Custom output directory
- ✅ Advanced policies configured
- ✅ ARW validation works

**Test commands:**
```bash
cd examples/advanced

npm run dev
npm run build

# Validate ARW implementation
npm run arw:validate
```

### 3. Pages Router Example

**Location:** `packages/nextjs-plugin/examples/pages-router`

**What to verify:**
- ✅ Works with legacy Pages Router
- ✅ `<ARWHead />` component in `<Head>`
- ✅ Provider in `_app.tsx`

**Test commands:**
```bash
cd examples/pages-router

npm run dev
```

## Manual Testing Checklist

### Basic Functionality

- [ ] Development server starts without errors
- [ ] Homepage renders correctly
- [ ] No TypeScript errors
- [ ] No console warnings

### ARW Features

- [ ] ARW manifest is accessible
- [ ] Machine view file exists and is valid
- [ ] HTML includes ARW discovery headers
- [ ] Chunk IDs are present in HTML

### Integration

- [ ] `withARW()` plugin doesn't break Next.js config
- [ ] React hooks work (`useARW()`, `useARWEnabled()`)
- [ ] Components render (`<ARWHead />`, `<ARWProvider />`)
- [ ] TypeScript types are available

### Production Build

- [ ] Build completes successfully
- [ ] Production server starts
- [ ] ARW files are generated
- [ ] No build warnings

## Testing with Browser DevTools

### 1. Check ARW Headers

Open DevTools → Network tab:

```bash
# Check HTML response headers
curl -I http://localhost:3000/

# Should include:
# X-ARW-Version: 0.1
```

### 2. Inspect HTML Metadata

View page source:

```html
<!-- Should include -->
<link rel="alternate" type="text/x-llm+markdown" href="/index.llm.md">
<link rel="alternate" type="application/json" href="/.well-known/arw-manifest.json">
<meta name="arw-priority" content="high">
<meta name="arw-enabled" content="true">
```

### 3. Test React Hooks

Open DevTools → Console:

```javascript
// Test useARW hook (if exposed)
// Check React DevTools for ARWProvider state
```

## Testing ARW Endpoints

### Test Manifest Endpoint

```bash
curl http://localhost:3000/.well-known/arw-manifest.json | jq
```

Expected output:
```json
{
  "version": "0.1",
  "profile": "ARW-1",
  "site": {
    "name": "My Website",
    "homepage": "https://example.com",
    "contact": "ai@example.com"
  },
  "content": [...],
  "policies": {...}
}
```

### Test Machine View

```bash
curl http://localhost:3000/index.llm.md
```

Expected output:
```markdown
# Welcome

<!-- chunk: hero -->

## Welcome

This is our homepage...
```

### Test Rewrites

```bash
# /llms.txt should redirect to /.well-known/arw-manifest.json
curl http://localhost:3000/llms.txt
```

## Automated Testing

### Unit Tests (Future)

```bash
# Run unit tests
npm run test

# Run with coverage
npm run test:coverage
```

### E2E Tests (Future)

```bash
# Run end-to-end tests
npm run test:e2e
```

## Troubleshooting

### Issue: "Cannot find module '@agent-ready-web/nextjs-plugin'"

**Solution:**
```bash
# From repository root
npm install

# Or rebuild workspace
npm run build
```

### Issue: "Module not found: Can't resolve 'next/navigation'"

**Solution:**
```bash
# Ensure Next.js 13+ is installed
npm install next@latest
```

### Issue: ARW manifest returns 404

**Solution:**
```bash
# Check file exists
ls -la public/.well-known/arw-manifest.json

# Verify Next.js config
cat next.config.js
```

### Issue: TypeScript errors in components

**Solution:**
```bash
# Rebuild TypeScript definitions
cd ../../
npm run build

# Re-install in example
cd examples/basic
npm install
```

## Performance Testing

### Measure Build Time Impact

```bash
# Without ARW
time npm run build

# With ARW (autoGenerate: true)
# Compare build times
```

### Measure Bundle Size

```bash
# Build and analyze
npm run build
npx @next/bundle-analyzer
```

## Integration Testing Workflow

### 1. Fresh Install Test

```bash
# Remove node_modules
rm -rf node_modules package-lock.json

# Clean install
npm install

# Verify works
npm run dev
```

### 2. Production Build Test

```bash
# Build for production
npm run build

# Test production server
npm run start

# Verify all ARW endpoints work
```

### 3. Hot Reload Test

```bash
# Start dev server
npm run dev

# Edit page.tsx
# Verify hot reload works
# Check ARW metadata updates
```

## Testing Different Next.js Versions

```bash
# Test with Next.js 13
npm install next@13

# Test with Next.js 14
npm install next@14

# Test with latest
npm install next@latest
```

## CI/CD Testing

### GitHub Actions Example

```yaml
name: Test Examples

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Install dependencies
        run: npm install

      - name: Test basic example
        run: |
          cd packages/nextjs-plugin/examples/basic
          npm install
          npm run build

      - name: Test advanced example
        run: |
          cd packages/nextjs-plugin/examples/advanced
          npm install
          npm run build
```

## Manual Validation Steps

1. **Install and run basic example**
2. **Open http://localhost:3000**
3. **Check browser console** (no errors)
4. **View page source** (ARW metadata present)
5. **Test manifest endpoint** (JSON valid)
6. **Test machine view** (Markdown valid)
7. **Build for production** (no errors)
8. **Test production server** (all endpoints work)

## Expected Test Results

### ✅ Success Indicators

- Development server starts in < 5 seconds
- No console errors or warnings
- ARW manifest accessible and valid
- Machine views accessible
- HTML includes ARW metadata
- Production build completes
- TypeScript types available

### ❌ Failure Indicators

- Build errors
- Runtime errors
- Missing ARW endpoints (404s)
- Invalid JSON/Markdown
- TypeScript errors
- Console warnings

## Next Steps

After testing examples:

1. **Customize for your project**
2. **Add your content**
3. **Configure policies**
4. **Deploy to production**
5. **Monitor agent traffic**

## Support

Issues or questions?

- [GitHub Issues](https://github.com/nolandubeau/agent-ready-web/issues)
- [Discussions](https://github.com/nolandubeau/agent-ready-web/discussions)
- [Documentation](https://arw.dev)

---

Happy testing! 🚀
