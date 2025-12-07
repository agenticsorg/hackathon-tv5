# SPARC Specification: Obsidian Clone in Rust

**Project:** ObsidianRS - A Backward-Compatible Obsidian Clone in Rust
**Version:** 1.0.0
**Date:** 2025-12-07
**Status:** Specification Phase Complete

---

## 1. Executive Summary

### 1.1 Project Vision

ObsidianRS is a high-performance, backward-compatible clone of Obsidian note-taking application built entirely in Rust. The project aims to provide:

- **100% Data Compatibility**: Seamless migration from existing Obsidian vaults
- **Plugin Compatibility**: Support for existing Obsidian plugins via JavaScript runtime
- **10x Performance**: Sub-millisecond search, instant startup, minimal memory usage
- **Cross-Platform**: Windows, macOS, Linux (desktop); iOS, Android (mobile); Web (PWA)

### 1.2 Key Metrics

| Metric | Obsidian (Electron) | ObsidianRS (Rust) | Improvement |
|--------|---------------------|-------------------|-------------|
| Startup Time | 2-5 seconds | <1 second | 5x faster |
| Memory (10K notes) | 500MB-1GB | ~330MB | 3x lower |
| Semantic Search | 50-200ms | <5ms | 10-40x faster |
| Bundle Size | ~150MB | ~15MB | 10x smaller |
| Battery Impact | High (Electron) | Low (Native) | Significant |

---

## 2. Functional Requirements

### 2.1 Core Note Management

#### FR-1: Vault Operations
- **FR-1.1**: Open existing Obsidian vaults without modification
- **FR-1.2**: Create new vaults with `.obsidian` configuration directory
- **FR-1.3**: Multiple vault support with vault switcher
- **FR-1.4**: Vault-level settings persistence

#### FR-2: Note Operations (CRUD)
- **FR-2.1**: Create notes with automatic link updating
- **FR-2.2**: Read notes with instant rendering
- **FR-2.3**: Update notes with automatic backup
- **FR-2.4**: Delete notes with optional trash support
- **FR-2.5**: Rename notes with automatic link resolution across vault
- **FR-2.6**: Move notes between folders with link preservation

#### FR-3: Markdown Support
- **FR-3.1**: CommonMark compliance
- **FR-3.2**: GitHub Flavored Markdown (tables, task lists, strikethrough)
- **FR-3.3**: Obsidian-specific syntax:
  - `[[wikilinks]]` - Internal note links
  - `[[Note|Display Text]]` - Aliased links
  - `[[Note#Heading]]` - Section links
  - `[[Note#^block-id]]` - Block references
  - `![[embedded]]` - Note/image embedding
  - `==highlighted text==` - Highlights
  - `%%hidden comments%%` - Comments
  - `#tags` and `#nested/tags` - Tag system
- **FR-3.4**: YAML frontmatter parsing and rendering
- **FR-3.5**: Code block syntax highlighting (50+ languages)
- **FR-3.6**: Math rendering (LaTeX via KaTeX/MathJax)

### 2.2 Search Functionality

#### FR-4: Full-Text Search
- **FR-4.1**: Instant search across all notes (<10ms)
- **FR-4.2**: Search operators: `tag:`, `file:`, `path:`, `content:`
- **FR-4.3**: Boolean operators: AND, OR, NOT
- **FR-4.4**: Regex search support
- **FR-4.5**: Search result highlighting

#### FR-5: Semantic Search (Enhanced)
- **FR-5.1**: Vector-based similarity search (<5ms)
- **FR-5.2**: "Notes like this" discovery
- **FR-5.3**: Natural language queries
- **FR-5.4**: Hybrid search (keyword + semantic)

### 2.3 Knowledge Graph

#### FR-6: Graph View
- **FR-6.1**: Force-directed graph visualization
- **FR-6.2**: Local graph (current note connections)
- **FR-6.3**: Global graph (entire vault)
- **FR-6.4**: Graph filtering (by tags, folders, depth)
- **FR-6.5**: Graph search and navigation
- **FR-6.6**: Orphan note detection

#### FR-7: Backlinks
- **FR-7.1**: Real-time backlink panel
- **FR-7.2**: Unlinked mentions detection
- **FR-7.3**: Outgoing links panel
- **FR-7.4**: Link preview on hover

### 2.4 User Interface

#### FR-8: Workspace
- **FR-8.1**: Split panes (horizontal/vertical)
- **FR-8.2**: Tab system with pinning
- **FR-8.3**: Left sidebar (file explorer, search, tags)
- **FR-8.4**: Right sidebar (backlinks, outline, tags)
- **FR-8.5**: Ribbon (quick actions)
- **FR-8.6**: Status bar
- **FR-8.7**: Command palette (Ctrl/Cmd+P)

