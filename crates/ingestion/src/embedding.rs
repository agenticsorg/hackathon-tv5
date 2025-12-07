//! Content embedding generation
//!
//! Implements the GenerateContentEmbedding algorithm from SPARC specification:
//! - Text embedding (title + overview): weight 0.4
//! - Metadata embedding (genres, year, ratings): weight 0.3
//! - Graph embedding (relationships): weight 0.3
//! - L2 normalization
//! - Complexity: O(d) where d=768 (embedding dimensions)

use crate::{normalizer::CanonicalContent, IngestionError, Result};
use ndarray::{Array, Array1};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Embedding dimension (standard for sentence transformers)
const EMBEDDING_DIM: usize = 768;

/// Embedding weights for different components
const TEXT_WEIGHT: f32 = 0.4;
const METADATA_WEIGHT: f32 = 0.3;
const GRAPH_WEIGHT: f32 = 0.3;

/// Embedding generator for content
pub struct EmbeddingGenerator {
    // In production, this would use a real embedding model (e.g., sentence-transformers)
    // For now, we implement a simplified version using feature hashing
    genre_embeddings: HashMap<String, Vec<f32>>,
}

impl EmbeddingGenerator {
    /// Create a new embedding generator
    pub fn new() -> Self {
        let mut generator = Self {
            genre_embeddings: HashMap::new(),
        };

        // Initialize genre embeddings (simplified)
        generator.init_genre_embeddings();

        generator
    }

    /// Generate content embedding
    ///
    /// Combines text, metadata, and graph embeddings with weighted sum.
    /// Complexity: O(d) where d=768
    pub async fn generate(&self, content: &CanonicalContent) -> Result<Vec<f32>> {
        // Generate component embeddings
        let text_embedding = self.generate_text_embedding(content);
        let metadata_embedding = self.generate_metadata_embedding(content);
        let graph_embedding = self.generate_graph_embedding(content);

        // Weighted combination
        let combined =
            self.combine_embeddings(&text_embedding, &metadata_embedding, &graph_embedding);

        // L2 normalization
        let normalized = Self::l2_normalize(&combined);

        Ok(normalized)
    }

    /// Generate text embedding from title and overview
    ///
    /// In production, this would use a sentence transformer model.
    /// For now, we use a simplified feature hashing approach.
    fn generate_text_embedding(&self, content: &CanonicalContent) -> Vec<f32> {
        let text = format!(
            "{} {}",
            content.title,
            content.overview.as_deref().unwrap_or("")
        );

        self.text_to_embedding(&text)
    }

    /// Generate metadata embedding from genres, year, and ratings
    fn generate_metadata_embedding(&self, content: &CanonicalContent) -> Vec<f32> {
        let mut embedding = vec![0.0; EMBEDDING_DIM];

        // Genre contribution (50% of metadata embedding)
        for genre in &content.genres {
            if let Some(genre_emb) = self.genre_embeddings.get(genre) {
                for (i, val) in genre_emb.iter().enumerate() {
                    embedding[i] += val * 0.5;
                }
            }
        }

        // Year contribution (25% of metadata embedding)
        if let Some(year) = content.release_year {
            // Normalize year to [0, 1] range (1900-2030)
            let normalized_year = ((year - 1900) as f32 / 130.0).clamp(0.0, 1.0);
            // Encode year in first few dimensions
            embedding[0] += normalized_year * 0.25;
            embedding[1] += (1.0 - normalized_year) * 0.25;
        }

        // Rating contribution (25% of metadata embedding)
        if let Some(rating) = content.user_rating {
            // Normalize rating to [0, 1] range (0-10)
            let normalized_rating = (rating / 10.0).clamp(0.0, 1.0);
            // Encode rating in specific dimensions
            embedding[2] += normalized_rating * 0.25;
            embedding[3] += (1.0 - normalized_rating) * 0.25;
        }

        embedding
    }

    /// Generate graph embedding from content relationships
    ///
    /// In production, this would use graph neural networks or graph embeddings.
    /// For now, we return a placeholder that incorporates content type and platform.
    fn generate_graph_embedding(&self, content: &CanonicalContent) -> Vec<f32> {
        let mut embedding = vec![0.0; EMBEDDING_DIM];

        // Platform encoding
        let platform_hash = self.hash_string(&content.platform_id) % EMBEDDING_DIM;
        embedding[platform_hash] = 1.0;

        // Content type encoding
        let type_str = format!("{:?}", content.content_type);
        let type_hash = self.hash_string(&type_str) % EMBEDDING_DIM;
        embedding[type_hash] = 1.0;

        embedding
    }

    /// Combine embeddings with weighted sum
    fn combine_embeddings(&self, text: &[f32], metadata: &[f32], graph: &[f32]) -> Vec<f32> {
        let mut combined = vec![0.0; EMBEDDING_DIM];

        for i in 0..EMBEDDING_DIM {
            combined[i] =
                text[i] * TEXT_WEIGHT + metadata[i] * METADATA_WEIGHT + graph[i] * GRAPH_WEIGHT;
        }

        combined
    }

