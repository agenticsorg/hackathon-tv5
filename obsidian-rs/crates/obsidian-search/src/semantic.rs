//! Semantic/vector search using embeddings powered by ruvector

use crate::error::{SearchError, SearchResult};
use crate::{MatchType, SearchHit};
use ndarray::ArrayView1;
use obsidian_core::note::Note;
use parking_lot::RwLock;
use ruvector_core::types::{DbOptions, DistanceMetric};
use ruvector_core::{SearchQuery, VectorDB, VectorEntry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Vector dimension for embeddings
pub const VECTOR_DIM: usize = 384;

/// A stored vector with metadata (for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredVector {
    /// Document path
    pub path: String,
    /// Document title
    pub title: String,
    /// Embedding vector
    pub vector: Vec<f32>,
    /// Text chunk that was embedded
    pub text: String,
}

/// Ruvector-powered vector store with HNSW indexing
pub struct VectorStore {
    /// Ruvector database
    db: Arc<RwLock<VectorDB>>,
    /// Storage path (None for in-memory)
    storage_path: Option<PathBuf>,
    /// Track vectors by path for deletion
    path_to_ids: RwLock<HashMap<String, Vec<String>>>,
    /// Counter for generating unique IDs
    id_counter: RwLock<u64>,
}

impl VectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        let options = DbOptions {
            dimensions: VECTOR_DIM,
            storage_path: String::new(), // Empty for in-memory
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };

        let db = VectorDB::new(options).expect("Failed to create vector database");

        Self {
            db: Arc::new(RwLock::new(db)),
            storage_path: None,
            path_to_ids: RwLock::new(HashMap::new()),
            id_counter: RwLock::new(0),
        }
    }

    /// Create a vector store with persistent storage
    pub fn with_storage(path: impl AsRef<Path>) -> SearchResult<Self> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = DbOptions {
            dimensions: VECTOR_DIM,
            storage_path: path.to_string_lossy().to_string(),
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };

        let db = VectorDB::new(options).map_err(|e| SearchError::Index(e.to_string()))?;

        info!("Opened ruvector database at {:?}", path);

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            storage_path: Some(path.to_path_buf()),
            path_to_ids: RwLock::new(HashMap::new()),
            id_counter: RwLock::new(0),
        })
    }

    /// Generate a unique ID
    fn generate_id(&self) -> String {
        let mut counter = self.id_counter.write();
        *counter += 1;
        format!("vec_{}", *counter)
    }

    /// Add a vector with metadata
    pub fn add(&self, stored: StoredVector) -> SearchResult<String> {
        let id = self.generate_id();

        let entry = VectorEntry {
            id: Some(id.clone()),
            vector: stored.vector,
            metadata: Some(HashMap::from([
                ("path".to_string(), json!(stored.path)),
                ("title".to_string(), json!(stored.title)),
                ("text".to_string(), json!(stored.text)),
            ])),
        };

        {
            let db = self.db.write();
            db.insert(entry).map_err(|e| SearchError::Index(e.to_string()))?;
        }

        // Track the ID for this path
        {
            let mut path_to_ids = self.path_to_ids.write();
            path_to_ids
                .entry(stored.path)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        Ok(id)
    }

    /// Remove all vectors for a document path
    pub fn remove(&self, path: &str) -> SearchResult<()> {
        let ids_to_remove: Vec<String> = {
            let mut path_to_ids = self.path_to_ids.write();
            path_to_ids.remove(path).unwrap_or_default()
        };

        if !ids_to_remove.is_empty() {
            let db = self.db.write();
            for id in &ids_to_remove {
                if let Err(e) = db.delete(id) {
                    warn!("Failed to delete vector {}: {}", id, e);
                }
            }
            debug!("Removed {} vectors for path: {}", ids_to_remove.len(), path);
        }

        Ok(())
    }

    /// Search for similar vectors using HNSW O(log n) search
    pub fn search(&self, query_vector: &[f32], limit: usize) -> SearchResult<Vec<(String, f32, StoredVector)>> {
        let query = SearchQuery {
            vector: query_vector.to_vec(),
            k: limit,
            filter: None,
            ef_search: Some(100), // Higher ef for better recall
        };

        let results = {
            let db = self.db.read();
            db.search(query).map_err(|e| SearchError::Index(e.to_string()))?
        };

        let mut hits = Vec::new();
        for result in results {
            if let Some(metadata) = result.metadata {
                let path = metadata
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let title = metadata
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let text = metadata
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let stored = StoredVector {
                    path,
                    title,
                    vector: result.vector.unwrap_or_default(),
                    text,
                };

                hits.push((result.id, result.score, stored));
            }
        }

        Ok(hits)
    }

    /// Get number of vectors in the store
    pub fn len(&self) -> usize {
        self.path_to_ids.read().values().map(|v| v.len()).sum()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the storage path if persistent
    pub fn storage_path(&self) -> Option<&Path> {
        self.storage_path.as_deref()
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate cosine similarity between two vectors (fallback for testing)
pub fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Simple text embedder
/// In production, replace with sentence-transformers or similar
pub struct SimpleEmbedder;

impl SimpleEmbedder {
    /// Create a new embedder
    pub fn new() -> Self {
        Self
    }

    /// Embed text into a vector
    /// Uses character n-gram + word hashing for fast approximate embeddings
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; VECTOR_DIM];

        // Character n-gram hashing
        let text_lower = text.to_lowercase();
        let chars: Vec<char> = text_lower.chars().collect();

        for i in 0..chars.len().saturating_sub(2) {
            let trigram: String = chars[i..i + 3].iter().collect();
            let hash = Self::simple_hash(&trigram);
            let idx = (hash as usize) % VECTOR_DIM;
            vector[idx] += 1.0;
        }

        // Word-level features (weighted higher)
        for word in text_lower.split_whitespace() {
            let hash = Self::simple_hash(word);
            let idx = (hash as usize) % VECTOR_DIM;
            vector[idx] += 2.0;
        }

        // L2 normalize
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        vector
    }

    /// Simple string hash (djb2)
    fn simple_hash(s: &str) -> u64 {
        let mut hash: u64 = 5381;
        for c in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(c as u64);
        }
        hash
    }
}