#### FR-9: Editor
- **FR-9.1**: Source mode (raw markdown)
- **FR-9.2**: Live Preview mode (WYSIWYG hybrid)
- **FR-9.3**: Reading mode (rendered view)
- **FR-9.4**: Vim mode support
- **FR-9.5**: Multiple cursors
- **FR-9.6**: Find and replace (with regex)

#### FR-10: Themes
- **FR-10.1**: Light/Dark mode toggle
- **FR-10.2**: System theme following
- **FR-10.3**: Custom CSS snippets
- **FR-10.4**: Community theme support (400+ CSS variables)

### 2.5 Plugin System

#### FR-11: Plugin Architecture
- **FR-11.1**: JavaScript plugin runtime (deno_core)
- **FR-11.2**: Obsidian API compatibility layer
- **FR-11.3**: Plugin manifest support (manifest.json)
- **FR-11.4**: Plugin settings persistence
- **FR-11.5**: Hot reload support

#### FR-12: Plugin Features
- **FR-12.1**: Command registration
- **FR-12.2**: Ribbon icons
- **FR-12.3**: Settings tabs
- **FR-12.4**: Custom views
- **FR-12.5**: Editor extensions
- **FR-12.6**: Event hooks (file create/modify/delete/rename)
- **FR-12.7**: Modal dialogs

### 2.6 Additional Features

#### FR-13: Daily Notes
- **FR-13.1**: Configurable date format
- **FR-13.2**: Custom template support
- **FR-13.3**: Auto-open on startup option

#### FR-14: Templates
- **FR-14.1**: Template folder configuration
- **FR-14.2**: Date/time variables
- **FR-14.3**: Cursor positioning
- **FR-14.4**: Insert from template command

#### FR-15: Canvas (JSON Canvas)
- **FR-15.1**: Node types: text, file, link, group
- **FR-15.2**: Edge connections with labels
- **FR-15.3**: Infinite canvas panning/zooming
- **FR-15.4**: JSON Canvas format compatibility

---

## 3. Non-Functional Requirements

### 3.1 Performance Requirements

#### NFR-1: Startup Performance
- **NFR-1.1**: Cold start < 1 second
- **NFR-1.2**: Hot start < 500ms
- **NFR-1.3**: Vault indexing < 5 seconds (10K notes)

#### NFR-2: Runtime Performance
- **NFR-2.1**: Note open < 100ms
- **NFR-2.2**: Search latency < 10ms (keyword), < 5ms (semantic)
- **NFR-2.3**: Graph render < 500ms (10K nodes)
- **NFR-2.4**: File watch → reindex < 300ms
- **NFR-2.5**: 60 FPS UI interactions

#### NFR-3: Resource Usage
- **NFR-3.1**: Memory < 400MB (10K notes)
- **NFR-3.2**: CPU idle < 1%
- **NFR-3.3**: Disk index < 500MB (10K notes)

### 3.2 Scalability Requirements

#### NFR-4: Vault Size
- **NFR-4.1**: Support 100K+ notes
- **NFR-4.2**: Support 1M+ links
- **NFR-4.3**: Support 10GB+ attachments
- **NFR-4.4**: Linear performance degradation

### 3.3 Reliability Requirements

#### NFR-5: Data Integrity
- **NFR-5.1**: ACID operations on metadata
- **NFR-5.2**: Automatic backup (hourly snapshots)
- **NFR-5.3**: Crash recovery with journal
- **NFR-5.4**: Zero data loss guarantee

### 3.4 Security Requirements

#### NFR-6: Security
- **NFR-6.1**: Plugin sandboxing (V8 isolates)
- **NFR-6.2**: No automatic network access
- **NFR-6.3**: Local-first data storage
- **NFR-6.4**: Optional vault encryption

### 3.5 Compatibility Requirements

#### NFR-7: Platform Support
- **NFR-7.1**: Windows 10/11 (x64, ARM64)
- **NFR-7.2**: macOS 11+ (Intel, Apple Silicon)
- **NFR-7.3**: Linux (glibc 2.31+, x64, ARM64)
- **NFR-7.4**: Web browsers (Chrome, Firefox, Safari, Edge)
- **NFR-7.5**: Mobile (iOS 14+, Android 10+) - Future

#### NFR-8: Data Compatibility
- **NFR-8.1**: 100% Obsidian vault format compatibility
- **NFR-8.2**: Import/export support
- **NFR-8.3**: JSON Canvas format compliance

