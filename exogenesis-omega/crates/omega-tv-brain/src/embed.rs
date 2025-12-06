use crate::types::{ViewContext, ViewingEvent};
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Mock embedding generator (to be replaced with ONNX MiniLM)
///
/// In production, this would use:
/// - ONNX Runtime with MiniLM model
/// - Text preprocessing (tokenization)
/// - Inference on CPU/NPU
///
/// For now, generates deterministic pseudo-embeddings for testing
pub struct EmbeddingEngine {
    dimension: usize,
}

impl EmbeddingEngine {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Generate embedding from view context
    pub fn embed_context(&self, context: &ViewContext) -> Vec<f32> {
        let text = context.to_string();
        self.embed_text(&text)
    }

    /// Generate embedding from viewing event
    pub fn embed_event(&self, event: &ViewingEvent) -> Vec<f32> {
        let text = format!(
            "user={} content={} type={} genre={} watch={:.2} engagement={:.2}",
            event.user_id,
            event.content_id,
            event.content_type,
            event.genre,
            event.watch_percentage,
            event.engagement_score
        );
        self.embed_text(&text)
    }

    /// Generate embedding from content metadata
    pub fn embed_content(&self, content_id: &str, title: &str, genre: &str) -> Vec<f32> {
        let text = format!("content={} title={} genre={}", content_id, title, genre);
        self.embed_text(&text)
    }

    /// Generate deterministic embedding from text
    ///
    /// TODO: Replace with actual ONNX inference:
    /// ```ignore
    /// let session = onnxruntime::Session::new("minilm.onnx")?;
    /// let tokens = tokenize(text);
    /// let output = session.run(vec![tokens])?;
    /// output.into_vec()
    /// ```
    fn embed_text(&self, text: &str) -> Vec<f32> {
        // Create deterministic seed from text
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        // Generate pseudo-random normalized embedding
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut embedding: Vec<f32> = (0..self.dimension)
            .map(|_| rng.gen::<f32>() * 2.0 - 1.0)
            .collect();

        // L2 normalize (important for cosine similarity)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }

        embedding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_embedding() {
        let engine = EmbeddingEngine::new(384);

        let text = "test content";
        let emb1 = engine.embed_text(text);
        let emb2 = engine.embed_text(text);

        assert_eq!(emb1, emb2, "Embeddings should be deterministic");
        assert_eq!(emb1.len(), 384);
    }

    #[test]
    fn test_normalized_embedding() {
        let engine = EmbeddingEngine::new(384);

        let embedding = engine.embed_text("test");
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        assert!((norm - 1.0).abs() < 0.01, "Embedding should be L2 normalized");
    }
}
