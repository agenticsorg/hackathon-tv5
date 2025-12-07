# Tauri Implementation Guide for Obsidian Clone

**Quick Reference for Building an Obsidian Clone with Tauri v2**

---

## Quick Start

```bash
# Create new Tauri project
npm create tauri-app@latest

# Project options:
# - Name: obsidian-rust
# - Template: React + TypeScript
# - Package manager: npm

cd obsidian-rust

# Install dependencies
npm install @monaco-editor/react cytoscape cytoscape-cola react-markdown rehype-highlight remark-gfm

# Start development
npm run tauri dev
```

---

## Project Structure

```
obsidian-rust/
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── main.rs                 # Entry point
│   │   ├── vault/
│   │   │   ├── mod.rs              # Vault module
│   │   │   ├── manager.rs          # File operations
│   │   │   ├── watcher.rs          # File system watcher
│   │   │   └── parser.rs           # Markdown parser
│   │   ├── search/
│   │   │   ├── mod.rs
│   │   │   └── indexer.rs          # Full-text search
│   │   ├── graph/
│   │   │   ├── mod.rs
│   │   │   └── builder.rs          # Graph data generator
│   │   └── plugins/
│   │       ├── mod.rs
│   │       └── manager.rs          # Plugin system
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                            # React frontend
│   ├── components/
│   │   ├── Editor.tsx              # Monaco editor
│   │   ├── Graph.tsx               # Cytoscape graph
│   │   ├── Sidebar.tsx             # File tree
│   │   ├── Preview.tsx             # Markdown preview
│   │   └── SearchBar.tsx           # Search interface
│   ├── plugins/
│   │   ├── api.ts                  # Plugin API
│   │   └── obsidian-compat.ts      # Obsidian compatibility
│   ├── App.tsx
│   ├── main.tsx
│   └── styles.css
│
├── package.json
└── README.md
```

---

## Backend Implementation

### 1. Cargo Dependencies

```toml
# src-tauri/Cargo.toml

[dependencies]
tauri = { version = "2.0", features = ["protocol-asset"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }

# File system
notify = "6"
walkdir = "2"

# Markdown parsing
pulldown-cmark = "0.9"
gray_matter = "0.2"

# Search
tantivy = "0.21"

# Graph processing
petgraph = "0.6"

# Async
futures = "0.3"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }
```

### 2. Vault Manager

```rust
// src-tauri/src/vault/manager.rs

use notify::{Watcher, RecursiveMode, Event};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub frontmatter: Option<FrontMatter>,
    pub links: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub aliases: Option<Vec<String>>,
}

pub struct VaultManager {
    root: PathBuf,
    notes: Arc<RwLock<Vec<Note>>>,
    watcher: Option<notify::RecommendedWatcher>,
}

impl VaultManager {
    pub fn new(root: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let notes = Arc::new(RwLock::new(Vec::new()));

        Ok(Self {
            root,
            notes,
            watcher: None,
        })
    }

    pub async fn scan_vault(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut notes = self.notes.write().await;
        notes.clear();

        for entry in WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(note) = self.parse_note(path).await {
                    notes.push(note);
                }
            }
        }

        Ok(())
    }

    async fn parse_note(&self, path: &Path) -> Result<Note, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(path).await?;

        // Parse frontmatter
        let (frontmatter, body) = self.parse_frontmatter(&content);

        // Extract title
        let title = frontmatter
            .as_ref()
            .and_then(|f| f.title.clone())
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(String::from)
            })
            .unwrap_or_default();

        // Extract wiki links [[Note Name]]
        let links = self.extract_links(&body);

        // Extract tags #tag
        let tags = self.extract_tags(&body);

        Ok(Note {
            path: path.to_path_buf(),
            title,
            content: body,
            frontmatter,
            links,
            tags,
        })
    }

    fn parse_frontmatter(&self, content: &str) -> (Option<FrontMatter>, String) {
        if content.starts_with("---\n") {
            if let Some(end) = content[4..].find("\n---\n") {
                let yaml = &content[4..end + 4];
                let body = &content[end + 8..];

                if let Ok(fm) = serde_yaml::from_str::<FrontMatter>(yaml) {
                    return (Some(fm), body.to_string());
                }
            }
        }
        (None, content.to_string())
    }

    fn extract_links(&self, content: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        re.captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    fn extract_tags(&self, content: &str) -> Vec<String> {
        let re = regex::Regex::new(r"#([a-zA-Z0-9_-]+)").unwrap();
        re.captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    pub async fn get_all_notes(&self) -> Vec<Note> {
        self.notes.read().await.clone()
    }

    pub async fn find_note(&self, title: &str) -> Option<Note> {
        self.notes
            .read()
            .await
            .iter()
            .find(|n| n.title == title)
            .cloned()
    }
}

// Tauri commands
use tauri::State;

#[tauri::command]
pub async fn get_all_notes(
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<Vec<Note>, String> {
    let vault = vault.read().await;
    Ok(vault.get_all_notes().await)
}

#[tauri::command]
pub async fn read_note(
    path: String,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<Note, String> {
    let vault = vault.read().await;
    vault
        .parse_note(Path::new(&path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_note(
    path: String,
    content: String,
) -> Result<(), String> {
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_notes(
    query: String,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<Vec<Note>, String> {
    let vault = vault.read().await;
    let notes = vault.get_all_notes().await;

    let results: Vec<Note> = notes
        .into_iter()
        .filter(|note| {
            note.title.to_lowercase().contains(&query.to_lowercase())
                || note.content.to_lowercase().contains(&query.to_lowercase())
        })
        .collect();

    Ok(results)
}
```

