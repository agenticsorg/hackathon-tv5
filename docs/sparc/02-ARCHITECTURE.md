# SPARC Architecture: Obsidian Clone in Rust

**Project:** ObsidianRS - A Backward-Compatible Obsidian Clone in Rust
**Version:** 1.0.0
**Date:** 2025-12-07
**Status:** Architecture Phase Complete

---

## 1. System Architecture Overview

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           ObsidianRS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Presentation Layer                        │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │   │
│  │  │   React UI   │  │ CodeMirror  │  │   Graph Canvas      │  │   │
│  │  │  Components  │  │   Editor    │  │   (WebGL/Canvas)    │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ↕ Tauri IPC                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Application Layer (Rust)                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │   │
│  │  │   Vault     │  │  Workspace  │  │     Plugin          │  │   │
│  │  │   Manager   │  │   Manager   │  │     Runtime         │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ↕                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Domain Layer (Rust)                       │   │
│  │  ┌───────────┐ ┌────────────┐ ┌──────────┐ ┌─────────────┐  │   │
│  │  │   Note    │ │  Markdown  │ │  Search  │ │   Graph     │  │   │
│  │  │  Service  │ │   Engine   │ │  Engine  │ │  Service    │  │   │
│  │  └───────────┘ └────────────┘ └──────────┘ └─────────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ↕                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  Infrastructure Layer (Rust)                 │   │
│  │  ┌───────────┐ ┌────────────┐ ┌──────────┐ ┌─────────────┐  │   │
│  │  │   File    │ │  Vector    │ │ FullText │ │  Metadata   │  │   │
│  │  │  System   │ │  Database  │ │  Index   │ │   Store     │  │   │
│  │  │ (notify)  │ │ (ruvector) │ │(tantivy) │ │   (redb)    │  │   │
│  │  └───────────┘ └────────────┘ └──────────┘ └─────────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ↕                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      Storage Layer                           │   │
│  │  ┌─────────────────────────────────────────────────────────┐│   │
│  │  │     Local File System (Markdown files, attachments)     ││   │
│  │  └─────────────────────────────────────────────────────────┘│   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 Component Diagram

