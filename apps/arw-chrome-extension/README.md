# ARW Chrome Extension - Agent-Ready Web Inspector

A Chrome extension that automatically inspects websites for Agent-Ready Web (ARW) compliance as you browse. The extension discovers ARW features including llms.txt, machine views, .well-known files, and more.

## Features

### Automatic Page Inspection
- **Runs on every page load** - Content script automatically inspects each page
- **Real-time analysis** - Displays results immediately in the extension popup
- **Badge indicator** - Shows ARW compliance status in the extension icon

### ARW Discovery
Discovers all ARW aspects similar to the arw-inspector:

1. **llms.txt** - Primary ARW manifest at site root
2. **.well-known Files** - ARW manifest, content index, and policies
3. **Machine Views** - Detects .llm.md files for current page
4. **robots.txt** - Checks for ARW hints and directives
5. **Meta Tags** - Scans for ARW-related meta tags and links
6. **AI Headers** - Detects AI-specific HTTP headers

### UI Features
- **Compliance Badge** - Visual indicator of ARW compliance
- **Feature Breakdown** - Detailed status of each ARW feature
- **Direct Links** - Click to view discovered ARW files
- **Error Reporting** - Clear error messages for issues

## Installation

### Method 1: Load Unpacked Extension (Development)

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd agent-ready-web/platform/extensions/arw-chrome-extension
   ```

2. **Open Chrome and navigate to:**
   ```
   chrome://extensions/
   ```

3. **Enable Developer Mode:**
   - Toggle the "Developer mode" switch in the top right

4. **Load the extension:**
   - Click "Load unpacked"
   - Select the `arw-chrome-extension` directory

5. **Verify installation:**
   - The ARW Inspector icon should appear in your extensions bar
   - If not visible, click the puzzle icon and pin it

### Method 2: Build and Package

```bash
# Install dependencies (if any are added later)
npm install

# Build extension (if build step is added)
npm run build

# Create .zip for distribution
zip -r arw-chrome-extension.zip . -x "*.git*" -x "node_modules/*" -x "*.md"
```

## Usage

### Basic Usage

1. **Navigate to any website** - The extension automatically inspects the page
2. **Check the badge** - ✓ for ARW compliant, ✗ for non-compliant
3. **Open the popup** - Click the extension icon to view detailed results

### Understanding Results

#### Compliance Badge
- **✓ ARW Compliant** (Green) - Site has llms.txt, .well-known files, or machine views
- **✗ Not Compliant** (Red) - No ARW features detected
- **Checking...** (Gray) - Inspection in progress

#### Feature Status
Each ARW feature shows:
- **✓** - Feature found and working
- **✗** - Feature not found
- **⚠️** - Feature found but has issues
- **Details** - Additional information and links

### Testing the Extension

Test on these sites to see ARW discovery in action:

1. **ARW-compliant sites:**
   - Sites with llms.txt in root
   - Sites with .well-known/arw-manifest.json

2. **Non-compliant sites:**
   - Regular websites without ARW features
   - Should show "Not Compliant" badge

## Architecture

### Manifest V3 Structure
```
arw-chrome-extension/
├── manifest.json              # Extension configuration
├── src/
│   ├── content/
│   │   └── content-script.js  # Runs on every page, inspects ARW features
│   ├── background/
│   │   └── service-worker.js  # Coordinates extension, manages data
│   └── popup/
│       ├── popup.html         # Popup UI structure
│       ├── popup.css          # Popup styling
│       └── popup.js           # Popup logic, displays results
└── public/
    └── icons/                 # Extension icons (16, 32, 48, 128)
```

### How It Works

1. **Content Script** (`content-script.js`)
   - Runs on every page load (document_idle)
   - Inspects for ARW features:
     - Fetches llms.txt
     - Checks .well-known files
     - Scans robots.txt
     - Detects machine views
     - Scans meta tags
   - Sends results to service worker

2. **Background Service Worker** (`service-worker.js`)
   - Receives inspection results
   - Stores data per tab
   - Updates extension badge
   - Handles popup requests
   - Cleans up on tab close

3. **Popup** (`popup.html`, `popup.js`)
   - Requests data from service worker
   - Displays formatted results
   - Shows compliance status
   - Provides links to ARW files

### Message Flow
```
Page Load → Content Script → Inspect ARW Features
                ↓
         Send Results → Service Worker → Store Data
                ↓                             ↓
          Update Badge                   Wait for Popup
                                              ↓