### 3. Graph Builder

```rust
// src-tauri/src/graph/builder.rs

use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub size: usize,
    pub centrality: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub weight: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphBuilder {
    graph: DiGraph<String, f32>,
    node_map: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn build_from_notes(&mut self, notes: &[crate::vault::manager::Note]) {
        // Add all notes as nodes
        for note in notes {
            let idx = self.graph.add_node(note.title.clone());
            self.node_map.insert(note.title.clone(), idx);
        }

        // Add edges for links
        for note in notes {
            if let Some(source_idx) = self.node_map.get(&note.title) {
                for link in &note.links {
                    if let Some(target_idx) = self.node_map.get(link) {
                        self.graph.add_edge(*source_idx, *target_idx, 1.0);
                    }
                }
            }
        }
    }

    pub fn to_cytoscape_format(&self) -> GraphData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Convert nodes
        for (title, idx) in &self.node_map {
            let degree = self.graph.edges(*idx).count();
            nodes.push(GraphNode {
                id: title.clone(),
                label: title.clone(),
                size: degree,
                centrality: 1.0, // TODO: Calculate actual centrality
            });
        }

        // Convert edges
        for edge in self.graph.edge_references() {
            let source = &self.graph[edge.source()];
            let target = &self.graph[edge.target()];
            edges.push(GraphEdge {
                source: source.clone(),
                target: target.clone(),
                weight: *edge.weight(),
            });
        }

        GraphData { nodes, edges }
    }
}

#[tauri::command]
pub async fn get_graph_data(
    vault: tauri::State<'_, std::sync::Arc<tokio::sync::RwLock<crate::vault::manager::VaultManager>>>,
) -> Result<GraphData, String> {
    let vault = vault.read().await;
    let notes = vault.get_all_notes().await;

    let mut builder = GraphBuilder::new();
    builder.build_from_notes(&notes);

    Ok(builder.to_cytoscape_format())
}
```

### 4. Main Entry Point

```rust
// src-tauri/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod vault;
mod graph;

use std::sync::Arc;
use tokio::sync::RwLock;
use vault::manager::VaultManager;

#[tokio::main]
async fn main() {
    // Initialize vault manager
    let vault_path = std::env::var("VAULT_PATH")
        .unwrap_or_else(|_| "./vault".to_string());

    let vault_manager = VaultManager::new(vault_path.into())
        .expect("Failed to create vault manager");

    // Scan vault on startup
    vault_manager.scan_vault().await.ok();

    let vault_state = Arc::new(RwLock::new(vault_manager));

    tauri::Builder::default()
        .manage(vault_state)
        .invoke_handler(tauri::generate_handler![
            vault::manager::get_all_notes,
            vault::manager::read_note,
            vault::manager::save_note,
            vault::manager::search_notes,
            graph::builder::get_graph_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Frontend Implementation

### 1. Monaco Editor Component

```typescript
// src/components/Editor.tsx

