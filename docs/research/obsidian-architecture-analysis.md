# Obsidian Architecture Analysis & Technical Specification

**Research Date:** 2025-12-07
**Purpose:** Comprehensive technical analysis for Rust-based Obsidian clone implementation
**Status:** Complete

---

## Executive Summary

Obsidian is an Electron-based note-taking application built with web technologies (JavaScript, HTML, CSS) that emphasizes local-first data storage, markdown files, and extensibility through plugins. As of December 2025, the ecosystem includes 2,692 community plugins, demonstrating a robust and thriving plugin architecture.

**Key Architecture Highlights:**
- **Platform:** Electron (desktop), Capacitor (mobile)
- **Storage:** Local markdown files with YAML frontmatter
- **Rendering:** CodeMirror 6 editor with custom HyperMD extensions
- **Plugin API:** TypeScript-based with ~400+ CSS variables for theming
- **Data Format:** Plain text .md files + JSON configuration files

---

## 1. Core Architecture

### 1.1 Technology Stack

**Desktop Application:**
- **Framework:** Electron (Chromium + Node.js)
- **Process Model:** Main process + Renderer processes
- **Packaging:** ASAR archives (read-only bundled files)
- **Languages:** JavaScript, TypeScript, CSS
- **Entry Point:** Specified in `package.json` main field

**Mobile Application:**
- **Framework:** Capacitor (cross-platform native runtime)
- **Platforms:** iOS, Android
- **Rendering:** Platform-specific browser components

**Important Note:** Obsidian is NOT available as a web app despite using web technologies.

### 1.2 Process Architecture (Electron)

```
Main Process
├── Responsible for application lifecycle
├── Access to Node.js APIs
├── Manages renderer processes
└── Handles system-level operations

Renderer Process(es)
├── Chromium-based browser window
├── Limited Node.js environment
├── Web APIs (primary interface)
├── IPC communication with Main Process
└── Runs Obsidian UI and editor
```

**Key Design Decision:** Obsidian runs in an Electron renderer process (Chromium browser window), emphasizing browser APIs over Node.js APIs. Plugins should use browser APIs where possible, with Node.js APIs loaded via `require()` only when necessary.

### 1.3 Application Entry Point

The `App` class serves as the central service locator, providing access to all major subsystems:

```typescript
interface App {
  vault: Vault;              // File system operations
  workspace: Workspace;       // UI panes and windows
  metadataCache: MetadataCache; // Cached file metadata
  // ... additional services
}
```

Plugins access the app instance via `this.app` from within their plugin class.

---

## 2. File System Structure

### 2.1 Vault Structure

A "vault" is Obsidian's term for a folder containing notes and configuration:

```
my-vault/
├── .obsidian/                    # Configuration directory
│   ├── app.json                  # Application settings
│   ├── appearance.json           # Theme and styling config
│   ├── workspace.json            # Desktop layout state
│   ├── workspace-mobile.json     # Mobile layout state
│   ├── workspaces.json           # Saved workspace layouts
│   ├── plugins/                  # Plugin data directories
│   │   ├── plugin-id/
│   │   │   └── data.json
│   ├── themes/                   # Custom themes
│   │   └── theme-name/
│   │       ├── manifest.json
│   │       └── theme.css
│   └── snippets/                 # CSS snippets
│       └── custom.css
├── notes/                        # User's notes (arbitrary structure)
│   ├── daily/
│   │   └── 2025-12-07.md
│   ├── projects/
│   └── reference/
└── attachments/                  # Media files (arbitrary location)
```

### 2.2 .obsidian Folder Details

**app.json:**
- Core application settings
- Plugin enable/disable state
- File and link preferences

**workspace.json / workspace-mobile.json:**
- Current pane layout structure
- Open files and their positions
- Sidebar states (collapsed/expanded)
- Active pane focus
- JSON format describing screen layout

**workspaces.json:**
- Named workspace configurations
- Allows switching between different layouts
- Used by core Workspaces plugin

### 2.3 Vault API

The Vault interface provides file system operations:

```typescript
interface Vault {
  // File Operations
  read(file: TFile): Promise<string>;
  modify(file: TFile, data: string): Promise<void>;
  create(path: string, data: string): Promise<TFile>;
  delete(file: TFile): Promise<void>;
  rename(file: TAbstractFile, newPath: string): Promise<void>;

  // Folder Operations
  createFolder(path: string): Promise<TFolder>;

  // File Retrieval
  getAbstractFileByPath(path: string): TAbstractFile | null;
  getMarkdownFiles(): TFile[];
  getFiles(): TFile[];

  // Recursive Operations
  static recurseChildren(folder: TFolder, callback: (file: TAbstractFile) => void): void;

  // Events
  on(name: 'create', callback: (file: TAbstractFile) => void): EventRef;
  on(name: 'modify', callback: (file: TAbstractFile) => void): EventRef;
  on(name: 'delete', callback: (file: TAbstractFile) => void): EventRef;
  on(name: 'rename', callback: (file: TAbstractFile, oldPath: string) => void): EventRef;
}

interface TAbstractFile {
  vault: Vault;
  path: string;
  name: string;
  parent: TFolder | null;
}

interface TFile extends TAbstractFile {
  basename: string;  // filename without extension
  extension: string;
  stat: { ctime: number; mtime: number; size: number };
}

interface TFolder extends TAbstractFile {
  children: TAbstractFile[];  // Array of TFile and TFolder
}
```

**Type Checking Pattern:**
```typescript
const fileOrFolder = this.app.vault.getAbstractFileByPath("path");
if (fileOrFolder instanceof TFile) {
  // It's a file
  const content = await this.app.vault.read(fileOrFolder);
} else if (fileOrFolder instanceof TFolder) {
  // It's a folder
  fileOrFolder.children.forEach(child => { /* ... */ });
}
```

---

## 3. Plugin System

### 3.1 Plugin API Overview

**Official Resources:**
- **API Repository:** https://github.com/obsidianmd/obsidian-api
- **Developer Docs:** https://docs.obsidian.md/
- **Sample Plugin:** https://github.com/obsidianmd/obsidian-sample-plugin
- **NPM Package:** `obsidian` (contains type definitions)

**Type Definition Files:**
- `obsidian.d.ts` - Core API (~400+ interfaces)
- `canvas.d.ts` - Canvas system API
- `publish.d.ts` - Publish system API

### 3.2 Plugin Structure

Every plugin extends the `Plugin` class:

```typescript
import { Plugin } from 'obsidian';

export default class MyPlugin extends Plugin {

  // Called when plugin is loaded
  async onload() {
    console.log('Loading plugin');

    // Register commands
    this.addCommand({
      id: 'my-command',
      name: 'My Command',
      callback: () => { /* ... */ }
    });

    // Register event handlers
    this.registerEvent(
      this.app.workspace.on('file-open', (file) => {
        console.log('File opened:', file.path);
      })
    );

    // Register DOM events
    this.registerDomEvent(document, 'click', (evt: MouseEvent) => {
      console.log('Click event');
    });

    // Register intervals
    this.registerInterval(
      window.setInterval(() => {
        console.log('Interval tick');
      }, 5000)
    );

    // Add ribbon icon
    this.addRibbonIcon('dice', 'My Plugin', () => {
      console.log('Ribbon clicked');
    });

    // Add settings tab
    this.addSettingTab(new MySettingTab(this.app, this));

    // Register view types
    this.registerView(
      'my-view-type',
      (leaf) => new MyCustomView(leaf)
    );

    // Load plugin data
    await this.loadData();
  }

  // Called when plugin is unloaded
  onunload() {
    console.log('Unloading plugin');
    // Cleanup is automatic for registered events/intervals
  }

  // Persist plugin data
  async loadData(): Promise<any> {
    return await this.app.vault.adapter.read(this.manifest.dir + '/data.json');
  }

  async saveData(data: any): Promise<void> {
    await this.app.vault.adapter.write(
      this.manifest.dir + '/data.json',
      JSON.stringify(data)
    );
  }
}
```