```
┌────────────────────────────────────────────────────────────────────────┐
│                         PRESENTATION LAYER                             │
│                                                                        │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────┐   │
│  │  App Shell   │───│  Sidebar     │───│  Main Content Area       │   │
│  │              │   │  - Files     │   │  - Editor Panes          │   │
│  │  - Titlebar  │   │  - Search    │   │  - Graph View            │   │
│  │  - Ribbon    │   │  - Tags      │   │  - Canvas                │   │
│  │  - Status    │   │  - Backlinks │   │  - Preview               │   │
│  └──────────────┘   └──────────────┘   └──────────────────────────┘   │
│                                                                        │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────┐   │
│  │  Modal       │   │  Command     │   │  Settings                │   │
│  │  System      │   │  Palette     │   │  Panel                   │   │
│  └──────────────┘   └──────────────┘   └──────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
                               │
                               │ Tauri Commands (IPC)
                               ▼
┌────────────────────────────────────────────────────────────────────────┐
│                         APPLICATION LAYER                              │
│                                                                        │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                       App State Manager                          │  │
│  │  - Vault Context      - Workspace State      - Settings State   │  │
│  │  - Plugin Registry    - Command Registry     - Event Bus        │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────┐   │
│  │  Vault       │   │  Workspace   │   │  Plugin                  │   │
│  │  Manager     │   │  Manager     │   │  Runtime                 │   │
│  │              │   │              │   │                          │   │
│  │  - Open      │   │  - Splits    │   │  - deno_core             │   │
│  │  - Create    │   │  - Tabs      │   │  - API Bindings          │   │
│  │  - Switch    │   │  - Focus     │   │  - Sandboxing            │   │
│  └──────────────┘   └──────────────┘   └──────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌────────────────────────────────────────────────────────────────────────┐
│                           DOMAIN LAYER                                 │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                         Note Service                              │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐ │ │
│  │  │   Create   │  │    Read    │  │   Update   │  │   Delete   │ │ │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘ │ │
│  │  ┌────────────┐  ┌────────────┐  ┌─────────────────────────────┐│ │
│  │  │   Rename   │  │    Move    │  │    Link Resolution         ││ │
│  │  └────────────┘  └────────────┘  └─────────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                       Markdown Engine                             │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐ │ │
│  │  │  Parser    │  │  Renderer  │  │  Wikilink  │  │   Embed    │ │ │
│  │  │ (pulldown) │  │  (HTML)    │  │  Parser    │  │  Resolver  │ │ │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘ │ │
│  │  ┌────────────┐  ┌────────────┐  ┌─────────────────────────────┐│ │
│  │  │ Frontmatter│  │    Tags    │  │   Syntax Highlighting      ││ │
│  │  │   Parser   │  │   Parser   │  │      (tree-sitter)         ││ │
│  │  └────────────┘  └────────────┘  └─────────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                        Search Engine                              │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐ │ │
│  │  │    Keyword     │  │    Semantic    │  │      Hybrid        │ │ │
│  │  │   (tantivy)    │  │   (ruvector)   │  │     Merger         │ │ │
│  │  └────────────────┘  └────────────────┘  └────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                        Graph Service                              │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐ │ │
│  │  │   In-Memory    │  │   Persistent   │  │   Visualization    │ │ │
│  │  │   (petgraph)   │  │  (ruvector-g)  │  │      Data          │ │ │
│  │  └────────────────┘  └────────────────┘  └────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌────────────────────────────────────────────────────────────────────────┐
│                       INFRASTRUCTURE LAYER                             │
│                                                                        │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────────────┐  │
│  │  File System  │  │    Vector     │  │      Full-Text            │  │
│  │   Adapter     │  │   Database    │  │       Index               │  │
│  │               │  │               │  │                           │  │
│  │  - notify     │  │  - ruvector   │  │  - tantivy                │  │
│  │  - walkdir    │  │  - HNSW       │  │  - BM25                   │  │
│  │  - std::fs    │  │  - 384d vecs  │  │  - FTS5                   │  │
│  └───────────────┘  └───────────────┘  └───────────────────────────┘  │
│                                                                        │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────────────┐  │
│  │   Metadata    │  │    Graph      │  │      Config               │  │
│  │    Store      │  │   Storage     │  │      Manager              │  │
│  │               │  │               │  │                           │  │
│  │  - redb       │  │  - ruvector-g │  │  - serde_yaml             │  │
│  │  - KV store   │  │  - Cypher     │  │  - serde_json             │  │
│  └───────────────┘  └───────────────┘  └───────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Module Architecture

### 2.1 Crate Organization

```
obsidian-rs/
├── Cargo.toml                    # Workspace definition
├── crates/
│   ├── obsidian-core/            # Core types and traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── note.rs           # Note data structures
│   │       ├── vault.rs          # Vault traits
│   │       ├── workspace.rs      # Workspace types
│   │       ├── metadata.rs       # Cached metadata
│   │       └── error.rs          # Error types
│   │
│   ├── obsidian-markdown/        # Markdown processing
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs         # pulldown-cmark wrapper
│   │       ├── wikilink.rs       # [[wikilink]] parser
│   │       ├── embed.rs          # ![[embed]] parser
│   │       ├── tag.rs            # #tag parser
│   │       ├── frontmatter.rs    # YAML frontmatter
│   │       └── renderer.rs       # HTML renderer
│   │
│   ├── obsidian-search/          # Search functionality
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── fulltext.rs       # tantivy integration
│   │       ├── semantic.rs       # ruvector integration
│   │       ├── hybrid.rs         # Combined search
│   │       └── filters.rs        # Search filters
│   │
│   ├── obsidian-graph/           # Knowledge graph
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── memory_graph.rs   # petgraph wrapper
│   │       ├── persistent.rs     # ruvector-graph
│   │       ├── backlinks.rs      # Backlink resolution
│   │       └── visualization.rs  # Graph view data
│   │
│   ├── obsidian-storage/         # Storage layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── filesystem.rs     # File operations
│   │       ├── watcher.rs        # File watching (notify)
│   │       ├── metadata_db.rs    # redb integration
│   │       └── config.rs         # Configuration files
│   │
│   ├── obsidian-editor/          # Editor support
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rope.rs           # ropey integration
│   │       ├── highlighting.rs   # tree-sitter
│   │       └── operations.rs     # Edit operations
│   │
│   └── obsidian-plugins/         # Plugin system
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── runtime.rs        # deno_core runtime
│           ├── api.rs            # Obsidian API bindings
│           ├── sandbox.rs        # Security sandboxing
│           └── manifest.rs       # Plugin manifest
│
├── src-tauri/                    # Tauri application
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── commands/             # Tauri commands
│       │   ├── mod.rs
│       │   ├── vault.rs
│       │   ├── note.rs
│       │   ├── search.rs
│       │   └── workspace.rs
│       └── state.rs              # Application state
│
└── src/                          # Frontend (React/TypeScript)
    ├── App.tsx
    ├── components/
    │   ├── Editor/
    │   ├── Sidebar/
    │   ├── GraphView/
    │   └── Modal/
    ├── hooks/
    ├── api/
    │   └── tauri.ts              # Tauri bindings
    └── styles/