---

## 4. Technology Stack

### 4.1 Core Framework

| Component | Technology | Purpose |
|-----------|------------|---------|
| Desktop Framework | Tauri v2.0 | Native desktop app with web frontend |
| Language | Rust 2021 Edition | Core backend logic |
| Frontend | TypeScript + React | User interface |
| Editor | CodeMirror 6 | Text editing |
| Plugin Runtime | deno_core | JavaScript plugin execution |

### 4.2 Storage & Search

| Component | Crate | Purpose |
|-----------|-------|---------|
| Vector Database | ruvector-core | Semantic search |
| Graph Database | ruvector-graph + petgraph | Knowledge graph |
| Full-Text Search | tantivy | Keyword search |
| Metadata Store | redb | Note metadata |
| Configuration | serde_yaml/serde_json | Config files |

### 4.3 Processing

| Component | Crate | Purpose |
|-----------|-------|---------|
| Markdown Parser | pulldown-cmark | CommonMark parsing |
| Custom Syntax | regex | Wikilinks, tags, embeds |
| Text Editing | ropey | Rope data structure |
| Syntax Highlighting | tree-sitter | Incremental highlighting |
| File Watching | notify | Real-time file events |
| Directory Walking | walkdir | Vault scanning |

### 4.4 Recommended Crate Versions

```toml
[dependencies]
# Core Framework
tauri = { version = "2.0", features = ["shell-open"] }

# Vector & Search
ruvector-core = "0.1.21"
ruvector-graph = "0.1.21"
ruvector-gnn = "0.1.15"
tantivy = "0.21"

# Graph
petgraph = "0.6"

# Markdown
pulldown-cmark = "0.9"

# File System
notify = "6.1"
walkdir = "2.4"

# Text Editing
ropey = "1.6"
tree-sitter = "0.20"

# Storage
redb = "1.5"

# Plugin System
deno_core = "0.240"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# Async Runtime
tokio = { version = "1", features = ["full"] }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
regex = "1"
rayon = "1.8"
thiserror = "1"
anyhow = "1"
tracing = "0.1"
```

---

## 5. Data Models

### 5.1 Core Data Structures

```rust
/// Represents a note file in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier (file path relative to vault)
    pub id: String,
    /// File path
    pub path: PathBuf,
    /// Filename without extension
    pub basename: String,
    /// File extension
    pub extension: String,
    /// Raw markdown content
    pub content: String,
    /// Parsed frontmatter
    pub frontmatter: Option<Frontmatter>,
    /// File statistics
    pub stat: FileStat,
}

/// YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub aliases: Option<Vec<String>>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub cssclasses: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Cached metadata for fast queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMetadata {
    pub links: Vec<LinkCache>,
    pub embeds: Vec<EmbedCache>,
    pub tags: Vec<TagCache>,
    pub headings: Vec<HeadingCache>,
    pub blocks: HashMap<String, BlockCache>,
    pub frontmatter: Option<Frontmatter>,
}

/// Internal link cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCache {
    pub link: String,           // Target note
    pub original: String,       // Original markdown
    pub display_text: Option<String>,
    pub heading: Option<String>,
    pub block_id: Option<String>,
    pub position: Position,
}

/// Tag cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCache {
    pub tag: String,
    pub position: Position,
}

/// Heading cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingCache {
    pub heading: String,
    pub level: u8,  // 1-6
    pub position: Position,
}

/// Text position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub start: Location,
    pub end: Location,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}
```

### 5.2 Graph Data Model

```rust
/// Knowledge graph for backlinks and visualization
pub struct KnowledgeGraph {
    /// In-memory graph for fast queries
    pub memory_graph: DiGraph<NoteNode, LinkEdge>,
    /// Node index mapping
    pub node_map: HashMap<String, NodeIndex>,
    /// Resolved links: source → (target → count)
    pub resolved_links: HashMap<String, HashMap<String, usize>>,
    /// Unresolved links: source → (target → count)
    pub unresolved_links: HashMap<String, HashMap<String, usize>>,
}

#[derive(Debug, Clone)]
pub struct NoteNode {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum LinkEdge {
    Reference,
    Embed,
    Tag,
}
```

### 5.3 Workspace Data Model

```rust
/// Workspace layout state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub main: WorkspaceSplit,
    pub left: Option<WorkspaceSplit>,
    pub right: Option<WorkspaceSplit>,
    pub active_leaf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkspaceSplit {
    Leaf(WorkspaceLeaf),
    Split {
        direction: SplitDirection,
        children: Vec<WorkspaceSplit>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLeaf {
    pub id: String,
    pub view_type: String,
    pub state: serde_json::Value,
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}
```