import React, { useState, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import { invoke } from '@tauri-apps/api/core';

interface EditorProps {
    filePath: string | null;
}

export function MarkdownEditor({ filePath }: EditorProps) {
    const [content, setContent] = useState('');
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        if (!filePath) return;

        setLoading(true);
        invoke<{ content: string }>('read_note', { path: filePath })
            .then((note) => {
                setContent(note.content);
            })
            .catch(console.error)
            .finally(() => setLoading(false));
    }, [filePath]);

    const handleChange = async (value: string | undefined) => {
        if (!value || !filePath) return;

        setContent(value);

        // Debounced save
        await invoke('save_note', {
            path: filePath,
            content: value,
        });
    };

    const handleEditorMount = (editor: any, monaco: any) => {
        // Register Obsidian-style markdown language
        monaco.languages.register({ id: 'obsidian-markdown' });

        // Add wiki-link syntax highlighting
        monaco.languages.setMonarchTokensProvider('obsidian-markdown', {
            tokenizer: {
                root: [
                    [/\[\[.*?\]\]/, 'wiki-link'],
                    [/#[a-zA-Z0-9_-]+/, 'tag'],
                    [/^#{1,6}\s.*$/, 'heading'],
                    [/`[^`]+`/, 'code'],
                    [/```[\s\S]*?```/, 'code-block'],
                ],
            },
        });

        // Define custom theme
        monaco.editor.defineTheme('obsidian-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [
                { token: 'wiki-link', foreground: '7c9cff', fontStyle: 'bold' },
                { token: 'tag', foreground: 'ff9d00' },
                { token: 'heading', foreground: 'ff79c6', fontStyle: 'bold' },
            ],
            colors: {
                'editor.background': '#1e1e1e',
                'editor.foreground': '#d4d4d4',
            },
        });

        monaco.editor.setTheme('obsidian-dark');

        // Add autocomplete for note links
        monaco.languages.registerCompletionItemProvider('obsidian-markdown', {
            provideCompletionItems: async (model: any, position: any) => {
                const notes = await invoke<any[]>('get_all_notes');
                return {
                    suggestions: notes.map((note) => ({
                        label: note.title,
                        kind: monaco.languages.CompletionItemKind.Reference,
                        insertText: `[[${note.title}]]`,
                        documentation: note.path,
                    })),
                };
            },
        });
    };

    if (loading) return <div>Loading...</div>;

    return (
        <Editor
            height="100vh"
            language="obsidian-markdown"
            value={content}
            onChange={handleChange}
            onMount={handleEditorMount}
            options={{
                fontSize: 14,
                fontFamily: 'JetBrains Mono, monospace',
                minimap: { enabled: true },
                wordWrap: 'on',
                lineNumbers: 'off',
                renderWhitespace: 'selection',
            }}
        />
    );
}
```

### 2. Graph Visualization

```typescript
// src/components/Graph.tsx

import React, { useEffect, useRef } from 'react';
import cytoscape from 'cytoscape';
import cola from 'cytoscape-cola';
import { invoke } from '@tauri-apps/api/core';

cytoscape.use(cola);

interface GraphData {
    nodes: Array<{
        id: string;
        label: string;
        size: number;
        centrality: number;
    }>;
    edges: Array<{
        source: string;
        target: string;
        weight: number;
    }>;
}

export function GraphView() {
    const containerRef = useRef<HTMLDivElement>(null);
    const cyRef = useRef<any>(null);

    useEffect(() => {
        if (!containerRef.current) return;

        // Initialize Cytoscape
        cyRef.current = cytoscape({
            container: containerRef.current,
            style: [
                {
                    selector: 'node',
                    style: {
                        'background-color': '#7c9cff',
                        'label': 'data(label)',
                        'width': 'data(size)',
                        'height': 'data(size)',
                        'font-size': '12px',
                        'text-valign': 'center',
                        'text-halign': 'center',
                        'color': '#fff',
                    },
                },
                {
                    selector: 'edge',
                    style: {
                        'width': 2,
                        'line-color': '#999',
                        'target-arrow-color': '#999',
                        'target-arrow-shape': 'triangle',
                        'curve-style': 'bezier',
                    },
                },
            ],
        });

        // Load graph data from Rust backend
        loadGraphData();

        return () => {
            if (cyRef.current) {
                cyRef.current.destroy();
            }
        };
    }, []);

    const loadGraphData = async () => {
        try {
            const data = await invoke<GraphData>('get_graph_data');

            const elements = [
                ...data.nodes.map((node) => ({
                    data: {
                        id: node.id,
                        label: node.label,
                        size: Math.max(20, node.size * 10),
                    },
                })),
                ...data.edges.map((edge) => ({
                    data: {
                        source: edge.source,
                        target: edge.target,
                    },
                })),
            ];

            cyRef.current.add(elements);

            // Apply force-directed layout
            cyRef.current.layout({
                name: 'cola',
                animate: true,
                randomize: false,
                maxSimulationTime: 2000,
                nodeSpacing: 50,
            }).run();
        } catch (error) {
            console.error('Failed to load graph data:', error);
        }
    };

    return (
        <div
            ref={containerRef}
            style={{
                width: '100%',
                height: '100%',
                background: '#1e1e1e',
            }}
        />
    );
}
```

### 3. Main App

```typescript
// src/App.tsx

import React, { useState } from 'react';
import { MarkdownEditor } from './components/Editor';
import { GraphView } from './components/Graph';
import { Sidebar } from './components/Sidebar';
import './App.css';

export default function App() {
    const [currentFile, setCurrentFile] = useState<string | null>(null);
    const [activeView, setActiveView] = useState<'editor' | 'graph'>('editor');

    return (
        <div className="app">
            <div className="sidebar">
                <Sidebar onFileSelect={setCurrentFile} />
            </div>

            <div className="main-content">
                <div className="toolbar">
                    <button
                        onClick={() => setActiveView('editor')}
                        className={activeView === 'editor' ? 'active' : ''}
                    >
                        Editor
                    </button>
                    <button
                        onClick={() => setActiveView('graph')}
                        className={activeView === 'graph' ? 'active' : ''}
                    >
                        Graph
                    </button>
                </div>

                <div className="view-container">
                    {activeView === 'editor' ? (
                        <MarkdownEditor filePath={currentFile} />
                    ) : (
                        <GraphView />
                    )}
                </div>
            </div>
        </div>
    );
}
```

---

## Performance Optimizations

### 1. Lazy Loading for Large Vaults

```rust
// Implement pagination for large vaults
#[tauri::command]
pub async fn get_notes_paginated(
    offset: usize,
    limit: usize,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<Vec<Note>, String> {
    let vault = vault.read().await;
    let notes = vault.get_all_notes().await;

    Ok(notes
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect())
}
```

### 2. Incremental Search with Tantivy

```rust
// Add tantivy for fast full-text search
use tantivy::*;

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
}

impl SearchIndex {
    pub fn new() -> Result<Self, tantivy::TantivyError> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT);
        schema_builder.add_text_field("path", STRING | STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema);
        let reader = index.reader()?;

        Ok(Self { index, reader })
    }

    pub fn index_note(&self, note: &Note) -> Result<(), tantivy::TantivyError> {
        let mut index_writer = self.index.writer(50_000_000)?;

        let schema = self.index.schema();
        let title = schema.get_field("title").unwrap();
        let content = schema.get_field("content").unwrap();
        let path = schema.get_field("path").unwrap();

        let mut doc = Document::new();
        doc.add_text(title, &note.title);
        doc.add_text(content, &note.content);
        doc.add_text(path, note.path.to_str().unwrap());

        index_writer.add_document(doc)?;
        index_writer.commit()?;

        Ok(())
    }

    pub fn search(&self, query_str: &str) -> Result<Vec<SearchResult>, tantivy::TantivyError> {
        let searcher = self.reader.searcher();
        let schema = self.index.schema();

        let query_parser = QueryParser::for_index(&self.index, vec![
            schema.get_field("title").unwrap(),
            schema.get_field("content").unwrap(),
        ]);

        let query = query_parser.parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(100))?;

        // Convert to SearchResult
        Ok(vec![]) // TODO: Implement conversion
    }
}
```

---

## Next Steps

1. **Set up project**: `npm create tauri-app@latest`
2. **Implement vault manager**: File I/O and markdown parsing
3. **Add Monaco editor**: Text editing with syntax highlighting
4. **Build graph view**: Cytoscape.js visualization
5. **Add search**: tantivy full-text search
6. **Plugin system**: JavaScript plugin compatibility layer
7. **Polish UI**: Theming, settings, keyboard shortcuts

---

## Resources

- **Tauri Docs**: https://tauri.app/v2/
- **Monaco Editor**: https://microsoft.github.io/monaco-editor/
- **Cytoscape.js**: https://js.cytoscape.org/
- **petgraph**: https://docs.rs/petgraph/
- **tantivy**: https://docs.rs/tantivy/

---

**Estimated Timeline:**
- Week 1-2: Core vault + editor
- Week 3-4: Graph + search
- Week 5-6: Plugins + polish
- **Total**: 6 weeks to MVP
