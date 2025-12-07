//! ONNX Runtime Inference Engine
//!
//! Provides real embedding generation using ONNX Runtime.
//! Replaces dummy vec![0.0; 512] vectors with actual model inference.

use anyhow::{anyhow, Result};
use ndarray::{Array2, Axis};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ONNX-based embedding inference engine
pub struct ONNXInference {
    session: Arc<RwLock<Session>>,
    embedding_dim: usize,
    max_batch_size: usize,
}

impl ONNXInference {
    /// Create new ONNX inference engine
    ///
    /// # Arguments
    /// * `model_path` - Path to ONNX model file (.onnx)
    /// * `embedding_dim` - Expected embedding dimension (default: 512)
    pub fn new(model_path: impl AsRef<Path>, embedding_dim: usize) -> Result<Self> {
        let start = std::time::Instant::now();

        // ORT 2.x API: Session::builder() returns a SessionBuilder
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        let load_time = start.elapsed();
        tracing::info!(
            "ONNX model loaded in {:.2}s (embedding_dim={})",
            load_time.as_secs_f64(),
            embedding_dim
        );

        if load_time.as_millis() > 2000 {
            tracing::warn!(
                "Model loading time {}ms exceeds 2s target",
                load_time.as_millis()
            );
        }

        Ok(Self {
            session: Arc::new(RwLock::new(session)),
            embedding_dim,
            max_batch_size: 32,
        })
    }

    /// Create from environment variable path
    ///
    /// Reads model path from SONA_MODEL_PATH env var
    pub fn from_env() -> Result<Self> {
        let model_path = std::env::var("SONA_MODEL_PATH")
            .unwrap_or_else(|_| "/models/sona_embeddings.onnx".to_string());

        let embedding_dim = std::env::var("SONA_EMBEDDING_DIM")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(512);

        Self::new(model_path, embedding_dim)
    }

    /// Generate embedding for single text input
    ///
    /// # Performance Target
    /// <50ms per item
    ///
    /// # Arguments
    /// * `text` - Input text to embed
    ///
    /// # Returns
    /// Vector of length `embedding_dim`
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let start = std::time::Instant::now();

        // Tokenize text (simplified - in production use proper tokenizer)
        let tokens = self.tokenize(text)?;

        // Prepare input tensor
        let input_ids = Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;

        let mut session = self.session.write().await;

        // Run inference
        let input_tensor = Tensor::from_array(input_ids)?;
        let outputs = session.run(ort::inputs!["input_ids" => input_tensor])?;

        // Extract embedding from output
        let output_tensor = outputs["embeddings"].try_extract_array::<f32>()?.to_owned();

        let embedding: Vec<f32> = if output_tensor.ndim() == 2 {
            // Shape: [batch_size, embedding_dim]
            output_tensor
                .index_axis(Axis(0), 0)
                .iter()
                .copied()
                .collect()
        } else if output_tensor.ndim() == 3 {
            // Shape: [batch_size, seq_len, embedding_dim] - take mean pooling
            let batch = output_tensor.index_axis(Axis(0), 0);
            let mean = batch.mean_axis(Axis(0)).unwrap();
            mean.iter().copied().collect()
        } else {
            return Err(anyhow!(
                "Unexpected output tensor shape: {:?}",
                output_tensor.shape()
            ));
        };