### 3.3 Plugin Lifecycle

**1. Initialization:**
- Plugin manifest loaded from `manifest.json`
- Plugin class instantiated
- `onload()` method called

**2. Active State:**
- Event handlers active
- Commands registered
- UI components rendered

**3. Cleanup:**
- `onunload()` method called
- Automatic cleanup of:
  - Events registered with `registerEvent()`
  - DOM events registered with `registerDomEvent()`
  - Intervals registered with `registerInterval()`
  - UI elements added by the plugin

### 3.4 Component Base Class

Both `Plugin` and `MarkdownRenderChild` extend `Component`:

```typescript
class Component {
  // Register event for automatic cleanup
  registerEvent(event: EventRef): void;

  // Register DOM event for automatic cleanup
  registerDomEvent(
    target: Window | Document | HTMLElement,
    type: string,
    callback: (event: any) => void,
    options?: boolean | AddEventListenerOptions
  ): void;

  // Register interval for automatic cleanup
  registerInterval(interval: number): number;

  // Add child component
  addChild<T extends Component>(component: T): T;

  // Manual cleanup trigger
  load(): void;
  unload(): void;
}
```

**Best Practice:** Always use `register*` methods instead of directly calling `addEventListener()` or `setInterval()` to prevent memory leaks.

### 3.5 Event System

**Vault Events:**
```typescript
this.app.vault.on('create', (file) => { /* file created */ });
this.app.vault.on('modify', (file) => { /* file modified */ });
this.app.vault.on('delete', (file) => { /* file deleted */ });
this.app.vault.on('rename', (file, oldPath) => { /* file renamed */ });
```

**Workspace Events:**
```typescript
this.app.workspace.on('file-open', (file) => { /* file opened */ });
this.app.workspace.on('layout-change', () => { /* layout changed */ });
this.app.workspace.on('active-leaf-change', (leaf) => { /* active pane changed */ });
```

**MetadataCache Events:**
```typescript
this.app.metadataCache.on('changed', (file, data, cache) => {
  // File indexed, cache available
  // Note: Not called on file rename for performance
});

this.app.metadataCache.on('resolve', (file) => {
  // File resolved for links
});

this.app.metadataCache.on('resolved', () => {
  // All files resolved (initial load complete)
});
```

### 3.6 Plugin Manifest

Every plugin requires a `manifest.json`:

```json
{
  "id": "my-plugin-id",
  "name": "My Plugin Name",
  "version": "1.0.0",
  "minAppVersion": "0.15.0",
  "description": "Description of the plugin",
  "author": "Author Name",
  "authorUrl": "https://example.com",
  "isDesktopOnly": false
}
```

### 3.7 Inter-Plugin Communication

Plugins can expose APIs to other plugins:

```typescript
// In plugin A
export default class PluginA extends Plugin {
  public api = {
    getData: () => { return this.data; },
    performAction: (param: string) => { /* ... */ }
  };

  onload() {
    // Register API globally
    (window as any).PluginAAPI = this.api;
  }
}

// In plugin B
const pluginAAPI = (window as any).PluginAAPI;
if (pluginAAPI) {
  const data = pluginAAPI.getData();
}
```

---

## 4. File Formats

### 4.1 Markdown Files (.md)

Obsidian uses standard Markdown with custom extensions:

```markdown
---
title: Note Title
tags: [tag1, tag2]
created: 2025-12-07
modified: 2025-12-07
aliases: [Alternative Name]
cssclasses: [custom-class]
---

# Note Content

Regular markdown content with [[wikilinks]] and #tags.

## Custom Syntax Extensions

- [[Internal Link]] - Wikilink to another note
- [[Note#Heading]] - Link to specific heading
- [[Note#^block-id]] - Link to specific block
- ![[Embedded Note]] - Embed another note
- ![[image.png]] - Embed image
- ==Highlighted text== - Highlight
- %%Hidden comment%% - Comment (not rendered)

## Code Blocks with Syntax Highlighting

```javascript
const example = "code";
```

## Task Lists

- [ ] Uncompleted task
- [x] Completed task

## Tables

| Column 1 | Column 2 |
| -------- | -------- |
| Value 1  | Value 2  |
```

### 4.2 YAML Frontmatter

Frontmatter is enclosed by triple dashes and uses YAML format:

```yaml
---
title: My Note Title
tags: [research, important]
created: 2025-12-07T10:30:00
modified: 2025-12-07T15:45:00
author: John Doe
status: draft
rating: 5
aliases:
  - Alternative Title
  - Another Alias
cssclasses:
  - wide-page
  - custom-style
custom_field: custom value
nested:
  property: value
---
```

**Common Fields:**
- `title` - Note title (overrides filename)
- `tags` - Array of tags
- `aliases` - Alternative names for the note
- `cssclasses` - CSS classes to apply
- `created` / `modified` - Timestamps
- Custom fields - Any key-value pairs

**Access in Plugins:**
```typescript
const cache = this.app.metadataCache.getFileCache(file);
if (cache?.frontmatter) {
  const title = cache.frontmatter.title;
  const tags = cache.frontmatter.tags || [];
}
```

### 4.3 JSON Configuration Files

**app.json:**
```json
{
  "alwaysUpdateLinks": true,
  "attachmentFolderPath": "./attachments",
  "newFileLocation": "folder",
  "newFileFolderPath": "notes",
  "showUnsupportedFiles": false,
  "useMarkdownLinks": false
}
```

**workspace.json:**
```json
{
  "main": {
    "id": "unique-id",
    "type": "split",
    "children": [
      {
        "id": "pane-id",
        "type": "leaf",
        "state": {
          "type": "markdown",
          "state": {
            "file": "notes/example.md",
            "mode": "source"
          }
        }
      }
    ]
  },
  "left": {
    "id": "left-sidebar",
    "type": "split",
    "children": [
      {
        "id": "file-explorer",
        "type": "leaf",
        "state": {
          "type": "file-explorer"
        }
      }
    ]
  }
}
```

### 4.4 Canvas Files (.canvas)

Obsidian Canvas uses the **JSON Canvas** open file format:

**Specification:** https://jsoncanvas.org/
**Repository:** https://github.com/obsidianmd/jsoncanvas
**License:** MIT (free for commercial use)

```json
{
  "nodes": [
    {
      "id": "unique-node-id",
      "type": "text",
      "x": 100,
      "y": 200,
      "width": 300,
      "height": 150,
      "text": "# Node Content\n\nMarkdown text"
    },
    {
      "id": "file-node-id",
      "type": "file",
      "x": 500,
      "y": 200,
      "width": 400,
      "height": 300,
      "file": "notes/reference.md"
    },
    {
      "id": "link-node-id",
      "type": "link",
      "x": 100,
      "y": 500,
      "width": 300,
      "height": 200,
      "url": "https://example.com"
    },
    {
      "id": "group-node-id",
      "type": "group",
      "x": 0,
      "y": 0,
      "width": 1000,
      "height": 800,
      "label": "Group Label"
    }
  ],
  "edges": [
    {
      "id": "edge-id",
      "fromNode": "unique-node-id",
      "toNode": "file-node-id",
      "fromSide": "right",
      "toSide": "left",
      "color": "3",
      "label": "Connection Label"
    }
  ]
}
```