---

## 6. File Format Specifications

### 6.1 Vault Structure

```
my-vault/
├── .obsidian/                    # Configuration directory
│   ├── app.json                  # Application settings
│   ├── appearance.json           # Theme settings
│   ├── workspace.json            # Layout state
│   ├── graph.json                # Graph view settings
│   ├── plugins/                  # Plugin data
│   │   └── plugin-id/
│   │       ├── data.json
│   │       └── main.js
│   ├── themes/                   # Custom themes
│   │   └── theme-name/
│   │       ├── manifest.json
│   │       └── theme.css
│   └── snippets/                 # CSS snippets
│       └── custom.css
├── notes/                        # User notes (any structure)
├── attachments/                  # Media files
└── templates/                    # Note templates
```

### 6.2 JSON Canvas Format

```json
{
  "nodes": [
    {
      "id": "unique-id",
      "type": "text|file|link|group",
      "x": 100,
      "y": 200,
      "width": 300,
      "height": 150,
      "text": "# Markdown content",  // for text nodes
      "file": "path/to/note.md",     // for file nodes
      "url": "https://..."           // for link nodes
    }
  ],
  "edges": [
    {
      "id": "edge-id",
      "fromNode": "node-1",
      "toNode": "node-2",
      "fromSide": "right",
      "toSide": "left",
      "color": "3",
      "label": "Connection"
    }
  ]
}
```

---

## 7. API Specifications

### 7.1 Vault API

```rust
/// Vault interface for file operations
pub trait Vault {
    // File Operations
    async fn read(&self, file: &TFile) -> Result<String>;
    async fn modify(&self, file: &TFile, data: &str) -> Result<()>;
    async fn create(&self, path: &str, data: &str) -> Result<TFile>;
    async fn delete(&self, file: &TAbstractFile) -> Result<()>;
    async fn rename(&self, file: &TAbstractFile, new_path: &str) -> Result<()>;

    // Folder Operations
    async fn create_folder(&self, path: &str) -> Result<TFolder>;

    // Query
    fn get_abstract_file_by_path(&self, path: &str) -> Option<TAbstractFile>;
    fn get_markdown_files(&self) -> Vec<TFile>;
    fn get_files(&self) -> Vec<TFile>;

    // Events
    fn on_create(&self, callback: impl Fn(&TAbstractFile) + Send + Sync);
    fn on_modify(&self, callback: impl Fn(&TAbstractFile) + Send + Sync);
    fn on_delete(&self, callback: impl Fn(&TAbstractFile) + Send + Sync);
    fn on_rename(&self, callback: impl Fn(&TAbstractFile, &str) + Send + Sync);
}
```

### 7.2 Search API

```rust
/// Hybrid search interface
pub trait SearchEngine {
    /// Full-text keyword search
    async fn keyword_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    /// Semantic similarity search
    async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    /// Combined hybrid search
    async fn hybrid_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    /// Search with filters
    async fn filtered_search(
        &self,
        query: &str,
        filters: SearchFilters,
        limit: usize
    ) -> Result<Vec<SearchResult>>;
}

pub struct SearchFilters {
    pub tags: Option<Vec<String>>,
    pub path: Option<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub file_type: Option<String>,
}

pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub matches: Vec<TextMatch>,
}
```

### 7.3 Plugin API

```typescript
// Obsidian-compatible Plugin API (JavaScript)
interface Plugin {
  app: App;
  manifest: PluginManifest;

  onload(): void;
  onunload(): void;

  addCommand(command: Command): Command;
  addRibbonIcon(icon: string, title: string, callback: () => void): HTMLElement;
  addSettingTab(tab: PluginSettingTab): void;
  registerView(type: string, viewCreator: ViewCreator): void;
  registerMarkdownPostProcessor(processor: MarkdownPostProcessor): void;
  registerEditorExtension(extension: Extension): void;

  loadData(): Promise<any>;
  saveData(data: any): Promise<void>;
}

interface App {
  vault: Vault;
  workspace: Workspace;
  metadataCache: MetadataCache;
}
```

---

## 8. UI/UX Specifications

### 8.1 Layout