```

### 2.2 Dependency Graph

```
                                ┌─────────────────┐
                                │  obsidian-core  │
                                │  (types/traits) │
                                └────────┬────────┘
                                         │
              ┌──────────────────────────┼──────────────────────────┐
              │                          │                          │
              ▼                          ▼                          ▼
    ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
    │obsidian-markdown│       │ obsidian-search │       │  obsidian-graph │
    │   (parsing)     │       │   (indexing)    │       │  (knowledge)    │
    └────────┬────────┘       └────────┬────────┘       └────────┬────────┘
             │                         │                          │
             └──────────────┬──────────┴──────────────────────────┘
                            │
                            ▼
                  ┌─────────────────┐
                  │obsidian-storage │
                  │ (persistence)   │
                  └────────┬────────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
    ┌─────────────────┐       ┌─────────────────┐
    │ obsidian-editor │       │obsidian-plugins │
    │  (text ops)     │       │   (runtime)     │
    └─────────────────┘       └─────────────────┘

              ┌───────────────────────────────────┐
              │                                   │
              ▼                                   │
    ┌─────────────────────────────────────┐      │
    │            Tauri App                │◄─────┘
    │  (integrates all crates)            │
    └─────────────────────────────────────┘
```

---

## 3. Data Flow Architecture

### 3.1 Note Creation Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  User    │     │   UI     │     │  Tauri   │     │  Vault   │
│          │     │ (React)  │     │ Command  │     │ Manager  │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ Create Note    │                │                │
     │───────────────>│                │                │
     │                │                │                │
     │                │ invoke("create │                │
     │                │ _note", path)  │                │
     │                │───────────────>│                │
     │                │                │                │
     │                │                │ create_note()  │
     │                │                │───────────────>│
     │                │                │                │
     │                │                │                │ ┌──────────────┐
     │                │                │                │─│ File System  │
     │                │                │                │ │ Write .md    │
     │                │                │                │ └──────────────┘
     │                │                │                │
     │                │                │                │ ┌──────────────┐
     │                │                │                │─│ Parse MD     │
     │                │                │                │ │ Extract meta │
     │                │                │                │ └──────────────┘
     │                │                │                │
     │                │                │                │ ┌──────────────┐
     │                │                │                │─│ Index Search │
     │                │                │                │ │ Vector+FTS   │
     │                │                │                │ └──────────────┘
     │                │                │                │
     │                │                │                │ ┌──────────────┐
     │                │                │                │─│ Update Graph │
     │                │                │                │ │ petgraph     │
     │                │                │                │ └──────────────┘
     │                │                │                │
     │                │                │◄───────────────│ Note created
     │                │◄───────────────│ Result         │
     │◄───────────────│ Update UI      │                │
     │                │                │                │
```

### 3.2 Search Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  User    │     │   UI     │     │  Search  │     │ Indexes  │
│          │     │ (React)  │     │  Engine  │     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ Type query     │                │                │
     │───────────────>│                │                │
     │                │                │                │
     │                │ hybrid_search  │                │
     │                │ (query)        │                │
     │                │───────────────>│                │
     │                │                │                │
     │                │                │  ┌─────────────────────────────┐
     │                │                │  │     Parallel Execution      │
     │                │                │  │                             │
     │                │                │  │  ┌───────────┐ ┌─────────┐ │
     │                │                │──│─>│  tantivy  │ │ruvector │ │
     │                │                │  │  │  keyword  │ │semantic │ │
     │                │                │  │  │  <10ms    │ │  <5ms   │ │
     │                │                │  │  └─────┬─────┘ └────┬────┘ │
     │                │                │  │        │            │      │
     │                │                │  │        └──────┬─────┘      │
     │                │                │  │               │            │
     │                │                │  │        Merge & Rank        │
     │                │                │  │        (score fusion)      │
     │                │                │  │               │            │
     │                │                │  └───────────────│────────────┘
     │                │                │                  │
     │                │◄───────────────│◄─────────────────┘
     │                │  Results       │  Ranked results
     │◄───────────────│                │
     │ Display        │                │
     │                │                │