impl Default for SimpleEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic search index using ruvector HNSW
pub struct SemanticIndex {
    /// Ruvector-backed vector store
    store: VectorStore,
    /// Text embedder
    embedder: SimpleEmbedder,
}

impl SemanticIndex {
    /// Create a new in-memory semantic index
    pub fn new(store: VectorStore) -> Self {
        Self {
            store,
            embedder: SimpleEmbedder::new(),
        }
    }

    /// Create a semantic index with persistent storage
    pub fn with_storage(path: impl AsRef<Path>) -> SearchResult<Self> {
        let store = VectorStore::with_storage(path)?;
        Ok(Self {
            store,
            embedder: SimpleEmbedder::new(),
        })
    }

    /// Index a note
    pub fn index_note(&self, note: &Note) -> SearchResult<()> {
        debug!("Indexing note semantically: {}", note.id);

        // Remove existing vectors for this note
        self.store.remove(&note.id)?;

        let title = note
            .frontmatter
            .as_ref()
            .and_then(|f| f.title.clone())
            .unwrap_or_else(|| note.basename.clone());

        // Chunk the content for better retrieval
        let chunks = self.chunk_text(&note.content);

        for (i, chunk) in chunks.iter().enumerate() {
            let vector = self.embedder.embed(chunk);

            let stored = StoredVector {
                path: note.id.clone(),
                title: if i == 0 {
                    title.clone()
                } else {
                    format!("{} (chunk {})", title, i + 1)
                },
                vector,
                text: chunk.clone(),
            };

            self.store.add(stored)?;
        }

        Ok(())
    }

    /// Remove a note from the index
    pub fn remove_note(&self, path: &str) -> SearchResult<()> {
        debug!("Removing note from semantic index: {}", path);
        self.store.remove(path)
    }