User Opens Popup → Request Data → Display Results
```

## Development

### File Structure

**manifest.json** - Extension configuration
- Defines permissions, scripts, and resources
- Manifest V3 compliant

**content-script.js** - Page inspection logic
- Automatic execution on page load
- ARW feature discovery
- Result reporting

**service-worker.js** - Background coordination
- Data storage and retrieval
- Badge management
- Message handling

**popup.html/css/js** - User interface
- Results display
- Feature breakdown
- Link navigation

### Key Permissions

```json
{
  "permissions": [
    "activeTab",    // Access current tab
    "storage",      // Store inspection data
    "scripting"     // Script injection
  ],
  "host_permissions": [
    "<all_urls>"    // Run on all websites
  ]
}
```

### Debugging

1. **View Content Script Logs:**
   - Right-click on page → Inspect
   - Console tab shows content script logs

2. **View Service Worker Logs:**
   - Go to `chrome://extensions/`
   - Click "Service Worker" under ARW Inspector
   - View console logs

3. **View Popup Logs:**
   - Right-click popup → Inspect
   - Console tab shows popup logs

4. **Check Stored Data:**
   - Service worker console
   - Type: `chrome.storage.local.get(console.log)`

### Common Issues

**Issue: Badge not updating**
- Check content script is running (page console)
- Verify message passing (service worker console)

**Issue: No inspection data in popup**
- Check service worker is active
- Verify tab data is stored
- Reload page and try again

**Issue: CORS errors**
- Expected for some resources
- Extension handles gracefully
- Check network tab for details

## Comparison to arw-inspector

The Chrome extension uses similar logic to the arw-inspector web app:

| Feature | arw-inspector | Chrome Extension |
|---------|---------------|------------------|
| llms.txt discovery | ✓ | ✓ |
| .well-known files | ✓ | ✓ |
| Machine views | ✓ | ✓ |
| robots.txt analysis | ✓ | ✓ |
| Token estimation | ✓ | ✗ (future) |
| Cost comparison | ✓ | ✗ (future) |
| Automatic inspection | ✗ | ✓ |
| Browser integration | ✗ | ✓ |

## Future Enhancements

### Planned Features
- [ ] Token estimation for machine views
- [ ] Cost comparison display
- [ ] History of inspected sites
- [ ] Export inspection reports
- [ ] Customizable inspection depth
- [ ] Notification for ARW-compliant sites
- [ ] Dark mode support
- [ ] Settings page

### Advanced Features
- [ ] Content security policy analysis
- [ ] Chunk extraction and display
- [ ] Protocol support detection (MCP, OpenAPI)
- [ ] OAuth action discovery
- [ ] Sitemap XML parsing
- [ ] Performance metrics

## Contributing

Contributions are welcome! Areas for improvement:

1. **UI/UX** - Enhance popup design
2. **Features** - Add token estimation, cost comparison
3. **Performance** - Optimize inspection speed
4. **Testing** - Add automated tests
5. **Documentation** - Improve guides

## Resources

### Documentation
- [ARW-000: Documentation Index](../../docs/arw-chrome-extension/ARW-000-Documentation-Index.md)
- [ARW-100: Extension Overview](../../docs/arw-chrome-extension/ARW-100-Chrome-Extension-Overview.md)
- [ARW-101: Installation Guide](../../docs/arw-chrome-extension/guides/ARW-101-Installation-Guide.md)
- [ARW-102: User Guide](../../docs/arw-chrome-extension/guides/ARW-102-User-Guide.md)
- [ARW-103: Developer Guide](../../docs/arw-chrome-extension/guides/ARW-103-Developer-Guide.md)

### Research Materials
- [ARW-001: Chrome Extension MV3 Research](../../docs/arw-chrome-extension/research/ARW-001-Chrome-Extension-MV3-Research.md)
- [ARW-002: Quick Reference](../../docs/arw-chrome-extension/research/ARW-002-Chrome-Extension-Quick-Reference.md)
- [ARW-003: Research Summary](../../docs/arw-chrome-extension/research/ARW-003-Chrome-Extension-Research-Summary.md)

### External Resources
- [Chrome Extensions Documentation](https://developer.chrome.com/docs/extensions/)
- [Manifest V3 Migration Guide](https://developer.chrome.com/docs/extensions/mv3/intro/)
- [Agent-Ready Web Specification](https://github.com/agent-ready-web)
- [arw-inspector Source](../../apps/arw-inspector/)

## License

[Your License Here]

## Support

For issues, questions, or suggestions:
- GitHub Issues: [repository-url]/issues
- Documentation: [docs-url]
- Community: [community-url]

---

**Built with ❤️ for the Agent-Ready Web ecosystem**