```

### 3.3 File Watch & Reindex Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│   File   │     │  notify  │     │  Vault   │     │ Indexers │
│  System  │     │ Watcher  │     │ Manager  │     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ File modified  │                │                │
     │───────────────>│                │                │
     │                │                │                │
     │                │ Debounce       │                │
     │                │ (500ms)        │                │
     │                │───────┐        │                │
     │                │       │        │                │
     │                │◄──────┘        │                │
     │                │                │                │
     │                │ FileEvent      │                │
     │                │───────────────>│                │
     │                │                │                │
     │                │                │ Read file     │
     │                │                │───────────────>│ (file system)
     │                │                │◄───────────────│
     │                │                │                │
     │                │                │ Parse markdown │
     │                │                │───────────────>│ (pulldown)
     │                │                │◄───────────────│
     │                │                │                │
     │                │                │ Update indexes │
     │                │                │───────────────>│
     │                │                │                │
     │                │                │                │ ┌─────────────┐
     │                │                │                │─│ ruvector    │
     │                │                │                │ │ re-embed    │
     │                │                │                │ └─────────────┘
     │                │                │                │
     │                │                │                │ ┌─────────────┐
     │                │                │                │─│ tantivy     │
     │                │                │                │ │ update doc  │
     │                │                │                │ └─────────────┘
     │                │                │                │
     │                │                │                │ ┌─────────────┐
     │                │                │                │─│ petgraph    │
     │                │                │                │ │ update links│
     │                │                │                │ └─────────────┘
     │                │                │                │
     │                │                │◄───────────────│ Done
     │                │                │                │
     │                │                │ Emit event     │
     │                │                │ to UI          │
```

---

## 4. Storage Architecture

### 4.1 Storage Layer Design

```
┌────────────────────────────────────────────────────────────────┐
│                      Storage Manager                           │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                    Primary Storage                        │ │
│  │                                                           │ │
│  │  ┌─────────────────────────────────────────────────────┐ │ │
│  │  │              File System (Markdown)                  │ │ │
│  │  │                                                      │ │ │
│  │  │  vault/                                              │ │ │
│  │  │  ├── notes/         # User markdown files            │ │ │
│  │  │  ├── attachments/   # Images, PDFs, etc.             │ │ │
│  │  │  └── .obsidian/     # Configuration                  │ │ │
│  │  │                                                      │ │ │
│  │  └─────────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                   Index Storage                           │ │
│  │                                                           │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │ │
│  │  │  Vector DB  │  │  Full-Text  │  │   Metadata DB   │   │ │
│  │  │             │  │   Index     │  │                 │   │ │
│  │  │  .obsidian/ │  │  .obsidian/ │  │   .obsidian/    │   │ │
│  │  │  /vectors/  │  │  /search/   │  │   /metadata.db  │   │ │
│  │  │             │  │             │  │                 │   │ │
│  │  │  ruvector   │  │  tantivy    │  │   redb          │   │ │
│  │  │  ~150MB     │  │  ~50MB      │  │   ~20MB         │   │ │
│  │  │  per 10K    │  │  per 10K    │  │   per 10K       │   │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘   │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                   Graph Storage                           │ │
│  │                                                           │ │
│  │  ┌─────────────────────┐  ┌────────────────────────────┐ │ │
│  │  │   In-Memory Graph   │  │   Persistent Hypergraph    │ │ │
│  │  │                     │  │                            │ │ │
│  │  │   petgraph::DiGraph │  │   ruvector-graph           │ │ │
│  │  │   ~30MB per 10K     │  │   .obsidian/graph/         │ │ │
│  │  │                     │  │   ~40MB per 10K            │ │ │
│  │  └─────────────────────┘  └────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

### 4.2 Database Schema

#### Metadata Store (redb)

```rust
// Table definitions
const NOTES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("notes");
const LINKS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("links");
const TAGS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("tags");
const CACHE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("cache");

// Note metadata schema
struct NoteMetadata {
    id: String,
    path: String,
    title: Option<String>,
    created_at: i64,
    modified_at: i64,
    size: u64,
    frontmatter_hash: u64,
}

// Link schema
struct LinkRecord {
    source: String,
    target: String,
    link_type: LinkType,
    position: Position,
}