**Node Types:**
- `text` - Text content (supports Markdown)
- `file` - Reference to vault file
- `link` - External URL
- `group` - Container for organizing nodes

**Edge Properties:**
- Connection between two nodes
- Optional label and color
- Side specification (top, right, bottom, left)

**Libraries Available:**
C, Dart, Go, Python, React, Rust, TypeScript

---

## 5. Markdown Rendering Engine

### 5.1 CodeMirror 6 Integration

Obsidian uses **CodeMirror 6** for the editor with "Live Preview" mode:

**Key Features:**
- Real-time syntax highlighting
- WYSIWYG-style editing (HyperMD mode)
- Syntax tree parsing
- Custom extensions system

### 5.2 Editor Extension API

Plugins can extend the editor using CodeMirror 6 extensions:

```typescript
import { EditorView, ViewPlugin, Decoration } from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';

export default class MyPlugin extends Plugin {
  onload() {
    // Register editor extension
    this.registerEditorExtension([
      ViewPlugin.fromClass(class {
        decorations: DecorationSet;

        constructor(view: EditorView) {
          this.decorations = this.buildDecorations(view);
        }

        update(update: ViewUpdate) {
          if (update.docChanged || update.viewportChanged) {
            this.decorations = this.buildDecorations(update.view);
          }
        }

        buildDecorations(view: EditorView): DecorationSet {
          const widgets: Range<Decoration>[] = [];

          for (let { from, to } of view.visibleRanges) {
            syntaxTree(view.state).iterate({
              from,
              to,
              enter: (node) => {
                // Parse syntax tree and add decorations
                if (node.name === 'formatting-hashtag') {
                  widgets.push(
                    Decoration.mark({
                      class: 'custom-tag-style'
                    }).range(node.from, node.to)
                  );
                }
              }
            });
          }

          return Decoration.set(widgets);
        }
      }, {
        decorations: v => v.decorations
      })
    ]);
  }
}
```

**Important:** Use the exact CodeMirror 6 classes from Obsidian's environment to ensure compatibility.

### 5.3 Markdown Post-Processing

Register custom markdown renderers:

```typescript
this.registerMarkdownPostProcessor((element, context) => {
  // Process rendered markdown
  const codeblocks = element.querySelectorAll('code');

  codeblocks.forEach(code => {
    // Custom rendering logic
    if (code.textContent?.startsWith('custom:')) {
      const widget = document.createElement('div');
      widget.className = 'custom-widget';
      widget.textContent = 'Custom rendered content';
      code.replaceWith(widget);
    }
  });
});
```

### 5.4 HyperMD Mode

Obsidian uses a modified version of HyperMD for WYSIWYG functionality:

**Features:**
- Inline image rendering
- Link preview on hover
- Collapsible headings
- Real-time formatting (bold, italic, etc.)
- Task list interactivity

---

## 6. MetadataCache System

### 6.1 Overview

The MetadataCache stores parsed metadata from all markdown files, enabling fast queries without re-parsing files.

**Supports:**
- File search
- Graph view generation
- Backlink counting
- Tag queries
- Link resolution

### 6.2 MetadataCache API

```typescript
interface MetadataCache {
  // Get cached metadata for file
  getFileCache(file: TFile): CachedMetadata | null;

  // Get all resolved links (link counts)
  resolvedLinks: Record<string, Record<string, number>>;

  // Get all unresolved links
  unresolvedLinks: Record<string, Record<string, number>>;

  // Events
  on(name: 'changed', callback: (file: TFile, data: string, cache: CachedMetadata) => void): EventRef;
  on(name: 'resolve', callback: (file: TFile) => void): EventRef;
  on(name: 'resolved', callback: () => void): EventRef;
}

interface CachedMetadata {
  // Frontmatter data
  frontmatter?: Record<string, any>;

  // Links [[...]]
  links?: LinkCache[];

  // Embeds ![[...]]
  embeds?: EmbedCache[];

  // Tags #tag
  tags?: TagCache[];

  // Headings
  headings?: HeadingCache[];

  // Code blocks
  sections?: SectionCache[];

  // List items
  listItems?: ListItemCache[];

  // Blocks ^block-id
  blocks?: Record<string, BlockCache>;
}

interface LinkCache {
  link: string;       // Link text
  original: string;   // Original markdown
  displayText?: string;
  position: Pos;      // Start/end line and column
}

interface Pos {
  start: { line: number; col: number; offset: number };
  end: { line: number; col: number; offset: number };
}
```

### 6.3 Link Resolution

**Resolved Links Structure:**
```typescript
// Maps source file -> destination file -> link count
{
  "notes/source.md": {
    "notes/destination.md": 3,
    "notes/other.md": 1
  }
}
```

**Unresolved Links Structure:**
```typescript
// Maps source file -> broken link text -> count
{
  "notes/source.md": {
    "NonExistentNote": 2,
    "AnotherBrokenLink": 1
  }
}
```

**Linktext Components:**
- **linktext:** Full link like `My note#Heading`
- **linkpath (path):** Path part: `My note`
- **subpath:** Heading/block part: `#Heading`

### 6.4 Usage Example

```typescript
// Get metadata for current file
const activeFile = this.app.workspace.getActiveFile();
if (activeFile) {
  const cache = this.app.metadataCache.getFileCache(activeFile);

  if (cache) {
    // Access frontmatter
    const tags = cache.frontmatter?.tags || [];

    // Get all links
    const links = cache.links || [];
    links.forEach(link => {
      console.log(`Link to: ${link.link}`);
    });

    // Get headings
    const headings = cache.headings || [];
    headings.forEach(heading => {
      console.log(`${heading.level} ${heading.heading}`);
    });
  }

  // Get backlinks to this file
  const backlinks = Object.keys(this.app.metadataCache.resolvedLinks)
    .filter(sourcePath => {
      return this.app.metadataCache.resolvedLinks[sourcePath][activeFile.path];
    });
}
```

---

## 7. Key Features Implementation Patterns

### 7.1 Graph View

**Architecture:**
- Uses force-directed graph layout algorithms
- Nodes represent notes
- Edges represent links (resolved links from MetadataCache)
- Real-time updates via MetadataCache events

**Data Source:**
```typescript
const nodes = this.app.vault.getMarkdownFiles().map(file => ({
  id: file.path,
  label: file.basename
}));

const edges: Edge[] = [];
Object.entries(this.app.metadataCache.resolvedLinks).forEach(([source, targets]) => {
  Object.keys(targets).forEach(target => {
    edges.push({ source, target });
  });
});
```

**Filtering:**
- Search by note name
- Filter by tags
- Depth limiting (show only N levels of connections)
- Orphan nodes (notes with no links)

**Visualization:**
- Canvas/WebGL rendering
- Interactive (drag nodes, zoom, pan)
- Color coding by tags or folders
- Size based on connection count

### 7.2 Backlinks and Forward Links

**Backlinks:** Notes that link TO the current note
**Forward Links:** Notes that the current note links TO

```typescript
function getBacklinks(file: TFile): string[] {
  const backlinks: string[] = [];

  Object.entries(this.app.metadataCache.resolvedLinks).forEach(([source, targets]) => {
    if (targets[file.path]) {
      backlinks.push(source);
    }
  });

  return backlinks;
}

function getForwardLinks(file: TFile): string[] {
  const cache = this.app.metadataCache.getFileCache(file);
  return (cache?.links || []).map(link => link.link);
}
```

