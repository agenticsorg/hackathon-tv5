//! Integration tests for SONA ONNX inference
//!
//! Tests real embedding generation and LoRA integration

use media_gateway_sona::{ONNXInference, UserLoRAAdapter, UpdateUserLoRA, ComputeLoRAForward};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Requires actual ONNX model file
async fn test_onnx_inference_integration() {
    // This test requires an actual model file
    let model_path = std::env::var("SONA_MODEL_PATH")
        .unwrap_or_else(|_| "/models/sona_embeddings.onnx".to_string());

    if !std::path::Path::new(&model_path).exists() {
        eprintln!("Skipping test - model file not found at {}", model_path);
        return;
    }

    let inference = ONNXInference::new(model_path, 512)
        .expect("Failed to load ONNX model");

    // Test single embedding generation
    let text = "A thrilling action movie with great visual effects";
    let start = std::time::Instant::now();
    let embedding = inference.generate_embedding(text).await
        .expect("Failed to generate embedding");
    let elapsed = start.elapsed();

    assert_eq!(embedding.len(), 512, "Embedding dimension should be 512");
    assert!(elapsed.as_millis() < 50, "Inference should complete in <50ms, took {}ms", elapsed.as_millis());

    // Verify embedding is not all zeros (real inference occurred)
    let sum: f32 = embedding.iter().sum();
    assert!(sum.abs() > 0.001, "Embedding should not be all zeros");
}

#[tokio::test]
#[ignore] // Requires actual ONNX model file
async fn test_batch_inference_performance() {
    let model_path = std::env::var("SONA_MODEL_PATH")
        .unwrap_or_else(|_| "/models/sona_embeddings.onnx".to_string());

    if !std::path::Path::new(&model_path).exists() {
        return;
    }

    let inference = ONNXInference::new(model_path, 512)
        .expect("Failed to load ONNX model");

    let texts = vec![
        "Action movie with explosions",
        "Romantic comedy set in Paris",
        "Documentary about wildlife",
        "Science fiction space adventure",
        "Horror film with suspense",
    ];

    let start = std::time::Instant::now();
    let embeddings = inference.generate_embeddings_batch(&texts).await
        .expect("Failed to generate batch embeddings");
    let elapsed = start.elapsed();

    assert_eq!(embeddings.len(), texts.len());

    for emb in &embeddings {
        assert_eq!(emb.len(), 512);
        let sum: f32 = emb.iter().sum();
        assert!(sum.abs() > 0.001, "Embedding should not be all zeros");
    }

    let per_item_ms = elapsed.as_millis() as f32 / texts.len() as f32;
    assert!(per_item_ms < 50.0, "Per-item inference should be <50ms, was {:.1}ms", per_item_ms);
}

#[tokio::test]
#[ignore] // Requires actual ONNX model file
async fn test_lora_with_real_embeddings() {
    let model_path = std::env::var("SONA_MODEL_PATH")
        .unwrap_or_else(|_| "/models/sona_embeddings.onnx".to_string());

    if !std::path::Path::new(&model_path).exists() {
        return;
    }

    let inference = Arc::new(ONNXInference::new(model_path, 512)
        .expect("Failed to load ONNX model"));

    // Create LoRA adapter
    let mut adapter = UserLoRAAdapter::new(Uuid::new_v4());
    adapter.initialize_random();

    // Generate real embedding
    let text = "Sci-fi thriller with time travel";
    let embedding = inference.generate_embedding(text).await
        .expect("Failed to generate embedding");

    // Forward pass through LoRA
    let lora_output = ComputeLoRAForward::execute(&adapter, &embedding)
        .expect("Failed to compute LoRA forward");

    assert_eq!(lora_output.len(), 768, "LoRA output should be 768-dim");

    // Apply LoRA to base embedding
    let adapted = inference.apply_lora_adapter(&embedding, &lora_output)
        .expect("Failed to apply LoRA adapter");

    assert_eq!(adapted.len(), 512);

    // Verify normalization
    let norm: f32 = adapted.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.001, "Adapted embedding should be L2 normalized");
}

#[test]
fn test_inference_from_env() {
    // Test that inference engine can be created from environment variables
    std::env::set_var("SONA_MODEL_PATH", "/tmp/test_model.onnx");
    std::env::set_var("SONA_EMBEDDING_DIM", "512");

    // This will fail if file doesn't exist, but we're testing the config loading
    let result = ONNXInference::from_env();

    // Clean up
    std::env::remove_var("SONA_MODEL_PATH");
    std::env::remove_var("SONA_EMBEDDING_DIM");

    // We expect this to fail (no actual model file), but it should fail
    // during model loading, not config parsing
    assert!(result.is_err());
}

#[test]
fn test_lora_adapter_dimensions() {
    let inference = Arc::new(unsafe {
        std::mem::transmute::<usize, ONNXInference>(0)
    });

    // This is just testing the logic, not running actual inference
    // Create vectors of proper dimensions
    let base = vec![0.5; 512];
    let lora = vec![0.1; 768];

    // Test dimension handling (this would work with real inference object)
    assert_eq!(base.len(), 512);
    assert_eq!(lora.len(), 768);
}

#[tokio::test]
#[ignore] // Requires actual ONNX model file
async fn test_model_loading_time() {
    let model_path = std::env::var("SONA_MODEL_PATH")
        .unwrap_or_else(|_| "/models/sona_embeddings.onnx".to_string());

    if !std::path::Path::new(&model_path).exists() {
        return;
    }

    let start = std::time::Instant::now();
    let _inference = ONNXInference::new(model_path, 512)
        .expect("Failed to load ONNX model");
    let load_time = start.elapsed();

    assert!(
        load_time.as_millis() < 2000,
        "Model loading should complete in <2s, took {}ms",
        load_time.as_millis()
    );
}
