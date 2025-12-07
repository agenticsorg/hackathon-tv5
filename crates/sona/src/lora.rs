//! Two-Tier LoRA (Low-Rank Adaptation)
//!
//! Implements UpdateUserLoRA and ComputeLoRAForward algorithms from SPARC pseudocode.
//! Provides per-user personalization with ~10KB memory footprint per user.

use crate::inference::ONNXInference;
use crate::types::ViewingEvent;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ndarray::{Array1, Array2};
use std::sync::Arc;
use uuid::Uuid;

const LORA_RANK: usize = 8;
const LORA_ALPHA: f32 = 16.0;
const INPUT_DIM: usize = 512;
const OUTPUT_DIM: usize = 768;
const LEARNING_RATE: f32 = 0.001;
const MIN_TRAINING_EVENTS: usize = 10;

/// User-specific LoRA adapter
///
/// Two-tier structure:
/// - Base layer weights (shared across users)
/// - User layer weights (per-user adaptation)
///
/// Memory footprint: ~10KB per user (rank=8)
#[derive(Debug, Clone)]
pub struct UserLoRAAdapter {
    pub user_id: Uuid,
    pub base_layer_weights: Array2<f32>, // [rank, input_dim]
    pub user_layer_weights: Array2<f32>, // [output_dim, rank]
    pub rank: usize,
    pub scaling_factor: f32, // alpha/rank
    pub last_trained_time: DateTime<Utc>,
    pub training_iterations: usize,
}

impl UserLoRAAdapter {
    pub fn new(user_id: Uuid) -> Self {
        let base_layer_weights = Array2::<f32>::zeros((LORA_RANK, INPUT_DIM));
        let user_layer_weights = Array2::<f32>::zeros((OUTPUT_DIM, LORA_RANK));

        Self {
            user_id,
            base_layer_weights,
            user_layer_weights,
            rank: LORA_RANK,
            scaling_factor: LORA_ALPHA / LORA_RANK as f32,
            last_trained_time: Utc::now(),
            training_iterations: 0,
        }
    }

    /// Initialize with random weights (Xavier initialization)
    pub fn initialize_random(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Xavier initialization for base layer
        let base_stddev = (2.0 / (LORA_RANK + INPUT_DIM) as f32).sqrt();
        for i in 0..LORA_RANK {
            for j in 0..INPUT_DIM {
                self.base_layer_weights[[i, j]] = rng.gen::<f32>() * base_stddev;
            }
        }

        // Xavier initialization for user layer
        let user_stddev = (2.0 / (OUTPUT_DIM + LORA_RANK) as f32).sqrt();
        for i in 0..OUTPUT_DIM {
            for j in 0..LORA_RANK {
                self.user_layer_weights[[i, j]] = rng.gen::<f32>() * user_stddev;
            }
        }
    }
}

/// Update user LoRA adapter with recent viewing events
///
/// Algorithm: UpdateUserLoRA (from SPARC pseudocode Part 2)
/// - Requires minimum 10 interactions
/// - Trains for 5 iterations
/// - Uses binary cross-entropy loss
/// - Gradient descent on user layer only
pub struct UpdateUserLoRA;

impl UpdateUserLoRA {
    /// Execute LoRA training with real embeddings from ONNX inference
    pub async fn execute_with_inference(
        adapter: &mut UserLoRAAdapter,
        recent_events: &[ViewingEvent],
        inference: Arc<ONNXInference>,
        get_content_text: impl Fn(Uuid) -> Result<String>,
        preference_vector: &[f32],
    ) -> Result<()> {
        // Check if enough new data for training
        if recent_events.len() < MIN_TRAINING_EVENTS {
            return Ok(());
        }

        // Prepare training data with real embeddings
        let mut training_pairs = Vec::new();
        for event in recent_events {
            let content_text = get_content_text(event.content_id)?;
            let content_embedding = inference.generate_embedding(&content_text).await?;
            let engagement_label = Self::calculate_engagement_label(event);
            training_pairs.push((content_embedding, engagement_label));
        }

        // LoRA training loop (few-shot adaptation)
        for iteration in 0..5 {
            let mut total_loss = 0.0;

            for (embedding, label) in &training_pairs {
                // Forward pass through LoRA
                let lora_output = ComputeLoRAForward::execute(adapter, embedding)?;

                // Predicted engagement (dot product + sigmoid)
                let predicted = Self::sigmoid(Self::dot_product(&lora_output, preference_vector));

                // Binary cross-entropy loss
                let loss = -label * predicted.ln() - (1.0 - label) * (1.0 - predicted).ln();
                total_loss += loss;

                // Backward pass (gradient descent on user layer only)
                let gradient_scalar = predicted - label;
                Self::update_user_layer_gradients(adapter, embedding, gradient_scalar)?;
            }

            let avg_loss = total_loss / training_pairs.len() as f32;
            tracing::debug!(
                "LoRA training iteration {}: avg_loss={}",
                iteration,
                avg_loss
            );
        }

        adapter.last_trained_time = Utc::now();
        adapter.training_iterations += 1;

        Ok(())
    }

