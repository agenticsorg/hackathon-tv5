//! Semantic/vector search using embeddings

use crate::error::{SearchError, SearchResult};
use crate::{MatchType, SearchHit};
use ndarray::{Array1, ArrayView1};
use obsidian_core::note::Note;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

/// Vector dimension for embeddings
const VECTOR_DIM: usize = 384;

/// A stored vector with metadata
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

/// In-memory vector store
/// In production, this would use ruvector or similar
pub struct VectorStore {
    /// Stored vectors
    vectors: RwLock<Vec<StoredVector>>,
    /// Path to vector index
    index_by_path: RwLock<HashMap<String, Vec<usize>>>,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new() -> Self {
        Self {
            vectors: RwLock::new(Vec::new()),
            index_by_path: RwLock::new(HashMap::new()),
        }
    }

    /// Add a vector
    pub fn add(&self, vector: StoredVector) -> usize {
        let mut vectors = self.vectors.write();
        let idx = vectors.len();
        let path = vector.path.clone();
        vectors.push(vector);

        let mut index = self.index_by_path.write();
        index.entry(path).or_insert_with(Vec::new).push(idx);

        idx
    }

    /// Remove all vectors for a path
    pub fn remove(&self, path: &str) {
        let mut index = self.index_by_path.write();
        index.remove(path);
        // Note: We don't actually remove from vectors to avoid index invalidation
        // In production, use proper deletion with compaction
    }

    /// Search for similar vectors
    pub fn search(&self, query_vector: &[f32], limit: usize) -> Vec<(usize, f32)> {
        let vectors = self.vectors.read();
        let index = self.index_by_path.read();

        // Get all valid indices
        let valid_indices: std::collections::HashSet<usize> =
            index.values().flatten().copied().collect();

        let query = Array1::from_vec(query_vector.to_vec());

        let mut scores: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .filter(|(idx, _)| valid_indices.contains(idx))
            .map(|(idx, stored)| {
                let doc_vec = Array1::from_vec(stored.vector.clone());
                let similarity = cosine_similarity(query.view(), doc_vec.view());
                (idx, similarity)
            })
            .collect();

        // Sort by similarity descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(limit);

        scores
    }

    /// Get vector by index
    pub fn get(&self, idx: usize) -> Option<StoredVector> {
        let vectors = self.vectors.read();
        vectors.get(idx).cloned()
    }

    /// Get number of vectors
    pub fn len(&self) -> usize {
        self.index_by_path.read().values().map(|v| v.len()).sum()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
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
/// In production, use a proper embedding model
pub struct SimpleEmbedder;

impl SimpleEmbedder {
    /// Create a new embedder
    pub fn new() -> Self {
        Self
    }

    /// Embed text into a vector
    /// This is a simple bag-of-words approach for demonstration
    /// In production, use sentence-transformers or similar
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; VECTOR_DIM];

        // Simple character n-gram hashing
        let text_lower = text.to_lowercase();
        let chars: Vec<char> = text_lower.chars().collect();

        for i in 0..chars.len().saturating_sub(2) {
            let trigram: String = chars[i..i + 3].iter().collect();
            let hash = Self::simple_hash(&trigram);
            let idx = (hash as usize) % VECTOR_DIM;
            vector[idx] += 1.0;
        }

        // Also add word-level features
        for word in text_lower.split_whitespace() {
            let hash = Self::simple_hash(word);
            let idx = (hash as usize) % VECTOR_DIM;
            vector[idx] += 2.0; // Words weighted more
        }

        // Normalize
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        vector
    }

    /// Simple string hash
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

/// Semantic search index
pub struct SemanticIndex {
    /// Vector store
    store: VectorStore,
    /// Embedder
    embedder: SimpleEmbedder,
}

impl SemanticIndex {
    /// Create a new semantic index
    pub fn new(store: VectorStore) -> Self {
        Self {
            store,
            embedder: SimpleEmbedder::new(),
        }
    }

    /// Index a note
    pub fn index_note(&self, note: &Note) -> SearchResult<()> {
        debug!("Indexing note semantically: {}", note.path);

        // Remove existing vectors
        self.store.remove(&note.path);

        let title = note
            .frontmatter
            .title
            .clone()
            .unwrap_or_else(|| note.basename.clone());

        // Chunk the content
        let chunks = self.chunk_text(&note.content);

        for (i, chunk) in chunks.iter().enumerate() {
            let vector = self.embedder.embed(chunk);

            let stored = StoredVector {
                path: note.path.clone(),
                title: if i == 0 {
                    title.clone()
                } else {
                    format!("{} ({})", title, i + 1)
                },
                vector,
                text: chunk.clone(),
            };

            self.store.add(stored);
        }

        Ok(())
    }

    /// Remove a note from the index
    pub fn remove_note(&self, path: &str) -> SearchResult<()> {
        debug!("Removing note from semantic index: {}", path);
        self.store.remove(path);
        Ok(())
    }

    /// Search for similar notes
    pub fn search(&self, query: &str, limit: usize) -> SearchResult<Vec<SearchHit>> {
        debug!("Semantic search for: {}", query);

        let query_vector = self.embedder.embed(query);
        let results = self.store.search(&query_vector, limit);

        let hits: Vec<SearchHit> = results
            .into_iter()
            .filter_map(|(idx, score)| {
                self.store.get(idx).map(|stored| {
                    SearchHit::new(PathBuf::from(&stored.path), score, stored.title)
                        .with_snippet(Self::truncate_text(&stored.text, 150))
                        .with_match_type(MatchType::Semantic)
                })
            })
            .collect();

        Ok(hits)
    }

    /// Chunk text into smaller pieces
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_note(path: &str, title: &str, content: &str) -> Note {
        let mut note = Note::new(title);
        note.path = path.to_string();
        note.content = content.to_string();
        note.frontmatter.title = Some(title.to_string());
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

        let idx = store.add(vector);
        assert_eq!(store.len(), 1);

        let retrieved = store.get(idx).unwrap();
        assert_eq!(retrieved.path, "test.md");

        store.remove("test.md");
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_semantic_index() {
        let store = VectorStore::new();
        let index = SemanticIndex::new(store);

        let note1 = create_test_note("rust.md", "Rust Programming", "Rust is a systems programming language focused on safety and performance.");
        let note2 = create_test_note("python.md", "Python Programming", "Python is a high-level programming language known for readability.");

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