// Tag schema
struct TagRecord {
    tag: String,
    note_id: String,
    position: Position,
}
```

#### Vector Index (ruvector)

```rust
// Vector configuration
let config = RuVectorConfig {
    dimensions: 384,          // all-MiniLM-L6-v2 embeddings
    metric: Metric::Cosine,
    m: 32,                    // HNSW connections
    ef_construction: 200,
    ef_search: 100,
};

// Vector metadata
struct VectorMetadata {
    note_id: String,
    chunk_index: usize,       // For long notes
    text_hash: u64,           // Change detection
}
```

#### Full-Text Index (tantivy)

```rust
// Schema definition
let mut schema_builder = Schema::builder();
schema_builder.add_text_field("id", STRING | STORED);
schema_builder.add_text_field("title", TEXT | STORED);
schema_builder.add_text_field("content", TEXT);
schema_builder.add_text_field("tags", STRING | STORED);
schema_builder.add_date_field("created", INDEXED);
schema_builder.add_date_field("modified", INDEXED);
```

---

## 5. Plugin Architecture

### 5.1 Plugin Runtime Design

```
┌────────────────────────────────────────────────────────────────────┐
│                        Plugin Manager                              │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    Plugin Registry                            │ │
│  │                                                               │ │
│  │  plugins: HashMap<String, PluginInstance>                     │ │
│  │  manifests: HashMap<String, PluginManifest>                   │ │
│  │  enabled: HashSet<String>                                     │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    deno_core Runtime                          │ │
│  │                                                               │ │
│  │  ┌────────────────────────────────────────────────────────┐  │ │
│  │  │                    V8 Isolate                           │  │ │
│  │  │                                                         │  │ │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │  │ │
│  │  │  │  Plugin A    │  │  Plugin B    │  │  Plugin C    │ │  │ │
│  │  │  │  (sandboxed) │  │  (sandboxed) │  │  (sandboxed) │ │  │ │
│  │  │  └──────────────┘  └──────────────┘  └──────────────┘ │  │ │
│  │  │                                                         │  │ │
│  │  └────────────────────────────────────────────────────────┘  │ │
│  │                                                               │ │
│  │  ┌────────────────────────────────────────────────────────┐  │ │
│  │  │               Obsidian API Bindings                     │  │ │
│  │  │                                                         │  │ │
│  │  │  op_vault_read, op_vault_write, op_vault_create         │  │ │
│  │  │  op_workspace_get_active, op_workspace_open_file        │  │ │
│  │  │  op_metadata_get_cache, op_metadata_resolve_link        │  │ │
│  │  │  op_command_register, op_ribbon_add, op_setting_add     │  │ │
│  │  └────────────────────────────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

### 5.2 Plugin API Mapping

```typescript
// JavaScript API (exposed to plugins)
interface ObsidianAPI {
  app: {
    vault: Vault;
    workspace: Workspace;
    metadataCache: MetadataCache;
  };
}

// Maps to Rust ops
const OP_MAPPING = {
  'app.vault.read': 'op_vault_read',
  'app.vault.modify': 'op_vault_modify',
  'app.vault.create': 'op_vault_create',
  'app.vault.delete': 'op_vault_delete',
  'app.workspace.getActiveFile': 'op_workspace_get_active',
  'app.workspace.openLinkText': 'op_workspace_open_link',
  'app.metadataCache.getFileCache': 'op_metadata_get_cache',
  // ... more mappings
};
```

### 5.3 Plugin Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Install   │────>│    Load     │────>│   Enable    │────>│   Active    │
│             │     │             │     │             │     │             │
│ - Download  │     │ - Parse     │     │ - onload()  │     │ - Commands  │
│ - Verify    │     │   manifest  │     │ - Register  │     │ - Events    │
│ - Extract   │     │ - Create    │     │   commands  │     │ - Views     │
│             │     │   runtime   │     │ - Init UI   │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                                                   │
                                               ┌───────────────────┘
                                               │
                                               ▼
                    ┌─────────────┐     ┌─────────────┐
                    │   Disable   │◄────│   Reload    │
                    │             │     │             │
                    │ - onunload()│     │ - Hot       │
                    │ - Cleanup   │     │   reload    │
                    │ - Remove UI │     │   support   │
                    └─────────────┘     └─────────────┘