    /// L2 normalization
    ///
    /// Normalize embedding to unit length for cosine similarity.
    fn l2_normalize(embedding: &[f32]) -> Vec<f32> {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm == 0.0 {
            return embedding.to_vec();
        }

        embedding.iter().map(|x| x / norm).collect()
    }

    /// Convert text to embedding using simple feature hashing
    ///
    /// In production, use a proper sentence transformer model.
    fn text_to_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; EMBEDDING_DIM];

        // Tokenize and hash
        let lowercase = text.to_lowercase();
        let words: Vec<&str> = lowercase.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            let hash = self.hash_string(word) % EMBEDDING_DIM;
            // Use position weighting (earlier words more important)
            let weight = 1.0 / (1.0 + i as f32 * 0.1);
            embedding[hash] += weight;
        }

        // Apply non-linearity (tanh)
        for val in embedding.iter_mut() {
            *val = val.tanh();
        }

        embedding
    }

    /// Simple hash function for strings
    fn hash_string(&self, s: &str) -> usize {
        let mut hash: usize = 5381;
        for byte in s.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as usize);
        }
        hash
    }

    /// Initialize genre embeddings
    ///
    /// In production, these would be learned from data.
    /// For now, we create random but consistent embeddings for each genre.
    fn init_genre_embeddings(&mut self) {
        let genres = vec![
            "Action",
            "Adventure",
            "Animation",
            "Comedy",
            "Crime",
            "Documentary",
            "Drama",
            "Family",
            "Fantasy",
            "History",
            "Horror",
            "Music",
            "Mystery",
            "Romance",
            "Science Fiction",
            "Thriller",
            "War",
            "Western",
        ];

        for genre in genres {
            let mut embedding = vec![0.0; EMBEDDING_DIM];

            // Use genre hash to create consistent random-like pattern
            let base_hash = self.hash_string(genre);

            for i in 0..EMBEDDING_DIM {
                // Create pseudo-random values based on hash
                let val_hash = base_hash.wrapping_add(i * 13);
                let normalized = ((val_hash % 1000) as f32 / 1000.0) * 2.0 - 1.0;
                embedding[i] = normalized;
            }

            // Normalize
            let normalized = Self::l2_normalize(&embedding);
            self.genre_embeddings.insert(genre.to_string(), normalized);
        }
    }
}

impl Default for EmbeddingGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::{AvailabilityInfo, ContentType, ImageSet};

    #[test]
    fn test_l2_normalization() {
        let embedding = vec![3.0, 4.0, 0.0];
        let normalized = EmbeddingGenerator::l2_normalize(&embedding);

        // Length should be 1
        let length: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((length - 1.0).abs() < 0.001);

        // Should be [0.6, 0.8, 0.0]
        assert!((normalized[0] - 0.6).abs() < 0.001);
        assert!((normalized[1] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_embedding_dimensions() {
        let generator = EmbeddingGenerator::new();
        let text_emb = generator.text_to_embedding("The Matrix");

        assert_eq!(text_emb.len(), EMBEDDING_DIM);
    }

    #[tokio::test]
    async fn test_generate_embedding() {
        let generator = EmbeddingGenerator::new();

        let content = CanonicalContent {
            platform_content_id: "test".to_string(),
            platform_id: "netflix".to_string(),
            entity_id: None,
            title: "The Matrix".to_string(),
            overview: Some("A computer hacker learns about the true nature of reality".to_string()),
            content_type: ContentType::Movie,
            release_year: Some(1999),
            runtime_minutes: Some(136),
            genres: vec!["Action".to_string(), "Science Fiction".to_string()],
            external_ids: std::collections::HashMap::new(),
            availability: AvailabilityInfo {
                regions: vec![],
                subscription_required: false,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            },
            images: ImageSet::default(),
            rating: None,
            user_rating: Some(8.7),
            embedding: None,
            updated_at: chrono::Utc::now(),
        };

        let embedding = generator.generate(&content).await.unwrap();

        // Check dimensions
        assert_eq!(embedding.len(), EMBEDDING_DIM);

        // Check L2 normalization
        let length: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((length - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_genre_embeddings_initialized() {
        let generator = EmbeddingGenerator::new();

        assert!(generator.genre_embeddings.contains_key("Action"));
        assert!(generator.genre_embeddings.contains_key("Science Fiction"));
        assert!(generator.genre_embeddings.contains_key("Comedy"));

        // Check that each genre embedding is normalized
        for (_, embedding) in &generator.genre_embeddings {
            let length: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((length - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_hash_consistency() {
        let generator = EmbeddingGenerator::new();

        let hash1 = generator.hash_string("Action");
        let hash2 = generator.hash_string("Action");

        assert_eq!(hash1, hash2);
    }
}