**Unlinked Mentions:**
Files that mention the note name but don't have explicit links:

```typescript
async function getUnlinkedMentions(file: TFile): Promise<string[]> {
  const mentions: string[] = [];
  const searchTerm = file.basename;

  for (const f of this.app.vault.getMarkdownFiles()) {
    if (f.path === file.path) continue;

    const content = await this.app.vault.read(f);
    if (content.includes(searchTerm)) {
      // Check if it's not already a link
      const cache = this.app.metadataCache.getFileCache(f);
      const hasLink = cache?.links?.some(link => link.link === file.basename);

      if (!hasLink) {
        mentions.push(f.path);
      }
    }
  }

  return mentions;
}
```

### 7.3 Search Functionality

**Search Implementation:**
- Full-text search across all markdown files
- Metadata search (tags, frontmatter)
- Regular expression support
- Search operators: `tag:#tagname`, `file:filename`, `path:folder/`

**Search API Pattern:**
```typescript
interface SearchQuery {
  query: string;
  caseSensitive: boolean;
  regex: boolean;
  includeContent: boolean;
  includeTags: boolean;
  includeFrontmatter: boolean;
}

async function search(query: SearchQuery): Promise<SearchResult[]> {
  const results: SearchResult[] = [];

  for (const file of this.app.vault.getMarkdownFiles()) {
    const content = await this.app.vault.read(file);
    const cache = this.app.metadataCache.getFileCache(file);

    // Search in content
    if (query.includeContent && matches(content, query)) {
      results.push({
        file,
        matches: findMatches(content, query)
      });
    }

    // Search in tags
    if (query.includeTags && cache?.tags) {
      // Tag search logic
    }

    // Search in frontmatter
    if (query.includeFrontmatter && cache?.frontmatter) {
      // Frontmatter search logic
    }
  }

  return results;
}
```

### 7.4 Daily Notes

**Implementation:**
- Core plugin (can be enabled/disabled)
- Creates note based on date format template
- Configurable folder location
- Template support with date variables

**Configuration:**
```typescript
interface DailyNotesSettings {
  format: string;          // "YYYY-MM-DD"
  folder: string;          // "daily/"
  template: string;        // "templates/daily-note.md"
  autorun: boolean;        // Open on startup
}
```

**API (from obsidian-daily-notes-interface package):**
```typescript
import {
  createDailyNote,
  getDailyNote,
  getAllDailyNotes
} from 'obsidian-daily-notes-interface';

// Create daily note for specific date
const note = await createDailyNote(moment("2025-12-07"));

// Get daily note (if exists)
const today = getDailyNote(moment(), dailyNotesMap);

// Get all daily notes
const allDailyNotes = getAllDailyNotes();
```

**Date UID System:**
- Unique identifier for notes by date
- Format: `day-YYYY-MM-DD`, `week-YYYY-WW`, `month-YYYY-MM`
- Enables fast note lookup

### 7.5 Templates

**Template System:**
- Plain markdown files with variable substitution
- Date variables: `{{date}}`, `{{time}}`
- Custom variables via plugins (Templater)
- Applied on note creation

**Template Plugin Features:**
- Insert template into current note
- Date format customization
- Folder templates (auto-apply to new notes in folder)

**Templater Plugin (Community - Very Popular):**
```markdown
---
created: <% tp.date.now("YYYY-MM-DD") %>
title: <% tp.file.title %>
---

# <% tp.file.title %>

Created: <% tp.date.now("HH:mm") %>

## Daily Log

<% tp.file.cursor() %>
```

**Templater API:**
- File operations: `tp.file.*`
- Date operations: `tp.date.*`
- System operations: `tp.system.*`
- User scripts: `tp.user.*`
- Dynamic JavaScript execution

### 7.6 Live Preview / Edit Modes

**Source Mode:**
- Raw markdown editing
- Syntax highlighting
- No WYSIWYG rendering

**Live Preview Mode:**
- Hybrid editing/preview
- Markdown formatting applied in real-time
- Click to edit specific elements
- CodeMirror 6 with HyperMD extensions

**Reading Mode:**
- Fully rendered markdown
- No editing capabilities
- Optimized for reading

**Mode Switching:**
```typescript
// Get active editor
const editor = this.app.workspace.activeEditor?.editor;

// Get/set mode
const currentMode = this.app.workspace.activeLeaf?.getViewState().state.mode;

// Switch mode
this.app.workspace.activeLeaf?.setViewState({
  type: 'markdown',
  state: {
    file: currentFile.path,
    mode: 'source' // or 'preview'
  }
});
```

---

## 8. UI Components

### 8.1 Workspace Layout

**Workspace Architecture:**
```
┌─────────────────────────────────────────────────┐
│ Title Bar                                        │
├───┬─────────────────────────────────────────┬───┤
│ L │ Main Workspace Area                     │ R │
│ e │  ┌─────────┬─────────┬─────────┐       │ i │
│ f │  │ Pane 1  │ Pane 2  │ Pane 3  │       │ g │
│ t │  │         │         │         │       │ h │
│   │  │         │         │         │       │ t │
│ S │  │         │         │         │       │   │
│ i │  └─────────┴─────────┴─────────┘       │ S │
│ d │                                         │ i │
│ e │                                         │ d │
│ b │                                         │ e │
│ a │                                         │ b │
│ r │                                         │ a │
│   │                                         │ r │
├───┴─────────────────────────────────────────┴───┤
│ Status Bar                                      │
└─────────────────────────────────────────────────┘
```

**Split Types:**
- **Horizontal Split:** Side-by-side panes
- **Vertical Split:** Stacked panes
- **Nested Splits:** Recursive splitting

### 8.2 Workspace API

```typescript
interface Workspace {
  // Active elements
  activeLeaf: WorkspaceLeaf | null;

  // Layout access
  leftSplit: WorkspaceSplit;
  rightSplit: WorkspaceSplit;
  rootSplit: WorkspaceSplit;

  // Leaf management
  getLeaf(newLeaf?: boolean): WorkspaceLeaf;
  createLeafBySplit(leaf: WorkspaceLeaf, direction?: 'horizontal' | 'vertical'): WorkspaceLeaf;

  // Active file
  getActiveFile(): TFile | null;
  getActiveViewOfType<T extends View>(type: new (...args: any[]) => T): T | null;

  // Layout operations
  setActiveLeaf(leaf: WorkspaceLeaf, pushHistory?: boolean): void;
  splitLeaf(leaf: WorkspaceLeaf, direction: 'horizontal' | 'vertical'): WorkspaceLeaf;

  // Modal and menu
  open(view: ItemView): void;
  revealLeaf(leaf: WorkspaceLeaf): void;

  // Events
  on(name: 'file-open', callback: (file: TFile) => void): EventRef;
  on(name: 'layout-change', callback: () => void): EventRef;
  on(name: 'active-leaf-change', callback: (leaf: WorkspaceLeaf | null) => void): EventRef;
}

interface WorkspaceLeaf {
  view: View;

  // State management
  setViewState(viewState: ViewState): Promise<void>;
  getViewState(): ViewState;

  // Navigation
  openFile(file: TFile, openState?: OpenViewState): Promise<void>;

  // Detachment
  detach(): void;
}
```