```

---

## 6. Security Architecture

### 6.1 Security Layers

```
┌────────────────────────────────────────────────────────────────────┐
│                       Security Architecture                        │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    Application Boundary                       │ │
│  │                                                               │ │
│  │  - Tauri security policies                                    │ │
│  │  - CSP (Content Security Policy)                              │ │
│  │  - No arbitrary network access                                │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    Plugin Sandbox                             │ │
│  │                                                               │ │
│  │  - V8 isolates (memory isolation)                             │ │
│  │  - Capability-based permissions                               │ │
│  │  - No direct file system access                               │ │
│  │  - API calls through Rust bindings                            │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    Data Protection                            │ │
│  │                                                               │ │
│  │  - Local-first storage                                        │ │
│  │  - Optional vault encryption (AES-256-GCM)                    │ │
│  │  - No telemetry without consent                               │ │
│  │  - Secure credential storage (OS keychain)                    │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

### 6.2 Plugin Permissions

```rust
pub struct PluginPermissions {
    /// File system access scope
    pub file_access: FileAccessScope,
    /// Network permissions
    pub network: NetworkPermission,
    /// System access
    pub system: SystemPermission,
}

pub enum FileAccessScope {
    /// Only vault directory
    VaultOnly,
    /// Vault + specific paths
    Extended(Vec<PathBuf>),
    /// No file access
    None,
}

pub enum NetworkPermission {
    /// No network access
    None,
    /// Specific domains only
    AllowList(Vec<String>),
    /// User-approved per request
    AskUser,
}

pub enum SystemPermission {
    /// Cannot access system APIs
    None,
    /// Can access clipboard
    Clipboard,
    /// Can open URLs
    OpenUrl,
}
```

---

## 7. Performance Architecture

### 7.1 Caching Strategy