        if embedding.len() != self.embedding_dim {
            return Err(anyhow!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.embedding_dim,
                embedding.len()
            ));
        }

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 50 {
            tracing::warn!(
                "Inference latency {}ms exceeds 50ms target for text: '{}'",
                elapsed.as_millis(),
                text.chars().take(50).collect::<String>()
            );
        }

        tracing::debug!("Generated embedding in {}ms", elapsed.as_millis());

        Ok(embedding)
    }

    /// Generate embeddings for batch of texts
    ///
    /// More efficient than calling generate_embedding repeatedly.
    ///
    /// # Arguments
    /// * `texts` - Batch of input texts
    ///
    /// # Returns
    /// Vector of embeddings, same order as input
    pub async fn generate_embeddings_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let start = std::time::Instant::now();

        // Process in chunks if batch too large
        if texts.len() > self.max_batch_size {
            let mut all_embeddings = Vec::with_capacity(texts.len());
            for chunk in texts.chunks(self.max_batch_size) {
                let chunk_embeddings = self.generate_embeddings_batch_internal(chunk).await?;
                all_embeddings.extend(chunk_embeddings);
            }
            return Ok(all_embeddings);
        }

        let embeddings = self.generate_embeddings_batch_internal(texts).await?;

        let elapsed = start.elapsed();
        let per_item = elapsed.as_millis() as f64 / texts.len() as f64;

        tracing::debug!(
            "Batch inference: {} items in {}ms ({:.1}ms/item)",
            texts.len(),
            elapsed.as_millis(),
            per_item
        );

        Ok(embeddings)
    }

    async fn generate_embeddings_batch_internal(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Tokenize all texts
        let mut all_tokens = Vec::new();
        let mut lengths = Vec::new();

        for text in texts {
            let tokens = self.tokenize(text)?;
            lengths.push(tokens.len());
            all_tokens.push(tokens);
        }

        // Pad to max length in batch
        let max_len = *lengths.iter().max().unwrap_or(&0);
        let batch_size = texts.len();

        let mut padded_tokens = Vec::with_capacity(batch_size * max_len);
        for tokens in &all_tokens {
            padded_tokens.extend(tokens);
            padded_tokens.resize(padded_tokens.len() + (max_len - tokens.len()), 0);
        }

        // Create input tensor
        let input_ids = Array2::from_shape_vec((batch_size, max_len), padded_tokens)?;

        let mut session = self.session.write().await;

        // Run inference
        let input_tensor = Tensor::from_array(input_ids)?;
        let outputs = session.run(ort::inputs!["input_ids" => input_tensor])?;

        // Extract embeddings
        let output_tensor = outputs["embeddings"].try_extract_array::<f32>()?.to_owned();

        let mut embeddings = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let embedding: Vec<f32> = if output_tensor.ndim() == 2 {
                output_tensor
                    .index_axis(Axis(0), i)
                    .iter()
                    .copied()
                    .collect()
            } else if output_tensor.ndim() == 3 {
                let batch = output_tensor.index_axis(Axis(0), i);
                let mean = batch.mean_axis(Axis(0)).unwrap();
                mean.iter().copied().collect()
            } else {
                return Err(anyhow!(
                    "Unexpected output tensor shape: {:?}",
                    output_tensor.shape()
                ));
            };

            if embedding.len() != self.embedding_dim {
                return Err(anyhow!(
                    "Embedding dimension mismatch: expected {}, got {}",
                    self.embedding_dim,
                    embedding.len()
                ));
            }

            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    /// Apply LoRA adapter weights to embedding
    ///
    /// Modulates base embedding with user-specific LoRA parameters
    ///
    /// # Arguments
    /// * `base_embedding` - Base model embedding
    /// * `lora_output` - LoRA forward pass output
    ///
    /// # Returns
    /// Combined embedding with LoRA adaptation
    pub fn apply_lora_adapter(
        &self,
        base_embedding: &[f32],
        lora_output: &[f32],
    ) -> Result<Vec<f32>> {
        if base_embedding.len() != self.embedding_dim {
            return Err(anyhow!(
                "Base embedding dimension mismatch: expected {}, got {}",
                self.embedding_dim,
                base_embedding.len()
            ));
        }

        // LoRA output should match embedding dim (truncate/pad if needed)
        let lora_vec = if lora_output.len() > self.embedding_dim {
            &lora_output[..self.embedding_dim]
        } else {
            lora_output
        };

        // Residual connection: base + lora
        let mut adapted = base_embedding.to_vec();
        for (i, &lora_val) in lora_vec.iter().enumerate() {
            adapted[i] += lora_val;
        }

        // L2 normalize
        let norm: f32 = adapted.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut adapted {
                *val /= norm;
            }
        }

        Ok(adapted)
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }

    /// Simple tokenization (placeholder - use real tokenizer in production)
    fn tokenize(&self, text: &str) -> Result<Vec<i64>> {
        // Simplified tokenization - in production use proper tokenizer
        // This is just a placeholder that converts chars to token IDs
        let tokens: Vec<i64> = text
            .chars()
            .take(512) // Max sequence length
            .map(|c| (c as u32 % 30000) as i64)
            .collect();

        if tokens.is_empty() {
            Ok(vec![0])
        } else {
            Ok(tokens)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Create a minimal mock ONNX model for testing
    fn create_mock_model_path() -> PathBuf {
        // In real tests, you would use a fixture model
        // For now, we'll skip tests that require actual model file
        PathBuf::from("/tmp/test_model.onnx")
    }

    #[test]
    fn test_tokenization() {
        let inference = ONNXInference {
            session: Arc::new(RwLock::new(unsafe { std::mem::zeroed() })),
            embedding_dim: 512,
            max_batch_size: 32,
        };

        let tokens = inference.tokenize("test input").unwrap();
        assert!(!tokens.is_empty());
        assert!(tokens.len() <= 512);
    }

    #[tokio::test]
    #[ignore] // Requires actual ONNX model file
    async fn test_generate_embedding() {
        let model_path = create_mock_model_path();
        if !model_path.exists() {
            return; // Skip if no model available
        }

        let inference = ONNXInference::new(model_path, 512).unwrap();
        let embedding = inference.generate_embedding("test text").await.unwrap();

        assert_eq!(embedding.len(), 512);
    }

    #[tokio::test]
    #[ignore] // Requires actual ONNX model file
    async fn test_batch_inference() {
        let model_path = create_mock_model_path();
        if !model_path.exists() {
            return;
        }

        let inference = ONNXInference::new(model_path, 512).unwrap();
        let texts = vec!["text1", "text2", "text3"];
        let embeddings = inference.generate_embeddings_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in embeddings {
            assert_eq!(emb.len(), 512);
        }
    }

    #[test]
    fn test_lora_adapter_application() {
        let inference = ONNXInference {
            session: Arc::new(RwLock::new(unsafe { std::mem::zeroed() })),
            embedding_dim: 512,
            max_batch_size: 32,
        };

        let base = vec![0.5; 512];
        let lora = vec![0.1; 512];

        let adapted = inference.apply_lora_adapter(&base, &lora).unwrap();

        assert_eq!(adapted.len(), 512);

        // Check normalization
        let norm: f32 = adapted.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_lora_dimension_mismatch() {
        let inference = ONNXInference {
            session: Arc::new(RwLock::new(unsafe { std::mem::zeroed() })),
            embedding_dim: 512,
            max_batch_size: 32,
        };

        let base = vec![0.5; 256]; // Wrong dimension
        let lora = vec![0.1; 512];

        let result = inference.apply_lora_adapter(&base, &lora);
        assert!(result.is_err());
    }

    #[test]
    fn test_lora_truncation() {
        let inference = ONNXInference {
            session: Arc::new(RwLock::new(unsafe { std::mem::zeroed() })),
            embedding_dim: 512,
            max_batch_size: 32,
        };

        let base = vec![0.5; 512];
        let lora = vec![0.1; 768]; // Larger than embedding_dim

        let adapted = inference.apply_lora_adapter(&base, &lora).unwrap();
        assert_eq!(adapted.len(), 512);
    }
}
