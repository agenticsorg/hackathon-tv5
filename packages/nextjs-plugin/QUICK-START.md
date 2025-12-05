# Quick Start: Testing ARW Next.js Plugin

## 🚀 Fastest Way to Test

```bash
# 1. Navigate to basic example
cd packages/nextjs-plugin/examples/basic

# 2. Install dependencies
npm install

# 3. Start development server
npm run dev

# 4. Open browser
open http://localhost:3000
```

That's it! The example uses `file:../../` to link to the local plugin package.

## Alternative: Test Standalone

Copy the example outside the monorepo:

```bash
# 1. Copy example to separate directory
cp -r packages/nextjs-plugin/examples/basic ~/arw-test
cd ~/arw-test

# 2. Install plugin from npm (once published)
npm install @agent-ready-web/nextjs-plugin

# 3. Update next.config.js imports
# Change: const { withARW } = require('@agent-ready-web/nextjs-plugin');
# (No changes needed - already uses this)

# 4. Start dev server
npm run dev
```

## ✅ What You Should See

### Browser (http://localhost:3000)
- Homepage with "Welcome" heading
- "Features" section listing 3 items
- Clean, styled layout

### ARW Manifest (http://localhost:3000/.well-known/arw-manifest.json)
```json
{
  "version": "0.1",
  "profile": "ARW-1",
  "site": {
    "name": "ARW Next.js Basic Example",
    "homepage": "http://localhost:3000",
    "contact": "ai@example.com"
  }
}
```

### Machine View (http://localhost:3000/index.llm.md)
```markdown
# Welcome

<!-- chunk: hero -->
## Welcome
This is our homepage...
```

### HTML Source (View → Developer → View Source)
```html
<link rel="alternate" type="text/x-llm+markdown" href="/index.llm.md">
<link rel="alternate" type="application/json" href="/.well-known/arw-manifest.json">
<meta name="arw-priority" content="high">
```

## 🧪 Quick Tests

### Test 1: Manifest Endpoint
```bash
curl http://localhost:3000/.well-known/arw-manifest.json | jq
```

### Test 2: Machine View
```bash
curl http://localhost:3000/index.llm.md
```

### Test 3: Production Build
```bash
npm run build
npm run start
```

## 📁 All Examples

| Example | Location | Best For |
|---------|----------|----------|
| **Basic** | `examples/basic/` | Learning ARW integration |
| **Advanced** | `examples/advanced/` | Production setup |
| **Pages Router** | `examples/pages-router/` | Legacy apps |

## 🔧 Troubleshooting

### Can't find module?
```bash
# From repo root
npm install
npm run build
```

### Port already in use?
```bash
# Use different port
npm run dev -- -p 3001
```

### TypeScript errors?
```bash
# Rebuild types
cd ../../../
npm run build
```

## 📚 Next Steps

1. **Read full testing guide:** [TESTING.md](./TESTING.md)
2. **See all integration options:** [INTEGRATION-GUIDE.md](./INTEGRATION-GUIDE.md)
3. **Read plugin documentation:** [README.md](./README.md)
4. **Try advanced example:** `cd examples/advanced`

## 💡 Key Files to Explore

```
examples/basic/
├── next.config.js          # Plugin configuration
├── app/
│   ├── layout.tsx          # ARWProvider setup
│   └── page.tsx            # ARW metadata usage
└── public/
    ├── .well-known/
    │   └── arw-manifest.json   # ARW manifest
    └── index.llm.md            # Machine view
```

## 🎯 Success Checklist

- [ ] Dev server starts without errors
- [ ] Homepage loads at localhost:3000
- [ ] Manifest accessible at `/.well-known/arw-manifest.json`
- [ ] Machine view accessible at `/index.llm.md`
- [ ] HTML includes ARW metadata
- [ ] No console errors
- [ ] Production build succeeds

---

**Questions?** See [TESTING.md](./TESTING.md) for detailed guide.
