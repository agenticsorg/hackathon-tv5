# ARW Chrome Extension - Quick Start Guide

## Installation (2 minutes)

### Step 1: Navigate to Extension Directory
```bash
cd agent-ready-web/platform/extensions/arw-chrome-extension
```

### Step 2: Create Placeholder Icons
The extension needs icons in the `public/icons/` directory. For now, you can use placeholder icons or create simple ones:

**Option A: Create Placeholder Text Files (Temporary)**
```bash
mkdir -p public/icons
touch public/icons/icon16.png
touch public/icons/icon32.png
touch public/icons/icon48.png
touch public/icons/icon128.png
```

**Option B: Use Existing Icons (Recommended)**
Copy icons from another source or create simple 16x16, 32x32, 48x48, and 128x128 PNG files.

**Option C: Online Icon Generator**
Use a service like [favicon.io](https://favicon.io/) to generate icons with "ARW" text.

### Step 3: Load Extension in Chrome

1. **Open Chrome Extensions page:**
   ```
   chrome://extensions/
   ```
   Or: Menu → Extensions → Manage Extensions

2. **Enable Developer Mode:**
   - Toggle the switch in the top-right corner

3. **Load Unpacked Extension:**
   - Click "Load unpacked" button
   - Navigate to: `agent-ready-web/platform/extensions/arw-chrome-extension`
   - Click "Select Folder"

4. **Verify Installation:**
   - You should see "ARW Inspector - Agent-Ready Web" in the extensions list
   - The extension icon should appear in your toolbar (click puzzle icon to pin it)

## Testing (1 minute)

### Test on a Website

1. **Navigate to any website** (e.g., https://example.com)

2. **Click the ARW Inspector icon** in your toolbar

3. **View the inspection results:**
   - Badge shows ARW compliance (✓ or ✗)
   - Popup displays detailed findings
   - See which ARW features were found

### Expected Behavior

**On ARW-Compliant Sites:**
- Badge: ✓ ARW Compliant (green)
- llms.txt: Found
- .well-known files: May be found
- Machine views: May be found

**On Regular Sites:**
- Badge: ✗ Not Compliant (red)
- All features: Not Found
- This is expected and normal

## Using the Extension

### Automatic Inspection
- Runs on every page load
- No manual action needed
- Badge updates automatically

### View Details
1. Click extension icon
2. See page information
3. Check ARW features status
4. Click links to view ARW files

### Understanding Results

**Feature Status Icons:**
- ✓ = Found and working
- ✗ = Not found
- ⚠️ = Found but has warnings

**Compliance:**
- Green badge = Has llms.txt, .well-known, or machine views
- Red badge = No ARW features detected

## Debugging

### View Console Logs

**Content Script (page inspection):**
```
1. Right-click on page → Inspect
2. Console tab
3. Look for "ARW Inspector" messages
```

**Service Worker (background):**
```
1. Go to chrome://extensions/
2. Find "ARW Inspector"
3. Click "Service Worker" link
4. View console logs
```

**Popup (UI):**
```
1. Right-click extension popup → Inspect
2. Console tab
3. View popup logs
```

### Common Issues

**Issue: Extension won't load**
- Check you selected the correct folder
- Verify manifest.json exists in root
- Look for errors in chrome://extensions/

**Issue: No icon showing**
- Icons may be missing (see Step 2 above)
- Extension will still work, just no icon
- Add real icons later

**Issue: "No inspection data available"**
- Reload the page
- Wait for inspection to complete
- Check content script console for errors

**Issue: Badge not updating**
- Refresh the page
- Check service worker console
- Verify permissions are granted

## Next Steps

### 1. Add Real Icons
Replace placeholder icons with actual PNG files in `public/icons/`:
- icon16.png (16x16)
- icon32.png (32x32)
- icon48.png (48x48)
- icon128.png (128x128)

### 2. Test on Multiple Sites
Try these types of sites:
- Personal websites
- Documentation sites
- E-commerce sites
- News sites

### 3. Explore Code
- `src/content/content-script.js` - Inspection logic
- `src/background/service-worker.js` - Data management
- `src/popup/popup.js` - UI display

### 4. Customize
- Modify inspection logic
- Enhance UI design
- Add new features

## Development Workflow

### Making Changes

1. **Edit source files** in `src/`
2. **Go to chrome://extensions/**
3. **Click refresh icon** on ARW Inspector card
4. **Reload the page** you're testing on
5. **Click extension icon** to see changes

### Adding Features

See `README.md` for:
- Architecture overview
- Planned features
- Contributing guidelines

## Troubleshooting

### Extension Not Working

1. **Check permissions:**
   - Go to chrome://extensions/
   - Click "Details" on ARW Inspector
   - Verify "On all sites" is enabled

2. **Verify files:**
   ```bash
   ls -R src/
   # Should show: content/, background/, popup/
   ```

3. **Check manifest:**
   ```bash
   cat manifest.json | grep version
   # Should show: "version": "1.0.0"
   ```

### CORS Errors (Expected)

Some sites block cross-origin requests. This is normal:
- llms.txt: May fail on CORS-restricted sites
- .well-known: May fail on CORS-restricted sites
- Meta tags: Always work (no fetch needed)

The extension handles CORS errors gracefully and continues inspection.

## Resources

### Extension Documentation
- **Full Documentation:** See [ARW-100: Overview](../../docs/arw-chrome-extension/ARW-100-Chrome-Extension-Overview.md)
- **User Guide:** [ARW-102: User Guide](../../docs/arw-chrome-extension/guides/ARW-102-User-Guide.md)
- **Developer Guide:** [ARW-103: Developer Guide](../../docs/arw-chrome-extension/guides/ARW-103-Developer-Guide.md)

### Research & Reference
- **Chrome Extension Research:** [ARW-001: MV3 Research](../../docs/arw-chrome-extension/research/ARW-001-Chrome-Extension-MV3-Research.md)
- **Quick Reference:** [ARW-002: Quick Reference](../../docs/arw-chrome-extension/research/ARW-002-Chrome-Extension-Quick-Reference.md)
- **Research Summary:** [ARW-003: Research Summary](../../docs/arw-chrome-extension/research/ARW-003-Chrome-Extension-Research-Summary.md)

### Related Projects
- **ARW Inspector:** `platform/apps/arw-inspector/`
- **Chrome Extensions Guide:** https://developer.chrome.com/docs/extensions/

## Support

**Need Help?**
- Check `README.md` for detailed documentation
- Review console logs for errors
- Test on different websites
- Check Chrome extension permissions

**Found a Bug?**
- Note which site it occurred on
- Check console logs
- Document reproduction steps
- Review error messages

---

**That's it! You now have a working ARW Chrome Extension that inspects sites as you browse. 🎉**

Time to explore the web and discover ARW-compliant sites!