    /// Legacy method - kept for backward compatibility
    pub async fn execute(
        adapter: &mut UserLoRAAdapter,
        recent_events: &[ViewingEvent],
        get_content_embedding: impl Fn(Uuid) -> Result<Vec<f32>>,
        preference_vector: &[f32],
    ) -> Result<()> {
        // Check if enough new data for training
        if recent_events.len() < MIN_TRAINING_EVENTS {
            return Ok(());
        }

        // Prepare training data
        let mut training_pairs = Vec::new();
        for event in recent_events {
            let content_embedding = get_content_embedding(event.content_id)?;
            let engagement_label = Self::calculate_engagement_label(event);
            training_pairs.push((content_embedding, engagement_label));
        }

        // LoRA training loop (few-shot adaptation)
        for iteration in 0..5 {
            let mut total_loss = 0.0;

            for (embedding, label) in &training_pairs {
                // Forward pass through LoRA
                let lora_output = ComputeLoRAForward::execute(adapter, embedding)?;

                // Predicted engagement (dot product + sigmoid)
                let predicted = Self::sigmoid(Self::dot_product(&lora_output, preference_vector));

                // Binary cross-entropy loss
                let loss = -label * predicted.ln() - (1.0 - label) * (1.0 - predicted).ln();
                total_loss += loss;

                // Backward pass (gradient descent on user layer only)
                let gradient_scalar = predicted - label;
                Self::update_user_layer_gradients(adapter, embedding, gradient_scalar)?;
            }

            let avg_loss = total_loss / training_pairs.len() as f32;
            tracing::debug!(
                "LoRA training iteration {}: avg_loss={}",
                iteration,
                avg_loss
            );
        }

        adapter.last_trained_time = Utc::now();
        adapter.training_iterations += 1;

        Ok(())
    }

    fn calculate_engagement_label(event: &ViewingEvent) -> f32 {
        const COMPLETION_WEIGHT: f32 = 0.4;
        const RATING_WEIGHT: f32 = 0.3;
        const REWATCH_WEIGHT: f32 = 0.2;

        let mut label = 0.0;

        let completion_score = 0.5 + (event.completion_rate - 0.3) / 1.4;
        label += completion_score * COMPLETION_WEIGHT;

        if let Some(rating) = event.rating {
            let rating_score = (rating as f32 - 1.0) / 4.0;
            label += rating_score * RATING_WEIGHT;
        } else {
            label += completion_score * RATING_WEIGHT * 0.5;
        }

        if event.is_rewatch {
            label += REWATCH_WEIGHT;
        }

        label.max(0.0).min(1.0)
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }

    fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    fn update_user_layer_gradients(
        adapter: &mut UserLoRAAdapter,
        embedding: &[f32],
        gradient_scalar: f32,
    ) -> Result<()> {
        // Compute intermediate activations
        let intermediate = adapter
            .base_layer_weights
            .dot(&Array1::from_vec(embedding.to_vec()));

        // Update user layer weights with gradient descent
        for i in 0..OUTPUT_DIM {
            for j in 0..LORA_RANK {
                let gradient = gradient_scalar * intermediate[j];
                adapter.user_layer_weights[[i, j]] -= LEARNING_RATE * gradient;
            }
        }

        Ok(())
    }
}

/// Compute LoRA forward pass
///
/// Algorithm: ComputeLoRAForward (from SPARC pseudocode Part 2)
/// LoRA formula: output = B * A * input * scaling_factor
/// where A: [rank, input_dim], B: [output_dim, rank]
pub struct ComputeLoRAForward;

impl ComputeLoRAForward {
    pub fn execute(adapter: &UserLoRAAdapter, input_vector: &[f32]) -> Result<Vec<f32>> {
        if input_vector.len() != INPUT_DIM {
            return Err(anyhow!(
                "Input vector dimension mismatch: expected {}, got {}",
                INPUT_DIM,
                input_vector.len()
            ));
        }

        let input_array = Array1::from_vec(input_vector.to_vec());

        // Low-rank projection: A * input
        let intermediate = adapter.base_layer_weights.dot(&input_array);

        // User-specific adaptation: B * intermediate
        let output = adapter.user_layer_weights.dot(&intermediate);

        // Scale by alpha/rank
        let scaled_output = output * adapter.scaling_factor;

        Ok(scaled_output.to_vec())
    }
}

/// Compute LoRA personalization score
pub fn compute_lora_score(
    adapter: &UserLoRAAdapter,
    content_embedding: &[f32],
    preference_vector: &[f32],
) -> Result<f32> {
    let lora_output = ComputeLoRAForward::execute(adapter, content_embedding)?;

    // Dot product similarity
    let score: f32 = lora_output
        .iter()
        .zip(preference_vector.iter())
        .map(|(a, b)| a * b)
        .sum();

    Ok(score.max(-1.0).min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lora_adapter_creation() {
        let adapter = UserLoRAAdapter::new(Uuid::new_v4());
        assert_eq!(adapter.rank, LORA_RANK);
        assert_eq!(adapter.scaling_factor, LORA_ALPHA / LORA_RANK as f32);
    }

    #[test]
    fn test_lora_forward_pass() {
        let mut adapter = UserLoRAAdapter::new(Uuid::new_v4());
        adapter.initialize_random();

        let input = vec![0.5; INPUT_DIM];
        let output = ComputeLoRAForward::execute(&adapter, &input).unwrap();

        assert_eq!(output.len(), OUTPUT_DIM);
    }
}