**Example: Open File in New Pane**
```typescript
// Get new leaf (creates if needed)
const leaf = this.app.workspace.getLeaf('tab');

// Open file in leaf
await leaf.openFile(file);

// Or set view state directly
await leaf.setViewState({
  type: 'markdown',
  state: {
    file: file.path,
    mode: 'source'
  }
});
```

### 8.3 Sidebar System

**Left Sidebar (Default Views):**
- File explorer
- Search
- Starred notes
- Tags view
- Outline (current file)

**Right Sidebar (Default Views):**
- Backlinks
- Outgoing links
- Tags (for current file)
- Outline

**Sidebar API:**
```typescript
// Toggle sidebar
this.app.workspace.leftSplit.toggle();
this.app.workspace.rightSplit.toggle();

// Collapse/expand
this.app.workspace.leftSplit.collapse();
this.app.workspace.leftSplit.expand();

// Get sidebar leaves
const leftLeaves = this.app.workspace.leftSplit.children;
```

### 8.4 Ribbon (Left Icon Bar)

```typescript
// Add ribbon icon
this.addRibbonIcon(
  'dice',              // Icon name (Lucide icon or custom)
  'My Plugin',         // Tooltip
  (evt: MouseEvent) => {
    // Click handler
    console.log('Ribbon clicked');
  }
);
```