```
┌────────────────────────────────────────────────────────────┐
│ ▪ ObsidianRS                               - □ X │
├───┬────────────────────────────────────────────────────┬───┤
│ R │ Tab Bar                                            │   │
│ i ├────────────────────────────────────────────────────┤ R │
│ b │                                                    │ i │
│ b │               Main Editor Area                     │ g │
│ o │                                                    │ h │
│ n │           (Split panes supported)                  │ t │
│   │                                                    │   │
│ ┌─┤                                                    ├─┐ │
│ │F│                                                    │B│ │
│ │i│                                                    │a│ │
│ │l│                                                    │c│ │
│ │e│                                                    │k│ │
│ │s│                                                    │l│ │
│ └─┤                                                    ├─┘ │
├───┴────────────────────────────────────────────────────┴───┤
│ Status: Ready                    Ln 42, Col 15      UTF-8 │
└────────────────────────────────────────────────────────────┘
```

### 8.2 Color System (CSS Variables)

```css
/* Light Theme */
.theme-light {
  --background-primary: #ffffff;
  --background-secondary: #f5f6f8;
  --text-normal: #2e3338;
  --text-muted: #888888;
  --accent-h: 254;
  --accent-s: 80%;
  --accent-l: 68%;
}

/* Dark Theme */
.theme-dark {
  --background-primary: #1e1e1e;
  --background-secondary: #262626;
  --text-normal: #dcddde;
  --text-muted: #888888;
  --accent-h: 254;
  --accent-s: 80%;
  --accent-l: 68%;
}
```

### 8.3 Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| Command Palette | Ctrl+P | Cmd+P |
| Quick Switcher | Ctrl+O | Cmd+O |
| Search | Ctrl+Shift+F | Cmd+Shift+F |
| New Note | Ctrl+N | Cmd+N |
| Toggle Edit/Preview | Ctrl+E | Cmd+E |
| Graph View | Ctrl+G | Cmd+G |
| Bold | Ctrl+B | Cmd+B |
| Italic | Ctrl+I | Cmd+I |
| Link | Ctrl+K | Cmd+K |

---

## 9. Success Criteria

### 9.1 MVP Requirements (Phase 1)

- [ ] Open/create Obsidian vaults
- [ ] CRUD operations on notes
- [ ] Markdown rendering with Obsidian syntax
- [ ] Full-text search (<10ms)
- [ ] File explorer sidebar
- [ ] Basic backlinks
- [ ] Light/Dark themes

### 9.2 Feature Parity (Phase 2)

- [ ] Graph view visualization
- [ ] Live Preview editor mode
- [ ] Daily notes
- [ ] Templates
- [ ] Semantic search
- [ ] Command palette
- [ ] Split panes

### 9.3 Plugin System (Phase 3)

- [ ] JavaScript plugin runtime
- [ ] Obsidian API compatibility
- [ ] Plugin settings
- [ ] Custom views
- [ ] Editor extensions

### 9.4 Advanced (Phase 4)

- [ ] Canvas feature
- [ ] Sync service
- [ ] Mobile apps
- [ ] Web deployment

---

## 10. Risk Assessment

### 10.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Plugin compatibility | High | Medium | Comprehensive API testing |
| Performance at scale | Medium | Low | Early benchmarking |
| UI/UX complexity | Medium | Medium | Phased development |
| Memory leaks | High | Low | Rust ownership model |

### 10.2 Project Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Scope creep | High | High | Strict MVP focus |
| Community adoption | Medium | Medium | Early user testing |
| Obsidian updates | Low | High | API abstraction layer |

---

## 11. Timeline

### Phase 1: MVP (Weeks 1-6)
- Core file operations
- Markdown rendering
- Basic UI
- Search functionality

### Phase 2: Feature Parity (Weeks 7-12)
- Graph view
- Advanced editor
- Daily notes/templates
- Workspace layouts

### Phase 3: Plugin System (Weeks 13-18)
- JavaScript runtime
- API compatibility
- Plugin marketplace

### Phase 4: Polish (Weeks 19-24)
- Performance optimization
- Mobile support
- Sync service

---

## 12. References

### Official Sources
- [Obsidian Developer Documentation](https://docs.obsidian.md/)
- [Obsidian API Repository](https://github.com/obsidianmd/obsidian-api)
- [JSON Canvas Specification](https://jsoncanvas.org/)

### Rust Ecosystem
- [RuVector Documentation](https://github.com/ruvnet/ruvector)
- [Tauri Framework](https://tauri.app/)
- [Tantivy Search Engine](https://github.com/quickwit-oss/tantivy)

---

**Document Status:** SPECIFICATION COMPLETE
**Next Phase:** ARCHITECTURE
**Author:** SPARC Specification Agent
**Date:** 2025-12-07
