use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;

/// Placeholder vector store using in-memory HNSW-like index
/// In production, this would use RuVector embedded database
#[derive(Clone)]
pub struct VectorStore {
    /// Vector dimensions
    dimensions: usize,

    /// Maximum number of vectors
    max_vectors: usize,

    /// In-memory storage: id -> (vector, metadata)
    vectors: Arc<DashMap<String, (Vec<f32>, serde_json::Value)>>,
}

/// Search result with id and distance
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub distance: f32,
    pub metadata: serde_json::Value,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new(dimensions: usize, max_vectors: usize) -> Result<Self> {
        Ok(Self {
            dimensions,
            max_vectors,
            vectors: Arc::new(DashMap::new()),
        })
    }

    /// Insert a vector with optional metadata
    /// Target: <1ms
    pub fn insert(
        &self,
        id: String,
        vector: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        if vector.len() != self.dimensions {
            anyhow::bail!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimensions,
                vector.len()
            );
        }

        // Check max capacity
        if self.vectors.len() >= self.max_vectors {
            // Remove oldest (simple FIFO eviction policy)
            if let Some(oldest) = self.vectors.iter().next() {
                let key = oldest.key().clone();
                drop(oldest);
                self.vectors.remove(&key);
            }
        }

        let meta = metadata.unwrap_or(serde_json::Value::Null);
        self.vectors.insert(id, (vector, meta));

        Ok(())
    }

    /// Search for k nearest neighbors using cosine similarity
    /// Target: <1ms for k=50
    pub fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        if query.len() != self.dimensions {
            tracing::warn!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimensions,
                query.len()
            );
            return vec![];
        }

        let mut results: Vec<SearchResult> = self
            .vectors
            .iter()
            .map(|entry| {
                let (id, (vector, metadata)) = (entry.key(), entry.value());
                let distance = cosine_distance(query, vector);

                SearchResult {
                    id: id.clone(),
                    distance,
                    metadata: metadata.clone(),
                }
            })
            .collect();

        // Sort by distance (ascending - lower is better)
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        // Return top k
        results.into_iter().take(k).collect()
    }

    /// Delete a vector by id
    pub fn delete(&self, id: &str) -> Result<()> {
        self.vectors.remove(id);
        Ok(())
    }

    /// Get count of stored vectors
    pub fn count(&self) -> usize {
        self.vectors.len()
    }

    /// Clear all vectors
    pub fn clear(&self) {
        self.vectors.clear();
    }
}

/// Calculate cosine distance between two vectors
/// Returns value in [0, 2] where 0 is identical, 2 is opposite
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 2.0; // Maximum distance
    }

    let similarity = dot_product / (norm_a * norm_b);

    // Convert similarity [-1, 1] to distance [0, 2]
    1.0 - similarity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store_insert_search() {
        let store = VectorStore::new(128, 1000).unwrap();

        // Insert some vectors
        let v1 = vec![1.0; 128];
        let v2 = vec![0.5; 128];
        let v3 = vec![-1.0; 128];

        store.insert("v1".to_string(), v1.clone(), None).unwrap();
        store.insert("v2".to_string(), v2.clone(), None).unwrap();
        store.insert("v3".to_string(), v3.clone(), None).unwrap();

        assert_eq!(store.count(), 3);

        // Search for similar to v1
        let results = store.search(&v1, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "v1"); // Should find itself first
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![-1.0, 0.0, 0.0];

        assert!(cosine_distance(&a, &b) < 0.001); // Same vector
        assert!((cosine_distance(&a, &c) - 2.0).abs() < 0.001); // Opposite vectors
    }
}