**Available Icons:**
- Obsidian uses Lucide icons (https://lucide.dev/)
- Custom SVG icons can be registered
- Feather Icons also supported via plugins

### 8.5 Command Palette

**Architecture:**
- Fuzzy search over all commands
- Keyboard shortcut display
- Recent commands prioritization

**Command Registration:**
```typescript
this.addCommand({
  id: 'my-command-id',
  name: 'My Command Name',

  // Optional: Only show when markdown view active
  editorCallback: (editor: Editor, view: MarkdownView) => {
    // Command implementation with editor access
    editor.replaceSelection('Inserted text');
  },

  // Optional: Keyboard shortcut
  hotkeys: [
    {
      modifiers: ['Mod', 'Shift'],
      key: 'k'
    }
  ],

  // Optional: Conditional availability
  checkCallback: (checking: boolean) => {
    const active = this.app.workspace.getActiveFile();
    if (active) {
      if (!checking) {
        // Execute command
        console.log('Command executed');
      }
      return true;
    }
    return false;
  }
});
```

**Opening Command Palette:**
- Default shortcut: `Ctrl/Cmd + P`
- Programmatic: Not directly accessible via API

### 8.6 Tab System

**Tab Types:**
- **Tab Groups:** Container for tabs
- **Tabs:** Individual open files/views
- **Pinned Tabs:** Stay open and left-aligned

**Tab Navigation:**
```typescript
// Get all leaves (tabs)
const leaves = this.app.workspace.getLeavesOfType('markdown');

// Close tab
leaf.detach();

// Pin functionality (requires plugin)
```

**Tab Plugins:**
- Tab Navigator - Fuzzy search tabs
- Autofit Tabs - Dynamic width adjustment

### 8.7 Modals

**Basic Modal:**
```typescript
import { Modal } from 'obsidian';

class MyModal extends Modal {
  onOpen() {
    const { contentEl } = this;

    contentEl.createEl('h2', { text: 'Modal Title' });

    contentEl.createEl('p', {
      text: 'Modal content goes here.'
    });

    // Add button
    const button = contentEl.createEl('button', {
      text: 'Close'
    });
    button.addEventListener('click', () => {
      this.close();
    });
  }

  onClose() {
    const { contentEl } = this;
    contentEl.empty();
  }
}

// Open modal
new MyModal(this.app).open();
```

**Suggest Modal (Autocomplete):**
```typescript
import { SuggestModal } from 'obsidian';

interface Item {
  name: string;
  value: string;
}

class MySuggestModal extends SuggestModal<Item> {
  getSuggestions(query: string): Item[] {
    // Return filtered suggestions
    return [
      { name: 'Option 1', value: 'opt1' },
      { name: 'Option 2', value: 'opt2' }
    ].filter(item =>
      item.name.toLowerCase().includes(query.toLowerCase())
    );
  }

  renderSuggestion(item: Item, el: HTMLElement) {
    el.createEl('div', { text: item.name });
  }

  onChooseSuggestion(item: Item, evt: MouseEvent | KeyboardEvent) {
    // Handle selection
    console.log('Selected:', item.value);
  }
}
```

### 8.8 Settings Tab

```typescript
import { PluginSettingTab, Setting } from 'obsidian';

interface MyPluginSettings {
  mySetting: string;
  enabled: boolean;
  numberValue: number;
}

class MySettingTab extends PluginSettingTab {
  plugin: MyPlugin;

  constructor(app: App, plugin: MyPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;

    containerEl.empty();

    // Text setting
    new Setting(containerEl)
      .setName('My Setting')
      .setDesc('Description of the setting')
      .addText(text => text
        .setPlaceholder('Enter value')
        .setValue(this.plugin.settings.mySetting)
        .onChange(async (value) => {
          this.plugin.settings.mySetting = value;
          await this.plugin.saveSettings();
        }));

    // Toggle setting
    new Setting(containerEl)
      .setName('Enable Feature')
      .setDesc('Toggle this feature on/off')
      .addToggle(toggle => toggle
        .setValue(this.plugin.settings.enabled)
        .onChange(async (value) => {
          this.plugin.settings.enabled = value;
          await this.plugin.saveSettings();
        }));

    // Slider setting
    new Setting(containerEl)
      .setName('Number Value')
      .setDesc('Adjust numeric value')
      .addSlider(slider => slider
        .setLimits(0, 100, 1)
        .setValue(this.plugin.settings.numberValue)
        .onChange(async (value) => {
          this.plugin.settings.numberValue = value;
          await this.plugin.saveSettings();
        })
        .setDynamicTooltip());

    // Dropdown setting
    new Setting(containerEl)
      .setName('Select Option')
      .setDesc('Choose from dropdown')
      .addDropdown(dropdown => dropdown
        .addOption('option1', 'Option 1')
        .addOption('option2', 'Option 2')
        .setValue('option1')
        .onChange(async (value) => {
          // Handle change
        }));
  }
}
```

---

## 9. Theme System

### 9.1 CSS Variable Architecture

Obsidian exposes **400+ CSS variables** for customization:

**Variable Categories:**
1. **Foundations** - Base values (colors, typography, spacing)
2. **Components** - UI component styling
3. **Editor** - Markdown editor styling
4. **Canvas** - Canvas view styling
5. **Graph** - Graph view styling
6. **Publish** - Publish site styling

### 9.2 Common CSS Variables

```css
/* Base Colors (00=lightest to 100=darkest) */
--color-base-00: hsl(0, 0%, 100%);
--color-base-10: hsl(0, 0%, 95%);
--color-base-20: hsl(0, 0%, 90%);
...
--color-base-100: hsl(0, 0%, 0%);

/* Accent Color (HSL components) */
--accent-h: 254;
--accent-s: 80%;
--accent-l: 68%;

/* Semantic Colors */
--background-primary: var(--color-base-00);
--background-secondary: var(--color-base-10);
--text-normal: var(--color-base-100);
--text-muted: var(--color-base-70);
--text-faint: var(--color-base-50);

/* Interactive Elements */
--interactive-normal: var(--color-base-20);
--interactive-hover: var(--color-base-30);
--interactive-accent: var(--color-accent);

/* Extended Color Palette */
--color-red: #e74c3c;
--color-green: #2ecc71;
--color-blue: #3498db;
--color-yellow: #f39c12;
--color-purple: #9b59b6;

/* Typography */
--font-text: 'Inter', -apple-system, system-ui, sans-serif;
--font-monospace: 'JetBrains Mono', monospace;
--font-interface: var(--font-text);

--font-text-size: 16px;
--font-line-height: 1.6;

/* Spacing */
--size-2-1: 2px;
--size-2-2: 4px;
--size-2-3: 8px;
--size-4-1: 4px;
--size-4-2: 8px;
--size-4-3: 12px;
```

### 9.3 Theme File Structure

**manifest.json:**
```json
{
  "name": "My Theme",
  "version": "1.0.0",
  "minAppVersion": "0.16.0",
  "author": "Theme Author",
  "authorUrl": "https://example.com"
}
```

**theme.css:**
```css
/* Theme root selectors */
.theme-light {
  /* Light theme variables */
  --background-primary: #ffffff;
  --text-normal: #000000;
}

.theme-dark {
  /* Dark theme variables */
  --background-primary: #1e1e1e;
  --text-normal: #ffffff;
}

/* Body selector (applies to both themes) */
body {
  /* Global overrides */
}

/* App container */
.app-container {
  /* App-wide styling */
}
```

### 9.4 CSS Snippets

**Location:** `.obsidian/snippets/`

**Example Snippet:**
```css
/* custom-styling.css */

/* Apply to notes with cssclasses: [wide-page] */
.wide-page .markdown-preview-view {
  max-width: 1200px;
}

/* Style specific elements */
.cm-hashtag {
  color: var(--color-accent);
  font-weight: 600;
}

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-thumb {
  background: var(--interactive-accent);
  border-radius: 4px;
}
```

### 9.5 Style Settings Plugin

The Style Settings plugin allows themes to expose configurable settings:

**In theme.css:**
```css
/* @settings

name: My Theme
id: my-theme-settings
settings:
  -
    id: primary-color
    title: Primary Color
    type: variable-color
    format: hsl
    default: hsl(254, 80%, 68%)
  -
    id: font-size
    title: Font Size
    type: variable-number-slider
    default: 16
    min: 12
    max: 24
    step: 1
  -
    id: enable-feature
    title: Enable Custom Feature
    type: class-toggle
    default: false

*/

body.enable-feature .custom-element {
  /* Conditional styling */
}
```

---

## 10. Implementation Patterns for Rust Clone

### 10.1 Architecture Recommendations

**For a Rust-based clone:**

1. **Application Framework:**
   - **Tauri** instead of Electron (Rust + web frontend)
   - Benefits: Smaller binary, lower memory usage, Rust performance
   - Web view: WebKit (Linux/macOS) or WebView2 (Windows)

2. **File System Layer:**
   - Rust std::fs for file operations
   - notify crate for file watching
   - SQLite for metadata cache (fast queries)

3. **Markdown Parsing:**
   - pulldown-cmark or comrak (Rust markdown parsers)
   - Custom syntax extensions for wikilinks, highlights, etc.

4. **Editor:**
   - CodeMirror 6 (keep JavaScript - proven solution)
   - Or: Monaco Editor (VS Code editor component)
   - Or: Pure Rust solution (xi-editor, Lapce components)

5. **Plugin System:**
   - WASM plugins (write in Rust, compile to WASM)
   - JavaScript plugins (V8 runtime or QuickJS)
   - FFI-based native plugins

6. **Rendering:**
   - Web-based UI (HTML/CSS/JS)
   - Rust backend for data operations
   - JSON-RPC or similar for frontend-backend communication

### 10.2 Core Data Structures (Rust)

```rust
// File system
struct Vault {
    root_path: PathBuf,
    files: HashMap<PathBuf, VaultFile>,
    metadata_cache: MetadataCache,
    watcher: FileWatcher,
}

struct VaultFile {
    path: PathBuf,
    basename: String,
    extension: String,
    stat: FileStat,
}

// Metadata cache
struct MetadataCache {
    cache: HashMap<PathBuf, CachedMetadata>,
    resolved_links: HashMap<PathBuf, HashMap<PathBuf, usize>>,
    unresolved_links: HashMap<PathBuf, HashMap<String, usize>>,
}

struct CachedMetadata {
    frontmatter: Option<HashMap<String, serde_json::Value>>,
    links: Vec<LinkCache>,
    embeds: Vec<EmbedCache>,
    tags: Vec<TagCache>,
    headings: Vec<HeadingCache>,
}

struct LinkCache {
    link: String,
    original: String,
    display_text: Option<String>,
    position: Position,
}

// Workspace
struct Workspace {
    root_split: WorkspaceSplit,
    left_sidebar: Sidebar,
    right_sidebar: Sidebar,
    active_leaf: Option<LeafId>,
}

enum WorkspaceSplit {
    Leaf(WorkspaceLeaf),
    Split {
        direction: SplitDirection,
        children: Vec<WorkspaceSplit>,
    },
}

struct WorkspaceLeaf {
    id: LeafId,
    view: ViewType,
    view_state: serde_json::Value,
}
```

### 10.3 Plugin Architecture (Rust)

**Option 1: WASM Plugins**
```rust
// Plugin trait
pub trait Plugin {
    fn on_load(&mut self, app: &mut App) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
}

// WASM host
struct PluginManager {
    plugins: HashMap<String, PluginInstance>,
    runtime: WasmRuntime,
}

// Load WASM plugin
impl PluginManager {
    fn load_plugin(&mut self, path: &Path) -> Result<()> {
        let wasm_bytes = std::fs::read(path)?;
        let instance = self.runtime.instantiate(&wasm_bytes)?;

        // Call plugin's on_load
        instance.call("on_load", &[])?;

        self.plugins.insert(path.to_string_lossy().to_string(), instance);
        Ok(())
    }
}
```

**Option 2: JavaScript Plugins (V8)**
```rust
use deno_core::JsRuntime;

struct JavaScriptPluginManager {
    runtime: JsRuntime,
    plugins: HashMap<String, PluginHandle>,
}

impl JavaScriptPluginManager {
    fn execute_plugin(&mut self, code: &str) -> Result<()> {
        self.runtime.execute_script("<plugin>", code)?;
        Ok(())
    }
}
```

### 10.4 Performance Optimizations

**1. Metadata Cache:**
- SQLite database for persistent cache
- Index links, tags, headings for fast queries
- Incremental updates on file changes

**2. File Watching:**
- notify crate for efficient file system monitoring
- Debounce file change events (avoid rapid re-parsing)
- Background thread for file operations

**3. Graph View:**
- Pre-compute graph layout in background
- Use WebGL/Canvas for rendering (offload to GPU)
- Lazy loading for large graphs

**4. Search:**
- tantivy crate (full-text search engine in Rust)
- Incremental indexing
- Parallel search across files

### 10.5 Key Feature Priorities

**Phase 1 (MVP):**
1. File system operations (create, read, update, delete)
2. Markdown rendering with basic wikilinks
3. File explorer sidebar
4. Basic editor (CodeMirror integration)
5. Metadata cache (links, tags)

**Phase 2:**
1. Graph view
2. Backlinks panel
3. Search functionality
4. Daily notes
5. Templates

**Phase 3:**
1. Plugin system (WASM or JS)
2. Theme system
3. Canvas feature (JSON Canvas format)
4. Mobile apps (Tauri mobile support)

### 10.6 Testing Strategy

**Unit Tests:**
- Vault operations
- Metadata parsing
- Link resolution
- File watching

**Integration Tests:**
- End-to-end file operations
- Plugin loading and execution
- UI interactions (with headless browser)

**Performance Benchmarks:**
- Large vault handling (10k+ files)
- Search performance
- Graph rendering
- Metadata cache updates

---

## 11. API Signature Reference

### 11.1 Core Interfaces

```typescript
// App - Global service locator
interface App {
  vault: Vault;
  workspace: Workspace;
  metadataCache: MetadataCache;
  fileManager: FileManager;
  lastEvent: UserEvent | null;
}

// Plugin - Base class for all plugins
abstract class Plugin extends Component {
  app: App;
  manifest: PluginManifest;

  abstract onload(): void;
  abstract onunload(): void;

  addRibbonIcon(icon: string, title: string, callback: (evt: MouseEvent) => any): HTMLElement;
  addStatusBarItem(): HTMLElement;
  addCommand(command: Command): Command;
  addSettingTab(settingTab: PluginSettingTab): void;
  registerView(type: string, viewCreator: ViewCreator): void;
  registerExtensions(extensions: string[], viewType: string): void;
  registerMarkdownPostProcessor(postProcessor: MarkdownPostProcessor, sortOrder?: number): void;
  registerMarkdownCodeBlockProcessor(language: string, handler: MarkdownCodeBlockProcessor, sortOrder?: number): void;
  registerEditorExtension(extension: Extension): void;
  registerObsidianProtocolHandler(action: string, handler: ObsidianProtocolHandler): void;
  registerEditorSuggest(editorSuggest: EditorSuggest<any>): void;

  loadData(): Promise<any>;
  saveData(data: any): Promise<void>;
}

// Vault - File system operations
interface Vault {
  adapter: DataAdapter;
  configDir: string;

  getName(): string;
  getAbstractFileByPath(path: string): TAbstractFile | null;
  getRoot(): TFolder;
  create(path: string, data: string, options?: DataWriteOptions): Promise<TFile>;
  createBinary(path: string, data: ArrayBuffer, options?: DataWriteOptions): Promise<TFile>;
  createFolder(path: string): Promise<TFolder>;
  read(file: TFile): Promise<string>;
  cachedRead(file: TFile): Promise<string>;
  readBinary(file: TFile): Promise<ArrayBuffer>;
  getResourcePath(file: TFile): string;
  delete(file: TAbstractFile, force?: boolean): Promise<void>;
  trash(file: TAbstractFile, system: boolean): Promise<void>;
  rename(file: TAbstractFile, newPath: string): Promise<void>;
  modify(file: TFile, data: string, options?: DataWriteOptions): Promise<void>;
  modifyBinary(file: TFile, data: ArrayBuffer, options?: DataWriteOptions): Promise<void>;
  append(file: TFile, data: string, options?: DataWriteOptions): Promise<void>;
  process(file: TFile, fn: (data: string) => string, options?: DataWriteOptions): Promise<string>;
  copy(file: TFile, newPath: string): Promise<TFile>;
  getAllLoadedFiles(): TAbstractFile[];
  getMarkdownFiles(): TFile[];
  getFiles(): TFile[];

  on(name: 'create', callback: (file: TAbstractFile) => any, ctx?: any): EventRef;
  on(name: 'modify', callback: (file: TAbstractFile) => any, ctx?: any): EventRef;
  on(name: 'delete', callback: (file: TAbstractFile) => any, ctx?: any): EventRef;
  on(name: 'rename', callback: (file: TAbstractFile, oldPath: string) => any, ctx?: any): EventRef;
  on(name: 'closed', callback: () => any, ctx?: any): EventRef;
}

// Workspace - UI and pane management
interface Workspace {
  activeLeaf: WorkspaceLeaf | null;
  containerEl: HTMLElement;
  layoutReady: boolean;
  leftRibbon: WorkspaceRibbon;
  leftSplit: WorkspaceSidedock;
  rightSplit: WorkspaceSidedock;
  rootSplit: WorkspaceRoot;

  changeLayout(workspace: any): Promise<void>;
  getActiveFile(): TFile | null;
  getActiveViewOfType<T extends View>(type: Constructor<T>): T | null;
  getLeaf(newLeaf?: boolean): WorkspaceLeaf;
  getLeftLeaf(split: boolean): WorkspaceLeaf;
  getRightLeaf(split: boolean): WorkspaceLeaf;
  getUnpinnedLeaf(type?: string): WorkspaceLeaf;
  getMostRecentLeaf(): WorkspaceLeaf | null;
  getLeafById(id: string): WorkspaceLeaf | null;
  getGroupLeaves(group: string): WorkspaceLeaf[];
  getLeavesOfType(viewType: string): WorkspaceLeaf[];
  iterateRootLeaves(callback: (leaf: WorkspaceLeaf) => any): void;
  iterateAllLeaves(callback: (leaf: WorkspaceLeaf) => any): void;

  setActiveLeaf(leaf: WorkspaceLeaf, pushHistory?: boolean, focus?: boolean): void;
  splitActiveLeaf(direction?: SplitDirection): WorkspaceLeaf;
  duplicateLeaf(leaf: WorkspaceLeaf, direction?: SplitDirection): Promise<WorkspaceLeaf>;
  createLeafBySplit(leaf: WorkspaceLeaf, direction?: SplitDirection): WorkspaceLeaf;
  createLeafInParent(parent: WorkspaceSplit, index: number): WorkspaceLeaf;

  openLinkText(linktext: string, sourcePath: string, newLeaf?: boolean, openViewState?: OpenViewState): Promise<void>;

  on(name: 'layout-change', callback: () => any, ctx?: any): EventRef;
  on(name: 'resize', callback: () => any, ctx?: any): EventRef;
  on(name: 'active-leaf-change', callback: (leaf: WorkspaceLeaf) => any, ctx?: any): EventRef;
  on(name: 'file-open', callback: (file: TFile) => any, ctx?: any): EventRef;
  on(name: 'quit', callback: () => any, ctx?: any): EventRef;
}

// MetadataCache - Cached file metadata
interface MetadataCache {
  resolvedLinks: Record<string, Record<string, number>>;
  unresolvedLinks: Record<string, Record<string, number>>;

  getFileCache(file: TFile): CachedMetadata | null;
  getCache(path: string): CachedMetadata | null;
  getFirstLinkpathDest(linkpath: string, sourcePath: string): TFile | null;

  on(name: 'changed', callback: (file: TFile, data: string, cache: CachedMetadata) => any, ctx?: any): EventRef;
  on(name: 'resolve', callback: (file: TFile) => any, ctx?: any): EventRef;
  on(name: 'resolved', callback: () => any, ctx?: any): EventRef;
}

// Editor - CodeMirror wrapper
interface Editor {
  getDoc(): any;
  refresh(): void;
  getValue(): string;
  setValue(content: string): void;
  getLine(line: number): string;
  lineCount(): number;
  lastLine(): number;
  getSelection(): string;
  somethingSelected(): boolean;
  getRange(from: EditorPosition, to: EditorPosition): string;
  replaceSelection(replacement: string): void;
  replaceRange(replacement: string, from: EditorPosition, to: EditorPosition): void;
  getCursor(string?: 'from' | 'to' | 'head' | 'anchor'): EditorPosition;
  listSelections(): EditorSelection[];
  setCursor(pos: EditorPosition | number, ch?: number): void;
  setSelection(anchor: EditorPosition, head?: EditorPosition): void;
  setSelections(ranges: EditorSelection[]): void;
  focus(): void;
  blur(): void;
  hasFocus(): boolean;
  getScrollInfo(): { top: number; left: number };
  scrollTo(x?: number, y?: number): void;
  scrollIntoView(range: EditorRange): void;
  undo(): void;
  redo(): void;
  exec(command: string): void;
  transaction(tx: EditorTransaction): void;
}
```

### 11.2 Data Structures

```typescript
interface PluginManifest {
  id: string;
  name: string;
  author: string;
  version: string;
  minAppVersion: string;
  description: string;
  authorUrl?: string;
  isDesktopOnly?: boolean;
}

interface CachedMetadata {
  links?: LinkCache[];
  embeds?: EmbedCache[];
  tags?: TagCache[];
  headings?: HeadingCache[];
  sections?: SectionCache[];
  listItems?: ListItemCache[];
  frontmatter?: FrontMatterCache;
  frontmatterPosition?: Pos;
  blocks?: Record<string, BlockCache>;
}

interface LinkCache {
  link: string;
  original: string;
  displayText?: string;
  position: Pos;
}

interface EmbedCache extends LinkCache {}

interface TagCache {
  tag: string;
  position: Pos;
}

interface HeadingCache {
  heading: string;
  level: number;
  position: Pos;
}

interface Pos {
  start: Loc;
  end: Loc;
}

interface Loc {
  line: number;
  col: number;
  offset: number;
}

interface ViewState {
  type: string;
  state: any;
  active?: boolean;
  pinned?: boolean;
  group?: string;
}

interface OpenViewState {
  state?: any;
  eState?: any;
  active?: boolean;
}

interface Command {
  id: string;
  name: string;
  icon?: string;
  mobileOnly?: boolean;

  callback?: () => any;
  checkCallback?: (checking: boolean) => boolean | void;
  editorCallback?: (editor: Editor, view: MarkdownView) => any;
  editorCheckCallback?: (checking: boolean, editor: Editor, view: MarkdownView) => boolean | void;
  hotkeys?: Hotkey[];
}

interface Hotkey {
  modifiers: Modifier[];
  key: string;
}

type Modifier = 'Mod' | 'Ctrl' | 'Meta' | 'Shift' | 'Alt';
```

---

## 12. External Resources & Documentation

### 12.1 Official Documentation
- **Developer Docs:** https://docs.obsidian.md/
- **Obsidian Help:** https://help.obsidian.md/
- **API Repository:** https://github.com/obsidianmd/obsidian-api
- **Sample Plugin:** https://github.com/obsidianmd/obsidian-sample-plugin
- **JSON Canvas Spec:** https://jsoncanvas.org/

### 12.2 Community Resources
- **Plugin Developer Docs:** https://marcusolsson.github.io/obsidian-plugin-docs/
- **Obsidian Typings (Undocumented APIs):** https://github.com/Fevol/obsidian-typings
- **Plugin Stats:** https://www.obsidianstats.com/
- **Obsidian Forum:** https://forum.obsidian.md/

### 12.3 Related Technologies
- **Electron:** https://www.electronjs.org/
- **CodeMirror 6:** https://codemirror.net/
- **Tauri (Rust Alternative):** https://tauri.app/
- **Lucide Icons:** https://lucide.dev/

### 12.4 Recommended Reading
- Wikipedia: https://en.wikipedia.org/wiki/Obsidian_(software)
- SitePoint Beginner Guide: https://www.sitepoint.com/obsidian-beginner-guide/
- Plugin Development Tutorial: https://iamgodot.com/blog/developing-obsidian-plugins/

---

## 13. Conclusion & Recommendations

### 13.1 Key Takeaways

1. **Local-First Architecture:** Obsidian's success stems from its commitment to local storage, plain text files, and user data ownership.

2. **Extensibility:** The plugin system (2,692 plugins as of Dec 2025) is a cornerstone of the platform's value proposition.

3. **Web Technologies:** Despite being desktop-native, leveraging web technologies (HTML/CSS/JS) enables rapid development and rich UI.

4. **Markdown as Foundation:** Standard markdown with minimal custom extensions ensures future-proofing and interoperability.

5. **Performance at Scale:** Metadata caching and efficient file watching are critical for handling large vaults (10k+ notes).

### 13.2 Rust Implementation Advantages

**Using Rust for an Obsidian clone offers:**
- **Performance:** Native speed for file operations, search, and metadata parsing
- **Memory Safety:** Eliminate entire classes of bugs (data races, null pointers)
- **Smaller Footprint:** Tauri apps are 10-50x smaller than Electron equivalents
- **Battery Efficiency:** Lower CPU usage, especially important for laptops
- **Cross-Platform:** Single codebase for Windows, macOS, Linux

### 13.3 Critical Success Factors

1. **Data Migration:** Provide seamless import from Obsidian (vault structure is identical)
2. **Plugin Compatibility:** Support WASM plugins initially, JS plugins via runtime
3. **Performance:** Must match or exceed Obsidian's speed (users are sensitive to lag)
4. **UI Polish:** Match Obsidian's refined UX (keyboard shortcuts, intuitive design)
5. **Incremental Development:** Start with MVP, iterate based on user feedback

### 13.4 Recommended Tech Stack

```
┌─────────────────────────────────────┐
│         Frontend (Web)              │
│  - HTML/CSS/JavaScript              │
│  - CodeMirror 6 (editor)            │
│  - React/Vue/Svelte (optional)      │
└─────────────────────────────────────┘
              ↕ IPC
┌─────────────────────────────────────┐
│         Backend (Rust)              │
│  - Tauri (app framework)            │
│  - serde (serialization)            │
│  - pulldown-cmark (markdown)        │
│  - notify (file watching)           │
│  - tantivy (search)                 │
│  - rusqlite (metadata cache)        │
│  - wasmtime (plugin runtime)        │
└─────────────────────────────────────┘
              ↕
┌─────────────────────────────────────┐
│         File System                 │
│  - .md files (notes)                │
│  - .json files (config)             │
│  - .canvas files (canvas)           │
└─────────────────────────────────────┘
```

### 13.5 Next Steps for SPARC Specification

**Specification Phase:**
1. Define exact feature set (MVP vs. full parity)
2. Decide on plugin architecture (WASM, JS, or both)
3. Specify API contracts between frontend and backend
4. Design database schema for metadata cache
5. Plan migration strategy from Obsidian

**Pseudocode Phase:**
1. Vault operations (CRUD, watching)
2. Markdown parsing and rendering
3. Metadata extraction and caching
4. Link resolution algorithm
5. Graph generation algorithm
6. Search indexing and querying

**Architecture Phase:**
1. Component diagram (frontend, backend, storage)
2. Sequence diagrams (file open, search, graph view)
3. Data flow diagrams
4. Plugin system architecture
5. Testing strategy

**Refinement Phase:**
1. TDD for core Rust modules
2. Integration tests for Tauri IPC
3. UI component development
4. Performance benchmarking

**Completion Phase:**
1. End-to-end testing
2. Documentation
3. Packaging and distribution
4. Migration tooling

---

## Research Metadata

**Total Sources Consulted:** 50+
**Primary Documentation:** Obsidian Developer Docs, GitHub repositories
**Community Resources:** Forum threads, plugin examples, blog posts
**Technical Depth:** API-level with implementation patterns
**Completeness:** ✅ Core architecture, ✅ Plugin system, ✅ File formats, ✅ UI components, ✅ Implementation patterns

---

**End of Technical Specification Document**