    /// Search for similar notes using HNSW O(log n) search
    pub fn search(&self, query: &str, limit: usize) -> SearchResult<Vec<SearchHit>> {
        debug!("Semantic search for: {}", query);

        let query_vector = self.embedder.embed(query);
        let results = self.store.search(&query_vector, limit)?;

        let hits: Vec<SearchHit> = results
            .into_iter()
            .map(|(_id, score, stored)| {
                SearchHit::new(PathBuf::from(&stored.path), score, stored.title)
                    .with_snippet(Self::truncate_text(&stored.text, 150))
                    .with_match_type(MatchType::Semantic)
            })
            .collect();

        Ok(hits)
    }

    /// Chunk text into smaller pieces with overlap
    fn chunk_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let chunk_size = 500;
        let overlap = 50;

        for paragraph in text.split("\n\n") {
            let paragraph = paragraph.trim();
            if paragraph.is_empty() {
                continue;
            }

            if current_chunk.len() + paragraph.len() > chunk_size && !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());

                // Keep overlap from end of current chunk
                let overlap_text: String = current_chunk
                    .chars()
                    .rev()
                    .take(overlap)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                current_chunk = overlap_text;
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(paragraph);
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // If no chunks, use the whole text
        if chunks.is_empty() && !text.trim().is_empty() {
            chunks.push(text.trim().to_string());
        }

        chunks
    }

    /// Truncate text to a maximum length
    fn truncate_text(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len])
        }
    }

    /// Get number of indexed chunks
    pub fn num_chunks(&self) -> usize {
        self.store.len()
    }

    /// Get the underlying vector store
    pub fn store(&self) -> &VectorStore {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_note(path: &str, title: &str, content: &str) -> Note {
        use obsidian_core::note::Frontmatter;
        let mut note = Note::new(path, path, content);
        let mut fm = Frontmatter::new();
        fm.title = Some(title.to_string());
        note.frontmatter = Some(fm);
        note
    }

    #[test]
    fn test_cosine_similarity() {
        let a = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let b = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let c = Array1::from_vec(vec![0.0, 1.0, 0.0]);

        assert!((cosine_similarity(a.view(), b.view()) - 1.0).abs() < 0.001);
        assert!((cosine_similarity(a.view(), c.view())).abs() < 0.001);
    }

    #[test]
    fn test_simple_embedder() {
        let embedder = SimpleEmbedder::new();

        let vec1 = embedder.embed("hello world");
        let vec2 = embedder.embed("hello world");
        let vec3 = embedder.embed("goodbye universe");

        // Same text should produce same embedding
        assert_eq!(vec1, vec2);

        // Different text should produce different embedding
        assert_ne!(vec1, vec3);

        // Check normalization
        let norm: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_store() {
        let store = VectorStore::new();

        let vector = StoredVector {
            path: "test.md".to_string(),
            title: "Test".to_string(),
            vector: vec![1.0; VECTOR_DIM],
            text: "Test content".to_string(),
        };

        let _id = store.add(vector).unwrap();
        assert_eq!(store.len(), 1);

        store.remove("test.md").unwrap();
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_semantic_index() {
        let store = VectorStore::new();
        let index = SemanticIndex::new(store);

        let note1 = create_test_note(
            "rust.md",
            "Rust Programming",
            "Rust is a systems programming language focused on safety and performance.",
        );
        let note2 = create_test_note(
            "python.md",
            "Python Programming",
            "Python is a high-level programming language known for readability.",
        );

        index.index_note(&note1).unwrap();
        index.index_note(&note2).unwrap();

        let results = index.search("systems programming language", 10).unwrap();
        assert!(!results.is_empty());
        // Rust note should rank higher for "systems programming"
        assert!(results[0].path.to_string_lossy().contains("rust"));
    }

    #[test]
    fn test_chunk_text() {
        let store = VectorStore::new();
        let index = SemanticIndex::new(store);

        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = index.chunk_text(text);

        assert!(!chunks.is_empty());
        assert!(chunks[0].contains("First paragraph"));
    }
}