```
┌────────────────────────────────────────────────────────────────────┐
│                       Caching Layers                               │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    L1: Memory Cache                           │ │
│  │                                                               │ │
│  │  - Active note content (LRU, 100 entries)                     │ │
│  │  - Parsed metadata (all notes)                                │ │
│  │  - In-memory graph (petgraph)                                 │ │
│  │  - Search results (recent queries)                            │ │
│  │                                                               │ │
│  │  Access time: <1ms                                            │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    L2: Index Cache                            │ │
│  │                                                               │ │
│  │  - Vector embeddings (ruvector)                               │ │
│  │  - Full-text index (tantivy)                                  │ │
│  │  - Metadata store (redb)                                      │ │
│  │                                                               │ │
│  │  Access time: <10ms                                           │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                    L3: File System                            │ │
│  │                                                               │ │
│  │  - Markdown files                                             │ │
│  │  - Attachments                                                │ │
│  │  - Configuration                                              │ │
│  │                                                               │ │
│  │  Access time: <50ms                                           │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

### 7.2 Async Architecture

```rust
// Async runtime configuration
#[tokio::main]
async fn main() {
    // Multi-threaded runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    // Spawn background tasks
    runtime.spawn(file_watcher_task());
    runtime.spawn(index_updater_task());
    runtime.spawn(graph_sync_task());

    // Run Tauri app
    tauri::Builder::default()
        .setup(|app| {
            // Initialize services
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Background indexing with debouncing
async fn index_updater_task() {
    let mut pending: HashSet<PathBuf> = HashSet::new();
    let mut interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            // Receive file change events
            Some(path) = file_changes.recv() => {
                pending.insert(path);
            }
            // Process batch every 500ms
            _ = interval.tick() => {
                if !pending.is_empty() {
                    let batch: Vec<_> = pending.drain().collect();
                    // Parallel indexing with rayon
                    batch.par_iter().for_each(|path| {
                        index_note(path).unwrap();
                    });
                }
            }
        }
    }
}
```

---

## 8. Deployment Architecture

### 8.1 Build Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                      Build Pipeline                             │
│                                                                 │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐    ┌────────┐ │
│  │  Source   │───>│   Build   │───>│   Test    │───>│ Package│ │
│  │           │    │           │    │           │    │        │ │
│  │ - Rust    │    │ - cargo   │    │ - unit    │    │ - MSI  │ │
│  │ - TS/React│    │ - vite    │    │ - integ   │    │ - DMG  │ │
│  │ - Assets  │    │ - tauri   │    │ - e2e     │    │ - deb  │ │
│  └───────────┘    └───────────┘    └───────────┘    └────────┘ │
│                                                                 │
│  Targets:                                                       │
│  - Windows: x64, ARM64                                          │
│  - macOS: Intel, Apple Silicon                                  │
│  - Linux: x64, ARM64                                            │
└─────────────────────────────────────────────────────────────────┘
```

### 8.2 Release Artifacts

```
releases/
├── windows/
│   ├── obsidian-rs-x64.msi
│   ├── obsidian-rs-arm64.msi
│   └── obsidian-rs-x64.exe          # NSIS installer
├── macos/
│   ├── obsidian-rs-intel.dmg
│   ├── obsidian-rs-arm64.dmg
│   └── obsidian-rs.app.tar.gz       # Universal binary
├── linux/
│   ├── obsidian-rs-x64.deb
│   ├── obsidian-rs-x64.AppImage
│   └── obsidian-rs-arm64.deb
└── checksums.sha256
```

---

## 9. Testing Architecture

### 9.1 Test Pyramid

```
                    ┌─────────────┐
                    │    E2E      │  5%
                    │   Tests     │  (Playwright)
                    └─────────────┘
                   ┌───────────────┐
                   │  Integration  │  15%
                   │    Tests      │  (Rust + Tauri)
                   └───────────────┘
                  ┌─────────────────┐
                  │      Unit       │  80%
                  │     Tests       │  (Rust + Jest)
                  └─────────────────┘
```

### 9.2 Test Categories

```rust
// Unit tests (per crate)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wikilink_parsing() {
        let input = "[[note|display]]";
        let link = parse_wikilink(input).unwrap();
        assert_eq!(link.target, "note");
        assert_eq!(link.display, Some("display"));
    }

    #[tokio::test]
    async fn test_search_performance() {
        let engine = create_test_engine().await;
        let start = Instant::now();
        let results = engine.search("test query", 10).await.unwrap();
        assert!(start.elapsed() < Duration::from_millis(10));
    }
}

// Integration tests
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_note_lifecycle() {
        let vault = create_temp_vault().await;

        // Create
        let note = vault.create("test.md", "# Test").await.unwrap();
        assert!(vault.exists(&note.path));

        // Read
        let content = vault.read(&note).await.unwrap();
        assert_eq!(content, "# Test");

        // Update
        vault.modify(&note, "# Updated").await.unwrap();

        // Delete
        vault.delete(&note).await.unwrap();
        assert!(!vault.exists(&note.path));
    }
}
```

---

## 10. Architecture Decision Records (ADRs)

### ADR-001: Tauri over Electron

**Status:** Accepted

**Context:** Need a cross-platform desktop framework.

**Decision:** Use Tauri v2 instead of Electron.

**Rationale:**
- 10x smaller bundle size
- 3x lower memory usage
- Native Rust backend
- Better security model

### ADR-002: RuVector for Semantic Search

**Status:** Accepted

**Context:** Need vector database for semantic search.

**Decision:** Use ruvector-core over alternatives.

**Rationale:**
- 8x faster than competitors (61μs vs 500μs)
- 96.8% recall accuracy
- Rust-native (no FFI)
- Active development

### ADR-003: deno_core for Plugins

**Status:** Accepted

**Context:** Need JavaScript runtime for Obsidian plugin compatibility.

**Decision:** Use deno_core for plugin execution.

**Rationale:**
- V8 isolates for security
- Full async/await support
- Easy Rust interop
- Smaller than embedding full Node.js

### ADR-004: Hybrid In-Memory + Persistent Graph

**Status:** Accepted

**Context:** Need fast graph queries for UI and complex persistent queries.

**Decision:** Use petgraph for in-memory + ruvector-graph for persistence.

**Rationale:**
- petgraph: Instant UI updates (<1ms)
- ruvector-graph: Complex Cypher queries
- Sync between them on changes

---

## 11. Next Steps

1. **Phase 1: Foundation**
   - Set up Tauri project structure
   - Implement core crates (obsidian-core, obsidian-storage)
   - Basic file operations

2. **Phase 2: Domain Logic**
   - Markdown engine (obsidian-markdown)
   - Search engine (obsidian-search)
   - Graph service (obsidian-graph)

3. **Phase 3: Application**
   - Tauri commands
   - React UI components
   - Integration testing

4. **Phase 4: Plugins**
   - deno_core integration
   - API bindings
   - Plugin marketplace

---

**Document Status:** ARCHITECTURE COMPLETE
**Next Phase:** PSEUDOCODE
**Author:** SPARC Architecture Agent
**Date:** 2025-12-07
